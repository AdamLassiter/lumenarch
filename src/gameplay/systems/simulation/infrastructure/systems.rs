use super::*;

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
