use super::*;

pub(crate) fn run_arch_automation(
    time: Res<Time>,
    ship_query: Single<
        (
            &Children,
            &ShipPowerState,
            &ShipWeaponState,
            &ShipDamageSensorState,
            &ShipInfrastructureState,
            &mut ShipArchCommandState,
            &mut ShipAutomationState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    hostile_query: Query<Entity, With<HostileTarget>>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut blocker_queries: ParamSet<(
        Query<(
            Entity,
            &RuntimeShipModule,
            Option<&JunctionCommandState>,
            Option<&ValveCommandState>,
            Option<&DestroyedModule>,
        )>,
        Query<(
            &RuntimeShipModule,
            Option<&mut JunctionCommandState>,
            Option<&mut ValveCommandState>,
            Option<&DestroyedModule>,
        )>,
    )>,
) {
    // SAFETY: Blocker readback uses `p0` to build ARCH snapshots, then command application uses `p1`;
    // the immutable blocker query is dropped before mutable command state access, avoiding aliased borrows.
    let dt = fx_from_time_delta(&time);
    let (
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        infrastructure_state,
        mut command_state,
        mut automation_state,
        mut mission_state,
    ) = ship_query.into_inner();

    *command_state = ShipArchCommandState::default();
    automation_state.active = false;
    automation_state.output_scale = Fx::from_num(1);
    automation_state.invalid_executions = 0;
    automation_state.last_primary_program = None;
    automation_state.last_secondary_program = None;
    automation_state.recent_writes.clear();

    let snapshot = build_snapshot(
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        infrastructure_state,
        !hostile_query.is_empty(),
        &module_query,
        &blocker_queries.p0(),
    );
    let lumen_snapshot = build_lumen_snapshot(
        children,
        ship_power_state,
        damage_state,
        !hostile_query.is_empty(),
        &module_query,
    );
    let mut aggregate = PendingArchCommands::default();
    let mut had_active_computer = false;
    let mut trigger_delta = 0u32;

    for child in children.iter() {
        let Ok((
            _,
            runtime_module,
            mut runtime_state,
            computer_marker,
            arch_runtime,
            _storage,
            _processor,
            _reactor,
            _detector,
            _turret,
            destroyed,
        )) = module_query.get_mut(child)
        else {
            continue;
        };
        let Some(_) = computer_marker else {
            continue;
        };
        let Some(mut arch_runtime) = arch_runtime else {
            continue;
        };
        if destroyed.is_some() || runtime_state.is_disabled || !arch_runtime.enabled {
            arch_runtime.last_result = ArchExecutionResult {
                executed: 0,
                budget: arch_runtime.instruction_budget,
                recent_writes: Vec::new(),
                halted_reason: Some("offline".to_string()),
                program_name: arch_runtime.program.name.clone(),
            };
            continue;
        }

        had_active_computer = true;
        if automation_state.last_primary_program.is_none() {
            automation_state.last_primary_program = Some(arch_runtime.program.name.clone());
            automation_state.mode = template_to_mode(arch_runtime.program.template);
        } else if automation_state.mode != template_to_mode(arch_runtime.program.template) {
            automation_state.mode = ShipAutomationMode::Mixed;
        }

        let was_active = aggregate.reactor_bias > Fx::from_num(0)
            || aggregate.logistics_enabled
            || aggregate.turret_assist
            || aggregate.turret_auto_fire;
        let (result, outputs) = execute_program(
            &arch_runtime.program,
            arch_runtime.instruction_budget,
            snapshot,
        );
        let now_active = outputs.reactor_bias > Fx::from_num(0)
            || outputs.logistics_enabled
            || outputs.turret_assist
            || outputs.turret_auto_fire;
        if now_active && !was_active {
            trigger_delta += 1;
        }
        aggregate.reactor_bias = aggregate.reactor_bias.max(outputs.reactor_bias);
        aggregate.logistics_enabled |= outputs.logistics_enabled;
        if outputs.logistics_enabled {
            aggregate.logistics_preference = outputs.logistics_preference;
        }
        aggregate.turret_assist |= outputs.turret_assist;
        aggregate.turret_auto_fire |= outputs.turret_auto_fire;
        if outputs.junction_open.is_some() {
            aggregate.junction_open = outputs.junction_open;
        }
        if outputs.valve_open.is_some() {
            aggregate.valve_open = outputs.valve_open;
        }
        if result.halted_reason.is_some() {
            automation_state.invalid_executions += 1;
        }
        if automation_state.recent_writes.is_empty() && !result.recent_writes.is_empty() {
            automation_state.recent_writes = result.recent_writes.clone();
        }
        arch_runtime.last_result = result;

        if runtime_module.kind == ModuleKind::Computer && aggregate.reactor_bias > Fx::from_num(0) {
            runtime_state.current_heat =
                (runtime_state.current_heat - Fx::from_num(0.4) * dt).max(Fx::from_num(0));
        }

        if arch_runtime.lumen_program.enabled {
            let (lumen_result, lumen_outputs) =
                execute_lumen_program(&arch_runtime.lumen_program, lumen_snapshot);
            automation_state.last_secondary_program = Some(arch_runtime.lumen_program.name.clone());
            aggregate.reactor_bias += lumen_outputs.reactor_bias;
            aggregate.logistics_enabled |= lumen_outputs.logistics_enabled;
            if lumen_outputs.logistics_enabled {
                aggregate.logistics_preference = lumen_outputs.logistics_preference;
            }
            aggregate.turret_assist |= lumen_outputs.turret_assist;
            aggregate.turret_auto_fire |= lumen_outputs.turret_auto_fire;
            aggregate.turret_fire_hold |= lumen_outputs.turret_fire_hold;
            if automation_state.recent_writes.len() < 4 && !lumen_result.recent_effects.is_empty() {
                automation_state
                    .recent_writes
                    .extend(lumen_result.recent_effects.iter().take(2).cloned());
            }
            if lumen_result.halted_reason.is_some() {
                automation_state.invalid_executions += 1;
            }
            arch_runtime.last_lumen_result = lumen_result;
        } else {
            arch_runtime.last_lumen_result = LumenExecutionResult {
                resolved_targets: 0,
                recent_effects: Vec::new(),
                halted_reason: Some("offline".to_string()),
                program_name: arch_runtime.lumen_program.name.clone(),
            };
        }
    }

    if !had_active_computer {
        automation_state.mode = ShipAutomationMode::Off;
        return;
    }

    command_state.reactor_cooling_bias = aggregate.reactor_bias;
    command_state.logistics_enabled = aggregate.logistics_enabled;
    command_state.logistics_preference = aggregate.logistics_preference;
    command_state.turret_assist_enabled = aggregate.turret_assist;
    command_state.turret_auto_fire = aggregate.turret_auto_fire;
    command_state.turret_fire_hold = aggregate.turret_fire_hold;
    command_state.junction_open = aggregate.junction_open;
    command_state.valve_open = aggregate.valve_open;

    apply_blocker_arch_commands(
        aggregate.junction_open,
        aggregate.valve_open,
        &mut blocker_queries.p1(),
    );

    automation_state.active = aggregate.reactor_bias > Fx::from_num(0)
        || aggregate.logistics_enabled
        || aggregate.turret_assist
        || aggregate.turret_auto_fire
        || aggregate.turret_fire_hold
        || aggregate.junction_open.is_some()
        || aggregate.valve_open.is_some();
    if aggregate.reactor_bias > Fx::from_num(0) {
        automation_state.output_scale = Fx::from_num(0.72);
    }
    if automation_state.active {
        automation_state.trigger_count += trigger_delta.max(1);
        mission_state.automation_used = true;
        mission_state.automation_trigger_count = automation_state.trigger_count;
        mission_state.logistics_automation_used |= aggregate.logistics_enabled;
        mission_state.recent_action = Some(format!(
            "Automation active: {}{}",
            automation_state
                .last_primary_program
                .as_deref()
                .unwrap_or("program"),
            automation_state
                .last_secondary_program
                .as_deref()
                .map(|name| format!(" + {name}"))
                .unwrap_or_default()
        ));
        mission_state.recent_action_timer = Fx::from_num(1.4);
    }

    if aggregate.reactor_bias > Fx::from_num(0) {
        for child in children.iter() {
            let Ok((_, runtime_module, mut runtime_state, _, _, _, _, _, _, _, destroyed)) =
                module_query.get_mut(child)
            else {
                continue;
            };
            if destroyed.is_some() || runtime_module.kind != ModuleKind::Reactor {
                continue;
            }
            runtime_state.current_heat = (runtime_state.current_heat
                - aggregate.reactor_bias * Fx::from_num(0.75) * dt)
                .max(Fx::from_num(0));
            runtime_state.electrical_instability = (runtime_state.electrical_instability
                - aggregate.reactor_bias * Fx::from_num(0.9) * dt)
                .max(Fx::from_num(0));
            if runtime_state.current_heat <= Fx::from_num(8)
                && runtime_state.electrical_instability <= Fx::from_num(6)
            {
                runtime_state.is_disabled = false;
                runtime_state.needs_attention = false;
            }
        }
    }
}

pub(crate) fn build_lumen_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    _damage_state: &ShipDamageSensorState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
) -> LumenSnapshot {
    snapshots::build_lumen_snapshot(
        children,
        ship_power_state,
        _damage_state,
        threat_present,
        module_query,
    )
}

/// Fades recent automation action feedback so command HUD messages feel timely without lingering forever.
pub(crate) fn tick_recent_action_feedback(
    time: Res<Time>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    let mut mission_state = mission_query.into_inner();
    if mission_state.recent_action_timer <= Fx::from_num(0) {
        return;
    }
    mission_state.recent_action_timer =
        (mission_state.recent_action_timer - fx_from_time_delta(&time)).max(Fx::from_num(0));
    if mission_state.recent_action_timer == Fx::from_num(0) {
        mission_state.recent_action = None;
    }
}

pub(crate) fn build_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    ship_weapon_state: &ShipWeaponState,
    damage_state: &ShipDamageSensorState,
    infrastructure_state: &ShipInfrastructureState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    blocker_query: &Query<(
        Entity,
        &RuntimeShipModule,
        Option<&JunctionCommandState>,
        Option<&ValveCommandState>,
        Option<&DestroyedModule>,
    )>,
) -> ArchSnapshot {
    snapshots::build_snapshot(
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        infrastructure_state,
        threat_present,
        module_query,
        blocker_query,
    )
}
