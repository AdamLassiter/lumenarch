use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            DestroyedModule,
            DronePhase,
            DroneStationCommandState,
            DroneStationModule,
            DroneTask,
            DroneUnit,
            MissionState,
            PlayerShip,
            ProcessorModule,
            ResourceKind,
            RuntimeShipModule,
            ShipRoot,
            StorageModule,
        },
        helpers::{FixedVec2, Fx, fx_from_time_delta, local_field_distance, resource_kind_label},
        systems::simulation::logistics::{LogisticsEndpointSnapshot, collect_endpoint_snapshots},
    },
    ship::ModuleKind,
    state::PlayingCleanup,
};

#[derive(Clone, Copy)]
pub(super) struct DroneTransferPlan {
    pub(super) source_module_id: u64,
    pub(super) target_module_id: u64,
    pub(super) resource_kind: ResourceKind,
}
pub(crate) fn sync_drone_station_population(
    mut commands: Commands,
    balance: Res<BalanceConfig>,
    ship_query: Single<Entity, (With<PlayerShip>, With<ShipRoot>)>,
    station_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &DroneStationModule,
            Option<&DestroyedModule>,
        ),
        With<DroneStationModule>,
    >,
    drone_query: Query<(Entity, &DroneUnit)>,
) {
    let ship_entity = ship_query.into_inner();
    let mut existing: BTreeMap<Entity, Vec<Entity>> = BTreeMap::new();
    for (entity, drone) in &drone_query {
        existing
            .entry(drone.station_entity)
            .or_default()
            .push(entity);
    }

    for (station_entity, runtime_module, station, destroyed) in &station_query {
        let drones = existing.remove(&station_entity).unwrap_or_default();
        if destroyed.is_some() {
            for drone_entity in drones {
                commands.entity(drone_entity).despawn();
            }
            continue;
        }

        let desired = station.max_drones as usize;
        if drones.len() > desired {
            for drone_entity in drones.iter().skip(desired) {
                commands.entity(*drone_entity).despawn();
            }
        } else if drones.len() < desired {
            for index in drones.len()..desired {
                let offset = match index {
                    0 => FixedVec2::from_num(-12.0, -18.0),
                    1 => FixedVec2::from_num(12.0, -18.0),
                    _ => FixedVec2::from_num(-12.0 + index as f32 * 6.0, -22.0),
                };
                let home = runtime_module.local_position + offset;
                let entity = commands
                    .spawn((
                        Sprite::from_color(Color::srgb(0.62, 0.90, 1.0), Vec2::splat(8.0)),
                        Transform::from_translation(Vec3::new(
                            home.x.to_num::<f32>(),
                            home.y.to_num::<f32>(),
                            5.8,
                        )),
                        DroneUnit {
                            station_entity,
                            station_module_id: runtime_module.module_id,
                            home_local_position: home,
                            local_position: home,
                            phase: DronePhase::Idle,
                            mode: DroneTask::Logistics,
                            source_module_id: None,
                            target_module_id: None,
                            resource_kind: None,
                            payload_amount: 0,
                            reserved_amount: 0,
                        },
                        PlayingCleanup,
                    ))
                    .id();
                commands.entity(entity).set_parent_in_place(ship_entity);
            }
        }
    }

    for orphaned_drones in existing.into_values() {
        for entity in orphaned_drones {
            commands.entity(entity).despawn();
        }
    }

    let _ = balance;
}

pub(crate) fn run_drone_logistics(
    time: Res<Time>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut station_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &mut DroneStationModule,
            Option<&DroneStationCommandState>,
            Option<&DestroyedModule>,
        ),
        With<DroneStationModule>,
    >,
    mut logistics_sets: ParamSet<(
        Query<(
            Entity,
            &RuntimeShipModule,
            Option<&StorageModule>,
            Option<&ProcessorModule>,
            Option<&DestroyedModule>,
        )>,
        Query<(
            Entity,
            &RuntimeShipModule,
            Option<&mut StorageModule>,
            Option<&mut ProcessorModule>,
            Option<&DestroyedModule>,
        )>,
    )>,
    mut drone_query: Query<(Entity, &mut DroneUnit, &mut Transform, &mut Sprite)>,
) {
    let dt = fx_from_time_delta(&time);
    let mut mission_state = mission_query.into_inner();
    let endpoints = {
        let logistics_query = logistics_sets.p0();
        collect_endpoint_snapshots(&logistics_query)
    };

    let mut drones_by_station: BTreeMap<Entity, Vec<Entity>> = BTreeMap::new();
    for (entity, drone, _, _) in &mut drone_query {
        drones_by_station
            .entry(drone.station_entity)
            .or_default()
            .push(entity);
    }

    let mut stations: Vec<_> = station_query
        .iter()
        .map(|(entity, runtime_module, _, _, _)| (entity, runtime_module.module_id))
        .collect();
    stations.sort_by_key(|(_, module_id)| *module_id);

    for (station_entity, _) in stations {
        let Ok((station_entity, runtime_module, mut station, command_state, destroyed)) =
            station_query.get_mut(station_entity)
        else {
            continue;
        };
        if destroyed.is_some() {
            station.active_drones = 0;
            station.active_tasks = 0;
            station.queued_tasks = 0;
            station.idle_drones = 0;
            station.power_draw = Fx::from_num(0);
            station.last_status = "Offline".to_string();
            continue;
        }

        let mode = command_state
            .map(|command| command.selected_task)
            .unwrap_or(DroneTask::Logistics);
        let mut drone_entities = drones_by_station
            .remove(&station_entity)
            .unwrap_or_default();
        drone_entities.sort_by_key(|entity| entity.index());

        let mut reservations: BTreeMap<(u64, ResourceKind), u32> = BTreeMap::new();
        let mut target_reservations: BTreeMap<(u64, ResourceKind), u32> = BTreeMap::new();
        for drone_entity in &drone_entities {
            let Ok((_, drone, _, _)) = drone_query.get_mut(*drone_entity) else {
                continue;
            };
            if let (Some(source), Some(resource)) = (drone.source_module_id, drone.resource_kind)
                && drone.reserved_amount > 0
            {
                *reservations.entry((source, resource)).or_default() += drone.reserved_amount;
            }
            if let (Some(target), Some(resource)) = (drone.target_module_id, drone.resource_kind)
                && drone.payload_amount > 0
            {
                *target_reservations.entry((target, resource)).or_default() += drone.payload_amount;
            }
        }

        let mut busy = 0u32;
        let mut idle = 0u32;
        let mut first_status = None;

        for drone_entity in drone_entities {
            let Ok((_, mut drone, mut transform, mut sprite)) = drone_query.get_mut(drone_entity)
            else {
                continue;
            };

            if matches!(mode, DroneTask::Idle) {
                drone.phase = DronePhase::Returning;
                drone.source_module_id = None;
                drone.target_module_id = None;
                drone.resource_kind = None;
                drone.reserved_amount = 0;
            } else if matches!(mode, DroneTask::Return) {
                drone.phase = DronePhase::Returning;
            }

            if drone.phase == DronePhase::Idle
                && matches!(mode, DroneTask::Salvage | DroneTask::Logistics)
                && let Some(plan) = plan_drone_transfer(
                    &endpoints,
                    runtime_module.local_position,
                    station.operational_range,
                    mode,
                    &reservations,
                    &target_reservations,
                )
            {
                drone.mode = mode;
                drone.phase = DronePhase::Assigned;
                drone.source_module_id = Some(plan.source_module_id);
                drone.target_module_id = Some(plan.target_module_id);
                drone.resource_kind = Some(plan.resource_kind);
                drone.reserved_amount = 1;
                *reservations
                    .entry((plan.source_module_id, plan.resource_kind))
                    .or_default() += 1;
            }

            advance_drone_state(
                &mut drone,
                &endpoints,
                &mut logistics_sets.p1(),
                dt,
                &mut mission_state,
            );

            if drone.phase == DronePhase::Idle {
                idle += 1;
            } else {
                busy += 1;
            }
            if first_status.is_none() && drone.phase != DronePhase::Idle {
                first_status = Some(drone.phase.as_str().to_string());
            }

            transform.translation = Vec3::new(
                drone.local_position.x.to_num::<f32>(),
                drone.local_position.y.to_num::<f32>(),
                5.8,
            );
            sprite.color = if drone.payload_amount > 0 {
                Color::srgb(1.0, 0.76, 0.32)
            } else if drone.phase == DronePhase::Idle {
                Color::srgb(0.62, 0.90, 1.0)
            } else {
                Color::srgb(0.48, 0.98, 0.76)
            };
        }

        station.active_drones = station.max_drones;
        station.active_tasks = busy;
        station.idle_drones = idle;
        station.queued_tasks = count_drone_transfer_candidates(
            &endpoints,
            runtime_module.local_position,
            station.operational_range,
            mode,
            &reservations,
            &target_reservations,
        );
        station.power_draw = Fx::from_num(busy) * Fx::from_num(0.35);
        station.last_status = first_status.unwrap_or_else(|| {
            if station.queued_tasks > 0 {
                "Queue Ready".to_string()
            } else {
                "Idle".to_string()
            }
        });
    }
}

pub(super) fn plan_drone_transfer(
    endpoints: &[LogisticsEndpointSnapshot],
    station_position: FixedVec2,
    range: Fx,
    mode: DroneTask,
    reservations: &BTreeMap<(u64, ResourceKind), u32>,
    target_reservations: &BTreeMap<(u64, ResourceKind), u32>,
) -> Option<DroneTransferPlan> {
    if matches!(mode, DroneTask::Idle | DroneTask::Return) {
        return None;
    }

    if let Some(plan) = find_airlock_candidate(
        endpoints,
        station_position,
        range,
        reservations,
        target_reservations,
    ) {
        return Some(plan);
    }

    if mode == DroneTask::Salvage {
        return None;
    }

    find_processor_candidate(
        endpoints,
        station_position,
        range,
        reservations,
        target_reservations,
    )
    .or_else(|| {
        find_processor_output_candidate(
            endpoints,
            station_position,
            range,
            reservations,
            target_reservations,
        )
    })
}

fn count_drone_transfer_candidates(
    endpoints: &[LogisticsEndpointSnapshot],
    station_position: FixedVec2,
    range: Fx,
    mode: DroneTask,
    reservations: &BTreeMap<(u64, ResourceKind), u32>,
    target_reservations: &BTreeMap<(u64, ResourceKind), u32>,
) -> u32 {
    let mut count = 0;
    let mut reservations = reservations.clone();
    let mut targets = target_reservations.clone();
    while let Some(plan) = plan_drone_transfer(
        endpoints,
        station_position,
        range,
        mode,
        &reservations,
        &targets,
    ) {
        *reservations
            .entry((plan.source_module_id, plan.resource_kind))
            .or_default() += 1;
        *targets
            .entry((plan.target_module_id, plan.resource_kind))
            .or_default() += 1;
        count += 1;
        if count >= 8 {
            break;
        }
    }
    count
}

fn find_airlock_candidate(
    endpoints: &[LogisticsEndpointSnapshot],
    station_position: FixedVec2,
    range: Fx,
    reservations: &BTreeMap<(u64, ResourceKind), u32>,
    target_reservations: &BTreeMap<(u64, ResourceKind), u32>,
) -> Option<DroneTransferPlan> {
    for source in endpoints {
        if source.destroyed
            || source.kind != ModuleKind::Airlock
            || local_field_distance(station_position, source.local_position) > range
        {
            continue;
        }
        let Some(storage) = source.storage else {
            continue;
        };
        for (resource_kind, amount) in [
            (ResourceKind::RawSalvage, storage.inventory.raw_salvage),
            (ResourceKind::RepairCharge, storage.inventory.repair_charge),
            (ResourceKind::Fuel, storage.inventory.fuel),
            (ResourceKind::Ammunition, storage.inventory.ammunition),
        ] {
            let reserved = reservations
                .get(&(source.module_id, resource_kind))
                .copied()
                .unwrap_or(0);
            if amount <= reserved {
                continue;
            }
            for target in endpoints {
                if target.destroyed
                    || target.module_id == source.module_id
                    || target.kind != ModuleKind::Cargo
                    || local_field_distance(station_position, target.local_position) > range
                {
                    continue;
                }
                let Some(target_storage) = target.storage else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => target_storage.accepts_fuel,
                    ResourceKind::Ammunition => target_storage.accepts_ammunition,
                    ResourceKind::Oxygen => target_storage.accepts_oxygen,
                    ResourceKind::RawSalvage | ResourceKind::RepairCharge => {
                        target_storage.accepts_general
                    }
                };
                let queued = target_reservations
                    .get(&(target.module_id, resource_kind))
                    .copied()
                    .unwrap_or(0);
                if accepts
                    && target_storage.inventory.total_units() + queued < target_storage.capacity
                {
                    return Some(DroneTransferPlan {
                        source_module_id: source.module_id,
                        target_module_id: target.module_id,
                        resource_kind,
                    });
                }
            }
        }
    }
    None
}

fn find_processor_candidate(
    endpoints: &[LogisticsEndpointSnapshot],
    station_position: FixedVec2,
    range: Fx,
    reservations: &BTreeMap<(u64, ResourceKind), u32>,
    target_reservations: &BTreeMap<(u64, ResourceKind), u32>,
) -> Option<DroneTransferPlan> {
    for source in endpoints {
        if source.destroyed || local_field_distance(station_position, source.local_position) > range
        {
            continue;
        }
        let Some(storage) = source.storage else {
            continue;
        };
        let reserved = reservations
            .get(&(source.module_id, ResourceKind::RawSalvage))
            .copied()
            .unwrap_or(0);
        if storage.inventory.raw_salvage <= reserved {
            continue;
        }
        for target in endpoints {
            if target.destroyed
                || target.kind != ModuleKind::Processor
                || target.module_id == source.module_id
                || local_field_distance(station_position, target.local_position) > range
            {
                continue;
            }
            let Some(processor) = target.processor else {
                continue;
            };
            let queued = target_reservations
                .get(&(target.module_id, ResourceKind::RawSalvage))
                .copied()
                .unwrap_or(0);
            if processor.inventory.raw_salvage + queued < processor.input_required * 2 {
                return Some(DroneTransferPlan {
                    source_module_id: source.module_id,
                    target_module_id: target.module_id,
                    resource_kind: ResourceKind::RawSalvage,
                });
            }
        }
    }
    None
}

fn find_processor_output_candidate(
    endpoints: &[LogisticsEndpointSnapshot],
    station_position: FixedVec2,
    range: Fx,
    reservations: &BTreeMap<(u64, ResourceKind), u32>,
    target_reservations: &BTreeMap<(u64, ResourceKind), u32>,
) -> Option<DroneTransferPlan> {
    for source in endpoints {
        if source.destroyed
            || source.kind != ModuleKind::Processor
            || local_field_distance(station_position, source.local_position) > range
        {
            continue;
        }
        let Some(processor) = source.processor else {
            continue;
        };
        for (resource_kind, amount) in [
            (
                ResourceKind::RepairCharge,
                processor.inventory.repair_charge,
            ),
            (ResourceKind::Fuel, processor.inventory.fuel),
            (ResourceKind::Ammunition, processor.inventory.ammunition),
        ] {
            let reserved = reservations
                .get(&(source.module_id, resource_kind))
                .copied()
                .unwrap_or(0);
            if amount <= reserved {
                continue;
            }
            for target in endpoints {
                if target.destroyed
                    || target.kind != ModuleKind::Cargo
                    || target.module_id == source.module_id
                    || local_field_distance(station_position, target.local_position) > range
                {
                    continue;
                }
                let Some(storage) = target.storage else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => storage.accepts_fuel,
                    ResourceKind::Ammunition => storage.accepts_ammunition,
                    ResourceKind::Oxygen => storage.accepts_oxygen,
                    ResourceKind::RawSalvage | ResourceKind::RepairCharge => {
                        storage.accepts_general
                    }
                };
                let queued = target_reservations
                    .get(&(target.module_id, resource_kind))
                    .copied()
                    .unwrap_or(0);
                if accepts && storage.inventory.total_units() + queued < storage.capacity {
                    return Some(DroneTransferPlan {
                        source_module_id: source.module_id,
                        target_module_id: target.module_id,
                        resource_kind,
                    });
                }
            }
        }
    }
    None
}

fn advance_drone_state(
    drone: &mut DroneUnit,
    endpoints: &[LogisticsEndpointSnapshot],
    logistics_mut_query: &mut Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    dt: Fx,
    mission_state: &mut MissionState,
) {
    let source_position = drone
        .source_module_id
        .and_then(|module_id| endpoint_by_module_id(endpoints, module_id))
        .map(|endpoint| endpoint.local_position);
    let target_position = drone
        .target_module_id
        .and_then(|module_id| endpoint_by_module_id(endpoints, module_id))
        .map(|endpoint| endpoint.local_position);

    match drone.phase {
        DronePhase::Idle => {
            drone.local_position =
                move_towards(drone.local_position, drone.home_local_position, dt, 90.0);
        }
        DronePhase::Assigned => {
            drone.phase = DronePhase::TravelingToSource;
        }
        DronePhase::TravelingToSource => {
            let Some(target) = source_position else {
                reset_drone(drone);
                return;
            };
            drone.local_position = move_towards(drone.local_position, target, dt, 120.0);
            if local_field_distance(drone.local_position, target) <= Fx::from_num(6.0) {
                drone.phase = DronePhase::PickingUp;
            }
        }
        DronePhase::PickingUp => {
            let Some(source_module_id) = drone.source_module_id else {
                reset_drone(drone);
                return;
            };
            let Some(resource_kind) = drone.resource_kind else {
                reset_drone(drone);
                return;
            };
            if transfer_out_of_source(logistics_mut_query, source_module_id, resource_kind, 1) > 0 {
                drone.payload_amount = 1;
                drone.reserved_amount = 0;
                drone.phase = DronePhase::TravelingToDestination;
            } else {
                reset_drone(drone);
            }
        }
        DronePhase::TravelingToDestination => {
            let Some(target) = target_position else {
                reset_drone(drone);
                return;
            };
            drone.local_position = move_towards(drone.local_position, target, dt, 120.0);
            if local_field_distance(drone.local_position, target) <= Fx::from_num(6.0) {
                drone.phase = DronePhase::Delivering;
            }
        }
        DronePhase::Delivering => {
            let Some(target_module_id) = drone.target_module_id else {
                reset_drone(drone);
                return;
            };
            let Some(resource_kind) = drone.resource_kind else {
                reset_drone(drone);
                return;
            };
            if try_transfer_into_target(
                logistics_mut_query,
                target_module_id,
                resource_kind,
                drone.payload_amount.max(1),
            ) {
                drone.payload_amount = 0;
                drone.phase = DronePhase::Returning;
                mission_state.transfer_count += 1;
                mission_state.logistics_automation_used = true;
                mission_state.recent_action = Some(format!(
                    "Drone delivered {}",
                    resource_kind_label(resource_kind)
                ));
                mission_state.recent_action_timer = Fx::from_num(1.2);
            }
        }
        DronePhase::Returning => {
            drone.local_position =
                move_towards(drone.local_position, drone.home_local_position, dt, 110.0);
            if local_field_distance(drone.local_position, drone.home_local_position)
                <= Fx::from_num(4.0)
            {
                reset_drone(drone);
            }
        }
    }
}

fn endpoint_by_module_id(
    endpoints: &[LogisticsEndpointSnapshot],
    module_id: u64,
) -> Option<LogisticsEndpointSnapshot> {
    endpoints
        .iter()
        .find(|endpoint| endpoint.module_id == module_id && !endpoint.destroyed)
        .copied()
}

fn move_towards(current: FixedVec2, target: FixedVec2, dt: Fx, speed: f32) -> FixedVec2 {
    let delta = target - current;
    let max_step = Fx::from_num(speed) * dt;
    if delta.length() <= max_step {
        target
    } else {
        current + delta.normalized_or_zero() * max_step
    }
}

fn transfer_out_of_source(
    logistics_mut_query: &mut Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    source_module_id: u64,
    resource_kind: ResourceKind,
    amount: u32,
) -> u32 {
    for (_, runtime_module, storage, processor, destroyed) in logistics_mut_query.iter_mut() {
        if destroyed.is_some() || runtime_module.module_id != source_module_id {
            continue;
        }
        if let Some(mut storage) = storage {
            return storage.inventory.remove(resource_kind, amount);
        }
        if let Some(mut processor) = processor {
            return processor.inventory.remove(resource_kind, amount);
        }
    }
    0
}

fn try_transfer_into_target(
    logistics_mut_query: &mut Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    target_module_id: u64,
    resource_kind: ResourceKind,
    amount: u32,
) -> bool {
    for (_, runtime_module, storage, processor, destroyed) in logistics_mut_query.iter_mut() {
        if destroyed.is_some() || runtime_module.module_id != target_module_id {
            continue;
        }
        if let Some(mut storage) = storage
            && storage.inventory.total_units() < storage.capacity
            && storage.accepts(resource_kind)
        {
            storage.inventory.add(resource_kind, amount);
            return true;
        }
        if let Some(mut processor) = processor {
            let limit = processor.input_required * 2 + processor.output_amount * 2;
            if processor.inventory.total_units() < limit {
                processor.inventory.add(resource_kind, amount);
                return true;
            }
        }
    }
    false
}

fn reset_drone(drone: &mut DroneUnit) {
    drone.phase = DronePhase::Idle;
    drone.source_module_id = None;
    drone.target_module_id = None;
    drone.resource_kind = None;
    drone.payload_amount = 0;
    drone.reserved_amount = 0;
}
