use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    gameplay::components::{
        DestroyedModule,
        InfrastructureNetworkSummary,
        InfrastructureRouteKind,
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
    ship::{ModuleKind, ModuleVariant, ShipFoundationKind},
};

#[derive(Clone)]
struct ModuleSnapshot {
    module_id: u64,
    kind: ModuleKind,
    variant: ModuleVariant,
    grid_x: i32,
    grid_y: i32,
    power_draw: Option<i32>,
    producer_output: Option<i32>,
    reactor_output: Option<crate::gameplay::helpers::Fx>,
    storage: Option<StorageSnapshot>,
    processor: bool,
    weapon_requires_ammo: bool,
    destroyed: bool,
    junction_open: Option<bool>,
    valve_open: Option<bool>,
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
    mut ship_query: Query<(Entity, &mut ShipInfrastructureState), With<ShipRoot>>,
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
    for (ship_entity, mut infrastructure) in &mut ship_query {
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
                    variant: runtime.variant,
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

        *infrastructure =
            build_infrastructure_state(route_tiles, modules, infrastructure.strict_routing);
    }
}

/// Aggregates routed power into the legacy ship-level power summary used by HUD/control code.
pub(crate) fn update_routed_ship_power(
    mut ship_query: Query<(
        &ShipInfrastructureState,
        &ShipPowerModel,
        &mut ShipPowerState,
    )>,
) {
    for (infrastructure, power_model, mut power_state) in &mut ship_query {
        let mut generation = crate::gameplay::helpers::Fx::from_num(0);
        let mut draw = crate::gameplay::helpers::Fx::from_num(0);
        let mut reserve = crate::gameplay::helpers::Fx::from_num(0);
        for network in &infrastructure.networks {
            if network.kind != Some(InfrastructureRouteKind::Power) {
                continue;
            }
            generation += network.supply;
            draw += network.demand.min(network.supply + network.reserve);
            reserve += network.reserve;
        }
        power_state.generation = generation;
        power_state.draw = draw;
        power_state.surplus = generation - draw;
        power_state.stored_energy = reserve.min(power_model.battery_capacity);
        power_state.engines_powered = infrastructure
            .module_statuses
            .iter()
            .any(|status| status.kind == ModuleKind::Engine && status.powered);
        power_state.weapons_powered = infrastructure
            .module_statuses
            .iter()
            .any(|status| status.kind == ModuleKind::Turret && status.powered);
        power_state.engine_power_ratio = if power_state.engines_powered {
            crate::gameplay::helpers::Fx::from_num(1)
        } else {
            crate::gameplay::helpers::Fx::from_num(0)
        };
    }
}

fn build_infrastructure_state(
    route_tiles: BTreeMap<(i32, i32), InfrastructureRouteKind>,
    modules: Vec<ModuleSnapshot>,
    strict_routing: bool,
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
                supply: crate::gameplay::helpers::Fx::from_num(0),
                demand: crate::gameplay::helpers::Fx::from_num(0),
                reserve: crate::gameplay::helpers::Fx::from_num(0),
                flow: crate::gameplay::helpers::Fx::from_num(0),
                blockers: blocked_tiles.len() as u32,
            });
        }
    }

    let mut module_statuses = Vec::new();
    for module in &modules {
        let resource_networks = resource_networks_for_module(module, &tile_networks);
        let power_network = if module_needs_or_produces_power(module) {
            attached_network(module, InfrastructureRouteKind::Power, &tile_networks)
        } else {
            None
        };
        let duct_network = if matches!(module.kind, ModuleKind::O2Generator | ModuleKind::Cargo) {
            attached_network(module, InfrastructureRouteKind::OxygenDuct, &tile_networks)
        } else {
            None
        };
        let power_required = module.power_draw.is_some();

        for network_id in [power_network, duct_network]
            .into_iter()
            .flatten()
            .chain(resource_networks.iter().map(|(_, network_id)| *network_id))
        {
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
                network.reserve += crate::gameplay::helpers::Fx::from_num(output);
            }
            if let Some(draw) = module.power_draw {
                network.demand += crate::gameplay::helpers::Fx::from_num(draw);
            }
        }

        add_resource_supply_and_demand(module, &resource_networks, &mut networks);

        module_statuses.push(ModuleInfrastructureStatus {
            module_id: module.module_id,
            kind: module.kind,
            power_network,
            duct_network,
            resource_networks,
            powered: !power_required,
            power_required,
            blocked_reason: None,
        });
    }

    for network in &mut networks {
        network.attached_modules.sort();
        network.flow = network.supply.min(network.demand);
    }

    for status in &mut module_statuses {
        if status.power_required {
            status.powered = status
                .power_network
                .and_then(|id| networks.iter().find(|network| network.id == id))
                .is_some_and(|network| {
                    network.supply + network.reserve >= network.demand
                        && network.supply + network.reserve
                            > crate::gameplay::helpers::Fx::from_num(0)
                });
            if !status.powered {
                status.blocked_reason = Some(if status.power_network.is_some() {
                    "insufficient generation".to_string()
                } else {
                    "no wired power".to_string()
                });
            }
        }
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

fn cardinal_neighbors((x, y): (i32, i32)) -> [(i32, i32); 4] {
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

fn attached_network(
    module: &ModuleSnapshot,
    route_kind: InfrastructureRouteKind,
    tile_networks: &BTreeMap<(InfrastructureRouteKind, i32, i32), u32>,
) -> Option<u32> {
    [(module.grid_x, module.grid_y)]
        .into_iter()
        .chain(cardinal_neighbors((module.grid_x, module.grid_y)))
        .filter_map(|(x, y)| tile_networks.get(&(route_kind, x, y)).copied())
        .min()
}

fn resource_networks_for_module(
    module: &ModuleSnapshot,
    tile_networks: &BTreeMap<(InfrastructureRouteKind, i32, i32), u32>,
) -> Vec<(ResourceKind, u32)> {
    let mut resources = Vec::new();
    for resource in compatible_resources(module) {
        if let Some(network_id) = attached_network(
            module,
            InfrastructureRouteKind::Resource(resource),
            tile_networks,
        ) {
            resources.push((resource, network_id));
        }
    }
    resources.sort();
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

fn add_resource_supply_and_demand(
    module: &ModuleSnapshot,
    resource_networks: &[(ResourceKind, u32)],
    networks: &mut [InfrastructureNetworkSummary],
) {
    let Some(storage) = module.storage else {
        for (resource, network_id) in resource_networks {
            if let Some(network) = networks
                .iter_mut()
                .find(|network| network.id == *network_id)
            {
                network.demand += crate::gameplay::helpers::Fx::from_num(match resource {
                    ResourceKind::Fuel if module.kind == ModuleKind::Reactor => 1,
                    ResourceKind::Ammunition if module.weapon_requires_ammo => 1,
                    ResourceKind::RawSalvage if module.processor => 2,
                    _ => 0,
                });
            }
        }
        return;
    };

    for (resource, network_id) in resource_networks {
        if let Some(network) = networks
            .iter_mut()
            .find(|network| network.id == *network_id)
        {
            network.supply += crate::gameplay::helpers::Fx::from_num(match resource {
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
