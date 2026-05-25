use super::*;

pub(super) fn build_infrastructure_state(
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

pub(super) fn route_kind_for_foundation(
    kind: ShipFoundationKind,
) -> Option<InfrastructureRouteKind> {
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

pub(super) fn attached_network(
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

pub(super) fn service_statuses_for_module(
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

pub(super) fn blocked_service_reason(
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

pub(super) fn refresh_service_blockers(
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

pub(super) fn resource_networks_for_module(
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

pub(super) fn compatible_resources(module: &ModuleSnapshot) -> Vec<ResourceKind> {
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

pub(super) fn required_resources(module: &ModuleSnapshot) -> Vec<ResourceKind> {
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

pub(super) fn add_resource_supply_and_demand(
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

pub(super) fn module_needs_or_produces_power(module: &ModuleSnapshot) -> bool {
    module.power_draw.is_some()
        || module.producer_output.is_some()
        || module.reactor_output.is_some()
}

pub(super) fn implicit_power_draw(kind: ModuleKind) -> Option<i32> {
    match kind {
        ModuleKind::Cockpit | ModuleKind::Computer => Some(1),
        _ => None,
    }
}
