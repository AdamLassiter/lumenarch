use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    gameplay::{
        components::{
            DestroyedModule,
            InfrastructureNetworkSummary,
            InfrastructureRouteKind,
            InfrastructureServiceStatus,
            Integrity,
            JunctionCommandState,
            ModuleInfrastructureStatus,
            PowerConsumer,
            PowerProducer,
            ProcessorModule,
            ReactorCommandState,
            ResourceKind,
            RuntimeFoundationVisual,
            RuntimeShipModule,
            ShipInfrastructureState,
            ShipPowerModel,
            ShipPowerState,
            ShipRoot,
            StorageModule,
            ValveCommandState,
            WeaponModule,
        },
        helpers::{Fx, cardinal_neighbors, component_service_coords, fx_from_time_delta},
    },
    ship::{ModuleKind, ShipFoundationKind},
};

#[derive(Clone)]
struct ModuleSnapshot {
    module_id: u64,
    kind: ModuleKind,
    grid_x: i32,
    grid_y: i32,
    power_draw: Option<i32>,
    producer_output: Option<i32>,
    reactor_output: Option<crate::helpers::Fx>,
    storage: Option<StorageSnapshot>,
    processor: bool,
    weapon_requires_ammo: bool,
    destroyed: bool,
    junction_open: Option<bool>,
    valve_open: Option<bool>,
}

#[derive(Clone, Copy)]
struct AttachedNetwork {
    id: u32,
    service_coord: (i32, i32),
}

#[derive(Clone, Copy)]
struct StorageSnapshot {
    accepts_fuel: bool,
    accepts_ammunition: bool,
    accepts_general: bool,
    accepts_oxygen: bool,
    fuel: u32,
    ammunition: u32,
    raw_salvage: u32,
    repair_charge: u32,
    oxygen: u32,
}

/// Rebuilds deterministic ship infrastructure graphs from route tiles and module placement.
pub(crate) fn rebuild_infrastructure_networks(
    mut ship_query: Query<(Entity, &mut ShipInfrastructureState, &ShipPowerModel), With<ShipRoot>>,
    foundation_query: Query<(&RuntimeFoundationVisual, &ChildOf)>,
    module_query: Query<(
        &RuntimeShipModule,
        &ChildOf,
        Option<&PowerConsumer>,
        Option<&PowerProducer>,
        Option<&ReactorCommandState>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&WeaponModule>,
        Option<&Integrity>,
        Option<&DestroyedModule>,
        Option<&JunctionCommandState>,
        Option<&ValveCommandState>,
    )>,
) {
    for (ship_entity, mut infrastructure, power_model) in &mut ship_query {
        let previous_reserve_by_network: BTreeMap<u32, Fx> = infrastructure
            .networks
            .iter()
            .filter(|network| network.kind == Some(InfrastructureRouteKind::Power))
            .map(|network| (network.id, network.reserve))
            .collect();
        let route_tiles: BTreeMap<(i32, i32), InfrastructureRouteKind> = foundation_query
            .iter()
            .filter(|(_, parent)| parent.get() == ship_entity)
            .filter_map(|(foundation, _)| {
                route_kind_for_foundation(foundation.kind)
                    .map(|kind| ((foundation.grid_x, foundation.grid_y), kind))
            })
            .collect();

        let modules: Vec<_> = module_query
            .iter()
            .filter(|(_, parent, ..)| parent.get() == ship_entity)
            .map(
                |(
                    runtime,
                    _,
                    consumer,
                    producer,
                    reactor,
                    storage,
                    processor,
                    weapon,
                    integrity,
                    destroyed,
                    junction,
                    valve,
                )| ModuleSnapshot {
                    module_id: runtime.module_id,
                    kind: runtime.kind,
                    grid_x: runtime.grid_x,
                    grid_y: runtime.grid_y,
                    power_draw: consumer
                        .map(|consumer| consumer.draw)
                        .or_else(|| implicit_power_draw(runtime.kind)),
                    producer_output: producer.map(|producer| producer.output),
                    reactor_output: reactor.map(|reactor| reactor.power_output),
                    storage: storage.map(|storage| StorageSnapshot {
                        accepts_fuel: storage.accepts_fuel,
                        accepts_ammunition: storage.accepts_ammunition,
                        accepts_general: storage.accepts_general,
                        accepts_oxygen: storage.accepts_oxygen,
                        fuel: storage.inventory.fuel,
                        ammunition: storage.inventory.ammunition,
                        raw_salvage: storage.inventory.raw_salvage,
                        repair_charge: storage.inventory.repair_charge,
                        oxygen: storage.inventory.oxygen,
                    }),
                    processor: processor.is_some(),
                    weapon_requires_ammo: weapon.is_some_and(|weapon| weapon.requires_ammo),
                    destroyed: destroyed.is_some()
                        || integrity.is_some_and(|integrity| integrity.current <= 0),
                    junction_open: junction.map(|junction| junction.open),
                    valve_open: valve.map(|valve| valve.open),
                },
            )
            .collect();

        *infrastructure = build_infrastructure_state(
            route_tiles,
            modules,
            infrastructure.strict_routing,
            &previous_reserve_by_network,
            power_model.battery_capacity,
        );
    }
}

/// Aggregates routed power into the legacy ship-level power summary used by HUD/control code.
pub(crate) fn update_routed_ship_power(
    time: Res<Time>,
    mut ship_query: Query<(
        &mut ShipInfrastructureState,
        &ShipPowerModel,
        &mut ShipPowerState,
    )>,
) {
    let dt = fx_from_time_delta(&time);
    for (mut infrastructure, power_model, mut power_state) in &mut ship_query {
        let _active_system_draw_capacity =
            power_model.engine_draw + power_model.weapon_draw + power_model.shield_draw;
        let mut generation = Fx::from_num(0);
        let mut draw = Fx::from_num(0);
        let mut reserve = Fx::from_num(0);
        let total_reserve_capacity = infrastructure
            .networks
            .iter()
            .filter(|network| network.kind == Some(InfrastructureRouteKind::Power))
            .map(|network| network.reserve_capacity)
            .fold(Fx::from_num(0), |total, capacity| total + capacity);
        for network in &mut infrastructure.networks {
            if network.kind != Some(InfrastructureRouteKind::Power) {
                continue;
            }
            let reserve_share = if total_reserve_capacity > Fx::from_num(0) {
                network.reserve_capacity / total_reserve_capacity
            } else {
                Fx::from_num(0)
            };
            let net_power = network.supply - network.demand;
            if net_power >= Fx::from_num(0) {
                let charge_delta =
                    (net_power * dt).min(power_model.battery_charge_rate * reserve_share * dt);
                network.reserve = (network.reserve + charge_delta).min(network.reserve_capacity);
            } else {
                let discharge_delta = ((-net_power) * dt)
                    .min(power_model.battery_discharge_rate * reserve_share * dt);
                network.reserve = (network.reserve - discharge_delta).max(Fx::from_num(0));
            }
            network.flow = network.supply.min(network.demand);
            generation += network.supply;
            draw += network.demand.min(network.supply + network.reserve);
            reserve += network.reserve;
        }
        power_state.generation = generation;
        power_state.draw = draw;
        power_state.surplus = generation - draw;
        power_state.stored_energy = reserve.min(power_model.battery_capacity);

        let network_power: BTreeMap<u32, bool> = infrastructure
            .networks
            .iter()
            .filter(|network| network.kind == Some(InfrastructureRouteKind::Power))
            .map(|network| {
                (
                    network.id,
                    network.supply + network.reserve >= network.demand
                        && network.supply + network.reserve > Fx::from_num(0),
                )
            })
            .collect();
        for status in &mut infrastructure.module_statuses {
            if !status.power_required {
                status.powered = true;
                status.blocked_reason = None;
                continue;
            }
            status.powered = status
                .power_network
                .and_then(|id| network_power.get(&id).copied())
                .unwrap_or(false);
            status.blocked_reason = (!status.powered).then(|| {
                if status.power_network.is_some() {
                    "insufficient generation".to_string()
                } else {
                    "no wired power".to_string()
                }
            });
        }
        power_state.engines_powered = infrastructure
            .module_statuses
            .iter()
            .any(|status| status.kind == ModuleKind::Engine && status.powered);
        power_state.weapons_powered = infrastructure
            .module_statuses
            .iter()
            .any(|status| status.kind == ModuleKind::Turret && status.powered);
        power_state.engine_power_ratio = if power_state.engines_powered {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        };
    }
}

fn build_infrastructure_state(
    route_tiles: BTreeMap<(i32, i32), InfrastructureRouteKind>,
    modules: Vec<ModuleSnapshot>,
    strict_routing: bool,
    previous_reserve_by_network: &BTreeMap<u32, Fx>,
    ship_battery_capacity: Fx,
) -> ShipInfrastructureState {
    let mut blocked_tiles_by_kind: BTreeMap<InfrastructureRouteKind, BTreeSet<(i32, i32)>> =
        BTreeMap::new();
    for module in &modules {
        if module.destroyed {
            continue;
        }
        if module.junction_open == Some(false) {
            blocked_tiles_by_kind
                .entry(InfrastructureRouteKind::Power)
                .or_default()
                .insert((module.grid_x, module.grid_y));
        }
        if module.valve_open == Some(false) {
            for kind in [
                InfrastructureRouteKind::OxygenDuct,
                InfrastructureRouteKind::Resource(ResourceKind::RawSalvage),
                InfrastructureRouteKind::Resource(ResourceKind::RepairCharge),
                InfrastructureRouteKind::Resource(ResourceKind::Fuel),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                InfrastructureRouteKind::Resource(ResourceKind::Oxygen),
            ] {
                blocked_tiles_by_kind
                    .entry(kind)
                    .or_default()
                    .insert((module.grid_x, module.grid_y));
            }
        }
    }

    let mut route_kinds: Vec<_> = route_tiles.values().copied().collect();
    route_kinds.sort();
    route_kinds.dedup();

    let mut tile_networks: BTreeMap<(InfrastructureRouteKind, i32, i32), u32> = BTreeMap::new();
    let mut networks = Vec::new();
    let mut next_id = 1;

    for route_kind in route_kinds {
        let blocked_tiles = blocked_tiles_by_kind
            .get(&route_kind)
            .cloned()
            .unwrap_or_default();
        let mut visited = BTreeSet::new();
        let mut coords: Vec<_> = route_tiles
            .iter()
            .filter_map(|(coord, kind)| {
                (*kind == route_kind && !blocked_tiles.contains(coord)).then_some(*coord)
            })
            .collect();
        coords.sort();

        for start in coords {
            if visited.contains(&start) {
                continue;
            }
            let network_id = next_id;
            next_id += 1;
            let mut queue = VecDeque::from([start]);
            visited.insert(start);
            let mut members = Vec::new();
            while let Some(coord) = queue.pop_front() {
                members.push(coord);
                tile_networks.insert((route_kind, coord.0, coord.1), network_id);
                for neighbor in cardinal_neighbors(coord) {
                    if visited.contains(&neighbor) || blocked_tiles.contains(&neighbor) {
                        continue;
                    }
                    if route_tiles.get(&neighbor) == Some(&route_kind) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
            members.sort();
            networks.push(InfrastructureNetworkSummary {
                id: network_id,
                kind: Some(route_kind),
                tile_count: members.len() as u32,
                tiles: members.clone(),
                attached_modules: Vec::new(),
                supply: crate::helpers::Fx::from_num(0),
                demand: crate::helpers::Fx::from_num(0),
                reserve: crate::helpers::Fx::from_num(0),
                reserve_capacity: crate::helpers::Fx::from_num(0),
                flow: crate::helpers::Fx::from_num(0),
                blockers: blocked_tiles.len() as u32,
            });
        }
    }

    let mut module_statuses = Vec::new();
    for module in &modules {
        let resource_networks = resource_networks_for_module(module, &tile_networks);
        let power_attachment = if module_needs_or_produces_power(module) {
            attached_network(module, InfrastructureRouteKind::Power, &tile_networks)
        } else {
            None
        };
        let duct_attachment = if matches!(module.kind, ModuleKind::O2Generator | ModuleKind::Cargo)
        {
            attached_network(module, InfrastructureRouteKind::OxygenDuct, &tile_networks)
        } else {
            None
        };
        let power_network = power_attachment.map(|attachment| attachment.id);
        let duct_network = duct_attachment.map(|attachment| attachment.id);
        let power_required = module.power_draw.is_some();
        let service_statuses = service_statuses_for_module(
            module,
            power_attachment,
            duct_attachment,
            &resource_networks,
            &route_tiles,
            &blocked_tiles_by_kind,
        );

        for network_id in [power_network, duct_network].into_iter().flatten().chain(
            resource_networks
                .iter()
                .map(|(_, attachment)| attachment.id),
        ) {
            if let Some(network) = networks.iter_mut().find(|network| network.id == network_id)
                && !network.attached_modules.contains(&module.module_id)
            {
                network.attached_modules.push(module.module_id);
            }
        }

        if let Some(power_network) = power_network
            && let Some(network) = networks
                .iter_mut()
                .find(|network| network.id == power_network)
        {
            if let Some(output) = module.reactor_output {
                network.supply += output;
            } else if let Some(output) = module.producer_output {
                network.reserve_capacity += crate::helpers::Fx::from_num(output);
            }
            if let Some(draw) = module.power_draw {
                network.demand += crate::helpers::Fx::from_num(draw);
            }
        }

        add_resource_supply_and_demand(module, &resource_networks, &mut networks);

        module_statuses.push(ModuleInfrastructureStatus {
            module_id: module.module_id,
            kind: module.kind,
            power_network,
            duct_network,
            resource_networks: resource_networks
                .iter()
                .map(|(kind, attachment)| (*kind, attachment.id))
                .collect(),
            service_statuses,
            powered: !power_required,
            power_required,
            blocked_reason: None,
        });
    }

    let total_reserve_units = networks
        .iter()
        .filter(|network| network.kind == Some(InfrastructureRouteKind::Power))
        .map(|network| network.reserve_capacity)
        .fold(Fx::from_num(0), |total, capacity| total + capacity);

    for network in &mut networks {
        network.attached_modules.sort();
        if network.kind == Some(InfrastructureRouteKind::Power)
            && total_reserve_units > Fx::from_num(0)
        {
            network.reserve_capacity =
                ship_battery_capacity * network.reserve_capacity / total_reserve_units;
        }
        network.reserve = if network.kind == Some(InfrastructureRouteKind::Power)
            && network.reserve_capacity > Fx::from_num(0)
        {
            previous_reserve_by_network
                .get(&network.id)
                .copied()
                .unwrap_or(network.reserve_capacity)
                .clamp(Fx::from_num(0), network.reserve_capacity)
        } else {
            Fx::from_num(0)
        };
        network.flow = network.supply.min(network.demand);
    }

    for status in &mut module_statuses {
        if status.power_required {
            status.powered = status
                .power_network
                .and_then(|id| networks.iter().find(|network| network.id == id))
                .is_some_and(|network| {
                    network.supply + network.reserve >= network.demand
                        && network.supply + network.reserve > crate::helpers::Fx::from_num(0)
                });
        }
        refresh_service_blockers(status, &networks);
        status.blocked_reason = status
            .service_statuses
            .iter()
            .filter(|service| service.required)
            .find_map(|service| service.blocked_reason.clone());
    }

    ShipInfrastructureState {
        networks,
        module_statuses,
        strict_routing,
    }
}

fn route_kind_for_foundation(kind: ShipFoundationKind) -> Option<InfrastructureRouteKind> {
    match kind {
        ShipFoundationKind::Wire => Some(InfrastructureRouteKind::Power),
        ShipFoundationKind::OxygenDuct => Some(InfrastructureRouteKind::OxygenDuct),
        ShipFoundationKind::PipeRawSalvage => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::RawSalvage))
        }
        ShipFoundationKind::PipeRepairCharge => Some(InfrastructureRouteKind::Resource(
            ResourceKind::RepairCharge,
        )),
        ShipFoundationKind::PipeFuel => Some(InfrastructureRouteKind::Resource(ResourceKind::Fuel)),
        ShipFoundationKind::PipeAmmunition => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::Ammunition))
        }
        ShipFoundationKind::PipeOxygen => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::Oxygen))
        }
        _ => None,
    }
}

fn attached_network(
    module: &ModuleSnapshot,
    route_kind: InfrastructureRouteKind,
    tile_networks: &BTreeMap<(InfrastructureRouteKind, i32, i32), u32>,
) -> Option<AttachedNetwork> {
    component_service_coords((module.grid_x, module.grid_y))
        .into_iter()
        .filter_map(|(x, y)| {
            tile_networks
                .get(&(route_kind, x, y))
                .copied()
                .map(|id| AttachedNetwork {
                    id,
                    service_coord: (x, y),
                })
        })
        .min_by_key(|attachment| {
            (
                attachment.id,
                attachment.service_coord.0,
                attachment.service_coord.1,
            )
        })
}

fn service_statuses_for_module(
    module: &ModuleSnapshot,
    power_attachment: Option<AttachedNetwork>,
    duct_attachment: Option<AttachedNetwork>,
    resource_networks: &[(ResourceKind, AttachedNetwork)],
    route_tiles: &BTreeMap<(i32, i32), InfrastructureRouteKind>,
    blocked_tiles_by_kind: &BTreeMap<InfrastructureRouteKind, BTreeSet<(i32, i32)>>,
) -> Vec<InfrastructureServiceStatus> {
    let mut services = Vec::new();
    if module_needs_or_produces_power(module) {
        let route_kind = InfrastructureRouteKind::Power;
        services.push(InfrastructureServiceStatus {
            route_kind,
            network_id: power_attachment.map(|attachment| attachment.id),
            service_coord: power_attachment.map(|attachment| attachment.service_coord),
            required: module.power_draw.is_some(),
            blocked_reason: blocked_service_reason(
                module,
                route_kind,
                route_tiles,
                blocked_tiles_by_kind,
            ),
        });
    }
    if matches!(module.kind, ModuleKind::O2Generator | ModuleKind::Cargo) {
        let route_kind = InfrastructureRouteKind::OxygenDuct;
        services.push(InfrastructureServiceStatus {
            route_kind,
            network_id: duct_attachment.map(|attachment| attachment.id),
            service_coord: duct_attachment.map(|attachment| attachment.service_coord),
            required: false,
            blocked_reason: blocked_service_reason(
                module,
                route_kind,
                route_tiles,
                blocked_tiles_by_kind,
            ),
        });
    }
    for resource in compatible_resources(module) {
        let route_kind = InfrastructureRouteKind::Resource(resource);
        services.push(InfrastructureServiceStatus {
            route_kind,
            network_id: resource_networks
                .iter()
                .find_map(|(kind, attachment)| (*kind == resource).then_some(attachment.id)),
            service_coord: resource_networks.iter().find_map(|(kind, attachment)| {
                (*kind == resource).then_some(attachment.service_coord)
            }),
            required: required_resources(module).contains(&resource),
            blocked_reason: blocked_service_reason(
                module,
                route_kind,
                route_tiles,
                blocked_tiles_by_kind,
            ),
        });
    }
    services
}

fn blocked_service_reason(
    module: &ModuleSnapshot,
    route_kind: InfrastructureRouteKind,
    route_tiles: &BTreeMap<(i32, i32), InfrastructureRouteKind>,
    blocked_tiles_by_kind: &BTreeMap<InfrastructureRouteKind, BTreeSet<(i32, i32)>>,
) -> Option<String> {
    let blocked_tiles = blocked_tiles_by_kind.get(&route_kind)?;
    component_service_coords((module.grid_x, module.grid_y))
        .into_iter()
        .any(|coord| route_tiles.get(&coord) == Some(&route_kind) && blocked_tiles.contains(&coord))
        .then(|| match route_kind {
            InfrastructureRouteKind::Power => "closed junction".to_string(),
            InfrastructureRouteKind::OxygenDuct | InfrastructureRouteKind::Resource(_) => {
                "closed valve".to_string()
            }
        })
}

fn refresh_service_blockers(
    status: &mut ModuleInfrastructureStatus,
    networks: &[InfrastructureNetworkSummary],
) {
    for service in &mut status.service_statuses {
        service.blocked_reason = match service.route_kind {
            InfrastructureRouteKind::Power if service.required && !status.powered => {
                service.blocked_reason.clone().or_else(|| {
                    Some(if service.network_id.is_some() {
                        "insufficient generation".to_string()
                    } else {
                        "no wired power".to_string()
                    })
                })
            }
            InfrastructureRouteKind::Resource(_) if service.required => {
                if let Some(network_id) = service.network_id {
                    if service.blocked_reason.is_some() {
                        service.blocked_reason.clone()
                    } else {
                        let supply = networks
                            .iter()
                            .find(|network| network.id == network_id)
                            .map(|network| network.supply)
                            .unwrap_or(Fx::from_num(0));
                        (supply <= Fx::from_num(0)).then(|| "insufficient supply".to_string())
                    }
                } else {
                    service
                        .blocked_reason
                        .clone()
                        .or_else(|| Some("no compatible resource".to_string()))
                }
            }
            _ => service.blocked_reason.clone(),
        };
    }
}

fn resource_networks_for_module(
    module: &ModuleSnapshot,
    tile_networks: &BTreeMap<(InfrastructureRouteKind, i32, i32), u32>,
) -> Vec<(ResourceKind, AttachedNetwork)> {
    let mut resources = Vec::new();
    for resource in compatible_resources(module) {
        if let Some(attachment) = attached_network(
            module,
            InfrastructureRouteKind::Resource(resource),
            tile_networks,
        ) {
            resources.push((resource, attachment));
        }
    }
    resources.sort_by_key(|(kind, attachment)| (*kind, attachment.id, attachment.service_coord));
    resources
}

fn compatible_resources(module: &ModuleSnapshot) -> Vec<ResourceKind> {
    let mut resources = Vec::new();
    if module.kind == ModuleKind::Reactor {
        resources.push(ResourceKind::Fuel);
    }
    if module.weapon_requires_ammo {
        resources.push(ResourceKind::Ammunition);
    }
    if module.processor {
        resources.extend([
            ResourceKind::RawSalvage,
            ResourceKind::RepairCharge,
            ResourceKind::Fuel,
            ResourceKind::Ammunition,
        ]);
    }
    if let Some(storage) = module.storage {
        if storage.accepts_general {
            resources.extend([ResourceKind::RawSalvage, ResourceKind::RepairCharge]);
        }
        if storage.accepts_fuel {
            resources.push(ResourceKind::Fuel);
        }
        if storage.accepts_ammunition {
            resources.push(ResourceKind::Ammunition);
        }
        if storage.accepts_oxygen {
            resources.push(ResourceKind::Oxygen);
        }
    }
    resources.sort();
    resources.dedup();
    resources
}

fn required_resources(module: &ModuleSnapshot) -> Vec<ResourceKind> {
    let mut resources = Vec::new();
    if module.kind == ModuleKind::Reactor {
        resources.push(ResourceKind::Fuel);
    }
    if module.weapon_requires_ammo {
        resources.push(ResourceKind::Ammunition);
    }
    if module.processor {
        resources.push(ResourceKind::RawSalvage);
    }
    resources
}

fn add_resource_supply_and_demand(
    module: &ModuleSnapshot,
    resource_networks: &[(ResourceKind, AttachedNetwork)],
    networks: &mut [InfrastructureNetworkSummary],
) {
    let Some(storage) = module.storage else {
        for (resource, attachment) in resource_networks {
            if let Some(network) = networks
                .iter_mut()
                .find(|network| network.id == attachment.id)
            {
                network.demand += crate::helpers::Fx::from_num(match resource {
                    ResourceKind::Fuel if module.kind == ModuleKind::Reactor => 1,
                    ResourceKind::Ammunition if module.weapon_requires_ammo => 1,
                    ResourceKind::RawSalvage if module.processor => 2,
                    _ => 0,
                });
            }
        }
        return;
    };

    for (resource, attachment) in resource_networks {
        if let Some(network) = networks
            .iter_mut()
            .find(|network| network.id == attachment.id)
        {
            network.supply += crate::helpers::Fx::from_num(match resource {
                ResourceKind::RawSalvage => storage.raw_salvage,
                ResourceKind::RepairCharge => storage.repair_charge,
                ResourceKind::Fuel => storage.fuel,
                ResourceKind::Ammunition => storage.ammunition,
                ResourceKind::Oxygen => storage.oxygen,
            });
        }
    }
}

fn module_needs_or_produces_power(module: &ModuleSnapshot) -> bool {
    module.power_draw.is_some()
        || module.producer_output.is_some()
        || module.reactor_output.is_some()
}

fn implicit_power_draw(kind: ModuleKind) -> Option<i32> {
    match kind {
        ModuleKind::Cockpit | ModuleKind::Computer => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{ModuleSnapshot, StorageSnapshot, build_infrastructure_state};
    use crate::{
        gameplay::components::{InfrastructureRouteKind, ResourceKind},
        helpers::Fx,
        ship::ModuleKind,
    };

    fn module(module_id: u64, kind: ModuleKind, grid_x: i32, grid_y: i32) -> ModuleSnapshot {
        ModuleSnapshot {
            module_id,
            kind,
            grid_x,
            grid_y,
            power_draw: None,
            producer_output: None,
            reactor_output: None,
            storage: None,
            processor: false,
            weapon_requires_ammo: false,
            destroyed: false,
            junction_open: None,
            valve_open: None,
        }
    }

    fn powered_turret(grid_x: i32, grid_y: i32) -> ModuleSnapshot {
        ModuleSnapshot {
            power_draw: Some(2),
            weapon_requires_ammo: true,
            ..module(1, ModuleKind::Turret, grid_x, grid_y)
        }
    }

    fn reactor(module_id: u64, grid_x: i32, grid_y: i32) -> ModuleSnapshot {
        ModuleSnapshot {
            reactor_output: Some(Fx::from_num(10)),
            ..module(module_id, ModuleKind::Reactor, grid_x, grid_y)
        }
    }

    fn ammunition_storage(
        module_id: u64,
        grid_x: i32,
        grid_y: i32,
        ammunition: u32,
    ) -> ModuleSnapshot {
        ModuleSnapshot {
            storage: Some(StorageSnapshot {
                accepts_fuel: false,
                accepts_ammunition: true,
                accepts_general: false,
                accepts_oxygen: false,
                fuel: 0,
                ammunition,
                raw_salvage: 0,
                repair_charge: 0,
                oxygen: 0,
            }),
            ..module(module_id, ModuleKind::Cargo, grid_x, grid_y)
        }
    }

    fn fuel_storage(module_id: u64, grid_x: i32, grid_y: i32, fuel: u32) -> ModuleSnapshot {
        ModuleSnapshot {
            storage: Some(StorageSnapshot {
                accepts_fuel: true,
                accepts_ammunition: false,
                accepts_general: false,
                accepts_oxygen: false,
                fuel,
                ammunition: 0,
                raw_salvage: 0,
                repair_charge: 0,
                oxygen: 0,
            }),
            ..module(module_id, ModuleKind::Cargo, grid_x, grid_y)
        }
    }

    fn state(
        routes: &[((i32, i32), InfrastructureRouteKind)],
        modules: Vec<ModuleSnapshot>,
    ) -> crate::gameplay::components::ShipInfrastructureState {
        build_infrastructure_state(
            routes.iter().copied().collect::<BTreeMap<_, _>>(),
            modules,
            true,
            &BTreeMap::new(),
            Fx::from_num(0),
        )
    }

    #[test]
    fn turret_connects_to_power_under_and_ammunition_adjacent() {
        let infrastructure = state(
            &[
                ((0, 0), InfrastructureRouteKind::Power),
                (
                    (1, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, -1, 0),
                ammunition_storage(3, 2, 0, 4),
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.powered);
        assert!(turret.power_network.is_some());
        assert_eq!(
            turret
                .service_statuses
                .iter()
                .find(|service| service.route_kind == InfrastructureRouteKind::Power)
                .and_then(|service| service.service_coord),
            Some((0, 0))
        );
        assert!(
            turret
                .network_for_resource(ResourceKind::Ammunition)
                .is_some()
        );
        assert_eq!(
            turret
                .service_statuses
                .iter()
                .find(|service| {
                    service.route_kind
                        == InfrastructureRouteKind::Resource(ResourceKind::Ammunition)
                })
                .and_then(|service| service.service_coord),
            Some((1, 0))
        );
        assert!(turret.blocked_reason.is_none());
    }

    #[test]
    fn turret_connects_to_ammunition_under_and_power_adjacent() {
        let infrastructure = state(
            &[
                (
                    (0, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
                ((1, 0), InfrastructureRouteKind::Power),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, 2, 0),
                ammunition_storage(3, -1, 0, 4),
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.powered);
        assert!(turret.power_network.is_some());
        assert_eq!(
            turret
                .service_statuses
                .iter()
                .find(|service| service.route_kind == InfrastructureRouteKind::Power)
                .and_then(|service| service.service_coord),
            Some((1, 0))
        );
        assert!(
            turret
                .network_for_resource(ResourceKind::Ammunition)
                .is_some()
        );
        assert_eq!(
            turret
                .service_statuses
                .iter()
                .find(|service| {
                    service.route_kind
                        == InfrastructureRouteKind::Resource(ResourceKind::Ammunition)
                })
                .and_then(|service| service.service_coord),
            Some((0, 0))
        );
        assert!(turret.blocked_reason.is_none());
    }

    #[test]
    fn turret_with_power_but_no_ammunition_reports_missing_resource() {
        let infrastructure = state(
            &[((0, 0), InfrastructureRouteKind::Power)],
            vec![powered_turret(0, 0), reactor(2, 1, 0)],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.powered);
        assert_eq!(
            turret.blocked_reason.as_deref(),
            Some("no compatible resource")
        );
    }

    #[test]
    fn turret_with_empty_ammunition_network_reports_insufficient_supply() {
        let infrastructure = state(
            &[
                ((0, 0), InfrastructureRouteKind::Power),
                (
                    (1, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, -1, 0),
                ammunition_storage(3, 2, 0, 0),
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.powered);
        assert_eq!(
            turret.blocked_reason.as_deref(),
            Some("insufficient supply")
        );
    }

    #[test]
    fn turret_with_ammunition_but_no_power_reports_no_wired_power() {
        let infrastructure = state(
            &[(
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            )],
            vec![powered_turret(0, 0), ammunition_storage(2, 2, 0, 4)],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(!turret.powered);
        assert_eq!(turret.blocked_reason.as_deref(), Some("no wired power"));
    }

    #[test]
    fn diagonal_routes_do_not_attach_to_component_service_ports() {
        let infrastructure = state(
            &[
                ((1, 1), InfrastructureRouteKind::Power),
                (
                    (-1, -1),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, 1, 2),
                ammunition_storage(3, -1, -2, 4),
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.power_network.is_none());
        assert!(
            turret
                .service_statuses
                .iter()
                .all(|service| service.service_coord.is_none())
        );
        assert!(
            turret
                .network_for_resource(ResourceKind::Ammunition)
                .is_none()
        );
    }

    #[test]
    fn different_route_kinds_do_not_merge_in_adjacent_service_area() {
        let infrastructure = state(
            &[
                ((0, 0), InfrastructureRouteKind::Power),
                (
                    (1, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, -1, 0),
                ammunition_storage(3, 2, 0, 4),
            ],
        );

        let power_network = infrastructure
            .networks
            .iter()
            .find(|network| network.kind == Some(InfrastructureRouteKind::Power))
            .unwrap();
        let ammo_network = infrastructure
            .networks
            .iter()
            .find(|network| {
                network.kind == Some(InfrastructureRouteKind::Resource(ResourceKind::Ammunition))
            })
            .unwrap();
        assert_ne!(power_network.id, ammo_network.id);
        assert_eq!(power_network.tile_count, 1);
        assert_eq!(ammo_network.tile_count, 1);
    }

    #[test]
    fn reactor_can_inject_power_and_consume_adjacent_fuel() {
        let infrastructure = state(
            &[
                ((0, 0), InfrastructureRouteKind::Power),
                (
                    (1, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Fuel),
                ),
            ],
            vec![reactor(1, 0, 0), fuel_storage(2, 2, 0, 3)],
        );

        let reactor = infrastructure.status_for_module(1).unwrap();
        assert!(reactor.power_network.is_some());
        assert!(reactor.network_for_resource(ResourceKind::Fuel).is_some());
        assert!(reactor.blocked_reason.is_none());
    }

    #[test]
    fn closed_adjacent_junction_breaks_power_without_breaking_ammunition() {
        let mut junction = module(4, ModuleKind::JunctionBox, 1, 0);
        junction.junction_open = Some(false);
        let infrastructure = state(
            &[
                ((1, 0), InfrastructureRouteKind::Power),
                (
                    (0, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, 2, 0),
                ammunition_storage(3, -1, 0, 4),
                junction,
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert_eq!(turret.blocked_reason.as_deref(), Some("closed junction"));
        assert!(
            turret
                .network_for_resource(ResourceKind::Ammunition)
                .is_some()
        );
    }

    #[test]
    fn closed_adjacent_valve_breaks_ammunition_without_breaking_power() {
        let mut valve = module(4, ModuleKind::Valve, 1, 0);
        valve.valve_open = Some(false);
        let infrastructure = state(
            &[
                ((0, 0), InfrastructureRouteKind::Power),
                (
                    (1, 0),
                    InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
                ),
            ],
            vec![
                powered_turret(0, 0),
                reactor(2, -1, 0),
                ammunition_storage(3, 2, 0, 4),
                valve,
            ],
        );

        let turret = infrastructure.status_for_module(1).unwrap();
        assert!(turret.powered);
        assert_eq!(turret.blocked_reason.as_deref(), Some("closed valve"));
    }
}
