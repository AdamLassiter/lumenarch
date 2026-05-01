use bevy::prelude::*;

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
            ResourceKind,
            RuntimeShipModule,
            SalvagePickup,
            SalvageWreck,
            ShipArchCommandState,
            ShipPowerState,
            ShipRoot,
            SimPosition,
            StorageModule,
        },
        helpers::{FixedVec2, Fx, fx_from_time_delta, local_field_distance, resource_kind_label},
    },
    ship::ModuleKind,
};

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

pub(crate) fn run_processors(
    time: Res<Time>,
    ship_query: Single<(&ShipPowerState, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    mut processor_query: Query<(
        &RuntimeShipModule,
        &ModuleRuntimeState,
        &mut ProcessorModule,
        Option<&ProcessorCommandState>,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);
    let (power_state, mut mission_state) = ship_query.into_inner();

    for (runtime_module, runtime_state, mut processor, command_state, destroyed) in
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
        if processor.inventory.raw_salvage < processor.input_required {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("waiting for raw salvage".to_string());
            continue;
        }
        if power_state.surplus <= Fx::from_num(0) && power_state.stored_energy <= Fx::from_num(1) {
            processor.active = false;
            processor.progress = Fx::from_num(0);
            processor.blocked_reason = Some("insufficient power".to_string());
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
        processor.inventory.add(output_kind, output_amount);
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

fn find_automation_transfer_task(
    snapshots: &[(
        Entity,
        u64,
        ModuleKind,
        FixedVec2,
        Option<(
            u32,
            crate::gameplay::components::ResourceInventory,
            bool,
            bool,
            bool,
        )>,
        Option<(u32, u32, crate::gameplay::components::ResourceInventory)>,
        bool,
    )],
    in_range: &impl Fn(FixedVec2) -> bool,
    preference: crate::gameplay::components::ArchLogisticsPreference,
) -> Option<(u64, u64, ResourceKind)> {
    for (_, source_id, _, source_pos, storage, _, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, source_inventory, _, _, _)) = storage else {
            continue;
        };
        if source_inventory.raw_salvage == 0 {
            continue;
        }
        for (_, target_id, target_kind, target_pos, _, processor, target_destroyed) in snapshots {
            if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                continue;
            }
            let Some((input_required, _, processor_inventory)) = processor else {
                continue;
            };
            if *target_kind != ModuleKind::Processor
                || processor_inventory.raw_salvage >= input_required * 2
            {
                continue;
            }
            return Some((*source_id, *target_id, ResourceKind::RawSalvage));
        }
    }

    if matches!(
        preference,
        crate::gameplay::components::ArchLogisticsPreference::FeedProcessor
    ) {
        return None;
    }

    for (_, source_id, source_kind, source_pos, _, processor, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, _, processor_inventory)) = processor else {
            continue;
        };
        if *source_kind != ModuleKind::Processor || processor_inventory.repair_charge == 0 {
            continue;
        }
        for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
            if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                continue;
            }
            let Some((capacity, storage_inventory, _, _, accepts_general)) = storage else {
                continue;
            };
            if *target_kind != ModuleKind::Cargo
                || !accepts_general
                || storage_inventory.total_units() >= *capacity
            {
                continue;
            }
            return Some((*source_id, *target_id, ResourceKind::RepairCharge));
        }
    }

    for (_, source_id, source_kind, source_pos, _, processor, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, _, processor_inventory)) = processor else {
            continue;
        };
        let outputs = [
            (ResourceKind::Fuel, processor_inventory.fuel),
            (ResourceKind::Ammunition, processor_inventory.ammunition),
        ];
        for (resource_kind, amount) in outputs {
            if amount == 0 {
                continue;
            }
            for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
                if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                    continue;
                }
                let Some((capacity, storage_inventory, accepts_fuel, accepts_ammunition, _)) =
                    storage
                else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => *accepts_fuel,
                    ResourceKind::Ammunition => *accepts_ammunition,
                    _ => false,
                };
                if !accepts
                    || *target_kind != ModuleKind::Cargo
                    || storage_inventory.total_units() >= *capacity
                {
                    continue;
                }
                return Some((*source_id, *target_id, resource_kind));
            }
        }
        let _ = source_kind;
    }

    None
}

fn find_airlock_to_cargo_transfer(
    snapshots: &[(
        Entity,
        u64,
        ModuleKind,
        FixedVec2,
        Option<(
            u32,
            crate::gameplay::components::ResourceInventory,
            bool,
            bool,
            bool,
        )>,
        Option<(u32, u32, crate::gameplay::components::ResourceInventory)>,
        bool,
    )],
    in_range: &impl Fn(FixedVec2) -> bool,
) -> Option<(u64, u64, ResourceKind)> {
    for (_, source_id, source_kind, source_pos, storage, _, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, source_inventory, _, _, _)) = storage else {
            continue;
        };
        if *source_kind != ModuleKind::Airlock {
            continue;
        }
        for (resource_kind, amount) in [
            (ResourceKind::RawSalvage, source_inventory.raw_salvage),
            (ResourceKind::RepairCharge, source_inventory.repair_charge),
            (ResourceKind::Fuel, source_inventory.fuel),
            (ResourceKind::Ammunition, source_inventory.ammunition),
        ] {
            if amount == 0 {
                continue;
            }
            for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
                if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                    continue;
                }
                let Some((
                    capacity,
                    storage_inventory,
                    accepts_fuel,
                    accepts_ammunition,
                    accepts_general,
                )) = storage
                else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => *accepts_fuel,
                    ResourceKind::Ammunition => *accepts_ammunition,
                    ResourceKind::RawSalvage | ResourceKind::RepairCharge => *accepts_general,
                };
                if *target_kind != ModuleKind::Cargo
                    || !accepts
                    || storage_inventory.total_units() >= *capacity
                {
                    continue;
                }
                return Some((*source_id, *target_id, resource_kind));
            }
        }
    }

    None
}
