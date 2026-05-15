use bevy::{ecs::relationship::Relationship, prelude::*};

pub(crate) use super::drones::{run_drone_logistics, sync_drone_station_population};
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            CollectedSalvage,
            DestroyedModule,
            ManipulatorCommandState,
            ManipulatorModule,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            ProcessorCommandState,
            ProcessorModule,
            ProcessorRecipe,
            ResourceInventory,
            ResourceKind,
            RuntimeShipModule,
            SalvagePickup,
            SalvageWreck,
            ShipArchCommandState,
            ShipInfrastructureState,
            ShipRoot,
            SimPosition,
            StorageModule,
        },
        helpers::{FixedVec2, Fx, fx_from_time_delta, local_field_distance, resource_kind_label},
        systems::simulation::transfers::{
            find_airlock_to_cargo_transfer,
            find_automation_transfer_task,
        },
    },
    ship::ModuleKind,
};

#[derive(Clone, Copy)]
pub(crate) struct StorageSnapshot {
    pub(crate) capacity: u32,
    pub(crate) inventory: ResourceInventory,
    pub(crate) accepts_fuel: bool,
    pub(crate) accepts_ammunition: bool,
    pub(crate) accepts_general: bool,
    pub(crate) accepts_oxygen: bool,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct ProcessorSnapshot {
    pub(crate) input_required: u32,
    pub(crate) output_amount: u32,
    pub(crate) inventory: ResourceInventory,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct LogisticsEndpointSnapshot {
    pub(crate) entity: Entity,
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) local_position: FixedVec2,
    pub(crate) storage: Option<StorageSnapshot>,
    pub(crate) processor: Option<ProcessorSnapshot>,
    pub(crate) destroyed: bool,
}

/// Leaves salvage collection to boarding-cargo flow so shipboard recovery stays in one mechanic.
pub(crate) fn collect_salvage(
    _commands: Commands,
    _balance: Res<BalanceConfig>,
    _keys: Res<ButtonInput<KeyCode>>,
    _player_ship_query: Single<
        (&SimPosition, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    _storage_query: Query<(
        &RuntimeShipModule,
        &mut StorageModule,
        Option<&DestroyedModule>,
    )>,
    _salvage_query: Query<
        (Entity, &SimPosition, &SalvagePickup),
        (With<SalvageWreck>, Without<CollectedSalvage>),
    >,
) {
    // Salvage recovery is handled through the boarding-era carried-cargo loop.
}

/// Advances manipulator transfers so logistics-capable modules can move resources between endpoints.
pub(crate) fn run_logistics_transfers(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    ship_query: Single<
        (&ShipArchCommandState, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
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
    mut manipulator_query: Query<(
        &RuntimeShipModule,
        &ModuleRuntimeState,
        &mut ManipulatorModule,
        Option<&mut ManipulatorCommandState>,
        Option<&DestroyedModule>,
    )>,
) {
    // SAFETY: Endpoint discovery is read-only in `p0`, and actual inventory mutation happens later in `p1`;
    // the `ParamSet` branches are not held simultaneously, so storage/processor mutation cannot alias.
    let dt = fx_from_time_delta(&time);
    let (arch_commands, mut mission_state) = ship_query.into_inner();
    let logistics_mode = arch_commands.logistics_enabled;

    let snapshots: Vec<_> = {
        let snapshot_query = logistics_sets.p0();
        snapshot_query
            .iter()
            .map(|(entity, runtime_module, storage, processor, destroyed)| {
                (
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    runtime_module.local_position,
                    storage.map(|s| {
                        (
                            s.capacity,
                            s.inventory,
                            s.accepts_fuel,
                            s.accepts_ammunition,
                            s.accepts_general,
                            s.accepts_oxygen,
                        )
                    }),
                    processor.map(|p| (p.input_required, p.output_amount, p.inventory)),
                    destroyed.is_some(),
                )
            })
            .collect()
    };

    for (manip_runtime_module, manip_runtime_state, mut manipulator, command_state, destroyed) in
        &mut manipulator_query
    {
        if destroyed.is_some() || manip_runtime_state.is_disabled {
            manipulator.active = false;
            manipulator.blocked_reason = Some("manipulator offline".to_string());
            manipulator.transfer_progress = Fx::from_num(0);
            continue;
        }

        let in_range = |source_pos: FixedVec2| {
            local_field_distance(manip_runtime_module.local_position, source_pos)
                <= Fx::from_num(TILE_SIZE * balance.logistics.manipulator_range_tiles)
        };

        let mut task: Option<(u64, u64, ResourceKind)> = None;

        if let Some(command_state) = command_state.as_ref()
            && command_state.manual_mode
            && command_state.transfer_enabled
            && let (Some(source_module_id), Some(target_module_id)) = (
                command_state.source_module_id,
                command_state.target_module_id,
            )
        {
            task = Some((
                source_module_id,
                target_module_id,
                command_state.resource_kind,
            ));
        } else if logistics_mode {
            task = find_automation_transfer_task(
                &snapshots,
                &in_range,
                arch_commands.logistics_preference,
            );
        }

        if task.is_none() {
            task = find_airlock_to_cargo_transfer(&snapshots, &in_range);
        }

        let Some((source_module_id, target_module_id, resource_kind)) = task else {
            manipulator.active = false;
            manipulator.transfer_progress = Fx::from_num(0);
            manipulator.blocked_reason = Some("no valid transfer task".to_string());
            if logistics_mode {
                mission_state.logistics_automation_used = true;
            }
            continue;
        };

        manipulator.active = true;
        manipulator.source_module_id = Some(source_module_id);
        manipulator.target_module_id = Some(target_module_id);
        manipulator.resource_kind = Some(resource_kind);
        manipulator.blocked_reason = None;
        manipulator.transfer_progress += dt;

        if manipulator.transfer_progress < manipulator.transfer_duration {
            continue;
        }
        manipulator.transfer_progress = Fx::from_num(0);

        let mut moved = false;
        let source_entity = snapshots
            .iter()
            .find(|(_, module_id, _, _, _, _, _)| *module_id == source_module_id)
            .map(|(entity, _, _, _, _, _, _)| *entity);
        let target_entity = snapshots
            .iter()
            .find(|(_, module_id, _, _, _, _, _)| *module_id == target_module_id)
            .map(|(entity, _, _, _, _, _, _)| *entity);

        let (Some(source_entity), Some(target_entity)) = (source_entity, target_entity) else {
            manipulator.blocked_reason = Some("transfer endpoints missing".to_string());
            continue;
        };

        let mut logistics_query = logistics_sets.p1();
        let Ok(
            [
                (_, _, mut source_storage, mut source_processor, source_destroyed),
                (_, _, mut target_storage, mut target_processor, target_destroyed),
            ],
        ) = logistics_query.get_many_mut([source_entity, target_entity])
        else {
            manipulator.blocked_reason = Some("transfer endpoints unavailable".to_string());
            continue;
        };
        if source_destroyed.is_some() || target_destroyed.is_some() {
            manipulator.blocked_reason = Some("transfer endpoint destroyed".to_string());
            continue;
        }

        let mut source_taken = 0;
        if let Some(storage) = source_storage.as_mut() {
            source_taken = storage.inventory.remove(resource_kind, 1);
        } else if let Some(processor) = source_processor.as_mut() {
            source_taken = processor.inventory.remove(resource_kind, 1);
        }
        if source_taken == 0 {
            manipulator.blocked_reason = Some(format!(
                "source lacks {}",
                resource_kind_label(resource_kind)
            ));
            continue;
        }

        if let Some(storage) = target_storage.as_mut() {
            if storage.inventory.total_units() < storage.capacity && storage.accepts(resource_kind)
            {
                storage.inventory.add(resource_kind, 1);
                moved = true;
            }
        } else if let Some(processor) = target_processor.as_mut() {
            let processor_limit = processor.input_required * 2 + processor.output_amount * 2;
            if processor.inventory.total_units() < processor_limit {
                processor.inventory.add(resource_kind, 1);
                moved = true;
            }
        }

        if moved {
            mission_state.transfer_count += 1;
            mission_state.logistics_bottleneck = None;
            if logistics_mode {
                mission_state.logistics_automation_used = true;
            }
            if let Some(mut command_state) = command_state {
                command_state.transfer_enabled = false;
            }
        } else {
            if let Some(storage) = source_storage.as_mut() {
                storage.inventory.add(resource_kind, 1);
            } else if let Some(processor) = source_processor.as_mut() {
                processor.inventory.add(resource_kind, 1);
            }
            manipulator.blocked_reason = Some("target inventory full".to_string());
            mission_state.logistics_bottleneck = Some("target inventory full".to_string());
        }
    }
}

/// Ticks processor modules so fabrication recipes progress, stall, or complete based on inventory and power.
pub(crate) fn run_processors(
    time: Res<Time>,
    ship_query: Single<
        (&ShipInfrastructureState, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    mut processor_query: Query<(
        &RuntimeShipModule,
        &ChildOf,
        &ModuleRuntimeState,
        &mut ProcessorModule,
        Option<&ProcessorCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut storage_query: Query<(&RuntimeShipModule, &ChildOf, &mut StorageModule)>,
) {
    let dt = fx_from_time_delta(&time);
    let (infrastructure, mut mission_state) = ship_query.into_inner();

    for (runtime_module, parent, runtime_state, mut processor, command_state, destroyed) in
        &mut processor_query
    {
        if destroyed.is_some() || runtime_state.is_disabled {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("processor offline".to_string());
            continue;
        }
        if command_state.is_some_and(|command_state| !command_state.enabled) {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("manual hold".to_string());
            continue;
        }
        let output_kind = match command_state
            .map(|command_state| command_state.selected_recipe)
            .unwrap_or(ProcessorRecipe::RepairCharge)
        {
            ProcessorRecipe::RepairCharge => ResourceKind::RepairCharge,
            ProcessorRecipe::Ammunition => ResourceKind::Ammunition,
            ProcessorRecipe::Fuel => ResourceKind::Fuel,
        };
        let output_cap = processor.output_amount * 3;
        if processor.inventory.get(output_kind) >= output_cap {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("output buffer full".to_string());
            mission_state.logistics_bottleneck = Some("processor output blocked".to_string());
            continue;
        }
        while processor.inventory.raw_salvage < processor.input_required
            && pull_connected_storage_resource(
                infrastructure,
                &mut storage_query,
                parent.get(),
                runtime_module.module_id,
                ResourceKind::RawSalvage,
            ) > 0
        {
            processor.inventory.add(ResourceKind::RawSalvage, 1);
        }
        if processor.inventory.raw_salvage < processor.input_required {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some(
                if infrastructure
                    .module_resource_network(runtime_module.module_id, ResourceKind::RawSalvage)
                    .is_some()
                {
                    "waiting for raw salvage".to_string()
                } else {
                    "no compatible resource".to_string()
                },
            );
            continue;
        }
        if !infrastructure.module_powered(runtime_module.module_id) {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("no wired power".to_string());
            mission_state.logistics_bottleneck = Some("processor starved for power".to_string());
            continue;
        }

        processor.active = true;
        processor.blocked_reason = None;
        processor.progress += dt;
        if processor.progress < processor.duration {
            continue;
        }

        processor.progress = Fx::from_num(0);
        let input_required = processor.input_required;
        let output_amount = processor.output_amount;
        let consumed = processor
            .inventory
            .remove(ResourceKind::RawSalvage, input_required);
        if consumed < input_required {
            processor.blocked_reason = Some("input lost before cycle complete".to_string());
            continue;
        }
        let mut pushed = 0;
        for _ in 0..output_amount {
            pushed += push_connected_storage_resource(
                infrastructure,
                &mut storage_query,
                parent.get(),
                runtime_module.module_id,
                output_kind,
            );
        }
        if pushed < output_amount {
            processor.inventory.add(output_kind, output_amount - pushed);
        }
        if output_kind == ResourceKind::RepairCharge {
            mission_state.processed_repair_charge += output_amount;
        }
        mission_state.processor_cycles += 1;
        mission_state.recent_action = Some(format!(
            "{} fabricated {}",
            runtime_module.kind.as_str(),
            resource_kind_label(output_kind)
        ));
        mission_state.recent_action_timer = Fx::from_num(1.8);
    }
}

fn pull_connected_storage_resource(
    infrastructure: &ShipInfrastructureState,
    storage_query: &mut Query<(&RuntimeShipModule, &ChildOf, &mut StorageModule)>,
    ship_entity: Entity,
    module_id: u64,
    resource: ResourceKind,
) -> u32 {
    let Some(network_id) = infrastructure.module_resource_network(module_id, resource) else {
        return 0;
    };
    for (storage_runtime, storage_parent, mut storage) in storage_query {
        if storage_parent.get() != ship_entity || !storage.accepts(resource) {
            continue;
        }
        if infrastructure.module_resource_network(storage_runtime.module_id, resource)
            != Some(network_id)
        {
            continue;
        }
        let taken = storage.inventory.remove(resource, 1);
        if taken > 0 {
            return taken;
        }
    }
    0
}

fn push_connected_storage_resource(
    infrastructure: &ShipInfrastructureState,
    storage_query: &mut Query<(&RuntimeShipModule, &ChildOf, &mut StorageModule)>,
    ship_entity: Entity,
    module_id: u64,
    resource: ResourceKind,
) -> u32 {
    let Some(network_id) = infrastructure.module_resource_network(module_id, resource) else {
        return 0;
    };
    for (storage_runtime, storage_parent, mut storage) in storage_query {
        if storage_parent.get() != ship_entity || !storage.accepts(resource) {
            continue;
        }
        if infrastructure.module_resource_network(storage_runtime.module_id, resource)
            != Some(network_id)
        {
            continue;
        }
        if storage.inventory.total_units() >= storage.capacity {
            continue;
        }
        storage.inventory.add(resource, 1);
        return 1;
    }
    0
}

pub(crate) fn collect_endpoint_snapshots(
    logistics_query: &Query<(
        Entity,
        &RuntimeShipModule,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
) -> Vec<LogisticsEndpointSnapshot> {
    let mut endpoints: Vec<_> = logistics_query
        .iter()
        .map(
            |(entity, runtime_module, storage, processor, destroyed)| LogisticsEndpointSnapshot {
                entity,
                module_id: runtime_module.module_id,
                kind: runtime_module.kind,
                local_position: runtime_module.local_position,
                storage: storage.map(|storage| StorageSnapshot {
                    capacity: storage.capacity,
                    inventory: storage.inventory,
                    accepts_fuel: storage.accepts_fuel,
                    accepts_ammunition: storage.accepts_ammunition,
                    accepts_general: storage.accepts_general,
                    accepts_oxygen: storage.accepts_oxygen,
                }),
                processor: processor.map(|processor| ProcessorSnapshot {
                    input_required: processor.input_required,
                    output_amount: processor.output_amount,
                    inventory: processor.inventory,
                }),
                destroyed: destroyed.is_some(),
            },
        )
        .collect();
    endpoints.sort_by_key(|endpoint| endpoint.module_id);
    endpoints
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::gameplay::{
        components::{DroneTask, ResourceInventory},
        systems::simulation::drones::plan_drone_transfer,
    };

    fn endpoint(
        module_id: u64,
        kind: ModuleKind,
        x: f32,
        storage: Option<StorageSnapshot>,
        processor: Option<ProcessorSnapshot>,
    ) -> LogisticsEndpointSnapshot {
        LogisticsEndpointSnapshot {
            entity: Entity::PLACEHOLDER,
            module_id,
            kind,
            local_position: FixedVec2::from_num(x, 0.0),
            storage,
            processor,
            destroyed: false,
        }
    }

    #[test]
    fn drone_planner_respects_existing_reservations() {
        let endpoints = vec![
            endpoint(
                1,
                ModuleKind::Airlock,
                0.0,
                Some(StorageSnapshot {
                    capacity: 6,
                    inventory: ResourceInventory {
                        raw_salvage: 1,
                        ..Default::default()
                    },
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: true,
                    accepts_oxygen: false,
                }),
                None,
            ),
            endpoint(
                2,
                ModuleKind::Cargo,
                64.0,
                Some(StorageSnapshot {
                    capacity: 6,
                    inventory: ResourceInventory::default(),
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: true,
                    accepts_oxygen: false,
                }),
                None,
            ),
        ];

        let mut reservations = BTreeMap::new();
        reservations.insert((1, ResourceKind::RawSalvage), 1);
        let plan = plan_drone_transfer(
            &endpoints,
            FixedVec2::from_num(0.0, 0.0),
            Fx::from_num(128.0),
            DroneTask::Salvage,
            &reservations,
            &BTreeMap::new(),
        );
        assert!(plan.is_none());
    }

    #[test]
    fn drone_planner_prefers_airlock_then_processor_chain_deterministically() {
        let endpoints = vec![
            endpoint(
                1,
                ModuleKind::Airlock,
                0.0,
                Some(StorageSnapshot {
                    capacity: 6,
                    inventory: ResourceInventory {
                        raw_salvage: 2,
                        ..Default::default()
                    },
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: true,
                    accepts_oxygen: false,
                }),
                None,
            ),
            endpoint(
                2,
                ModuleKind::Cargo,
                64.0,
                Some(StorageSnapshot {
                    capacity: 6,
                    inventory: ResourceInventory::default(),
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: true,
                    accepts_oxygen: false,
                }),
                None,
            ),
            endpoint(
                3,
                ModuleKind::Processor,
                96.0,
                None,
                Some(ProcessorSnapshot {
                    input_required: 2,
                    output_amount: 1,
                    inventory: ResourceInventory::default(),
                }),
            ),
        ];

        let plan = plan_drone_transfer(
            &endpoints,
            FixedVec2::from_num(0.0, 0.0),
            Fx::from_num(160.0),
            DroneTask::Logistics,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(plan.source_module_id, 1);
        assert_eq!(plan.target_module_id, 2);
        assert_eq!(plan.resource_kind, ResourceKind::RawSalvage);
    }
}
