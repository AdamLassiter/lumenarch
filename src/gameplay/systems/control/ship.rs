use super::*;
use crate::{
    gameplay::components::{
        DroneStationCommandState,
        DroneTask,
        PlayerConditionState,
        ProcessorRecipe,
        ResourceKind,
    },
    netcode::{INPUT_CYCLE_TEMPLATE, INPUT_SPACE_EDGE, StationControlOp},
    ship::arch::ArchProgram,
};

/// Converts cockpit, turret, reactor, and station inputs into module-level command state changes.
pub(crate) fn update_station_command_input(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    ship_query: Single<(&SimPosition, &SimRotation), (With<PlayerShip>, With<ShipRoot>)>,
    player_control_query: Query<(
        &PlayerHandleComponent,
        &ShipboardControlState,
        &PlayerConditionState,
    )>,
    ship_control_query: Single<&mut ShipControlState, (With<PlayerShip>, With<ShipRoot>)>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut TurretCommandState>,
        Option<&mut ReactorCommandState>,
        Option<&mut StorageCommandState>,
        Option<&mut ManipulatorCommandState>,
        Option<&mut ProcessorCommandState>,
        Option<&mut AirlockCommandState>,
        Option<&mut JunctionCommandState>,
        Option<&mut ValveCommandState>,
        Option<&mut DroneStationCommandState>,
        Option<&mut RuntimeArchComputer>,
    )>,
    candidate_query: Query<&RuntimeShipModule>,
) {
    let dt = fx_from_time_delta(&time);
    let (_, ship_rotation) = ship_query.into_inner();
    let mut ship_controls = ship_control_query.into_inner();
    let mut mission_runtime = mission_query.into_inner();
    let previous_throttle = ship_controls.throttle_demand;
    let mut control_states: Vec<_> = player_control_query.iter().collect();
    control_states.sort_by_key(|(handle, _, _)| handle.handle);
    let mut claimed_entities = BTreeSet::new();

    ship_controls.fire_pressed = false;
    ship_controls.throttle_demand = Fx::from_num(0);
    ship_controls.turn_input = Fx::from_num(0);
    if mission_runtime.failed || mission_runtime.completed {
        return;
    }

    for (handle, control_state, condition) in control_states {
        if condition.control_disabled() {
            continue;
        }
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        match control_state.mode {
            ShipControlMode::Interior => {}
            ShipControlMode::Cockpit => {
                ship_controls.throttle_demand = (Fx::from_num(command.throttle_milli)
                    / Fx::from_num(1000))
                .clamp(Fx::from_num(0), Fx::from_num(1));
                ship_controls.turn_input = (Fx::from_num(command.turn_milli) / Fx::from_num(1000))
                    .clamp(Fx::from_num(-1), Fx::from_num(1));
                if command.raw.pressed(netcode::INPUT_FIRE) {
                    ship_controls.turn_input =
                        Fx::from_num(-command.aim_x_milli) / Fx::from_num(1000);
                    ship_controls.throttle_demand = (((Fx::from_num(command.aim_y_milli)
                        / Fx::from_num(1000))
                        + Fx::from_num(1))
                        / Fx::from_num(2))
                    .clamp(Fx::from_num(0), Fx::from_num(1));
                }
                match command.station.op {
                    StationControlOp::HelmThrottle => {
                        ship_controls.throttle_demand = (previous_throttle
                            + Fx::from_num(command.station.arg0) / Fx::from_num(1000))
                        .clamp(Fx::from_num(0), Fx::from_num(1));
                    }
                    StationControlOp::HelmTurn => {
                        ship_controls.turn_input = (Fx::from_num(command.station.arg0)
                            / Fx::from_num(1000))
                        .clamp(Fx::from_num(-1), Fx::from_num(1));
                    }
                    _ => {}
                }
            }
            ShipControlMode::Turret => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let Ok((_, _, turret_state, _, _, _, _, _, _, _, _, _)) =
                    module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                let Some(mut turret_state) = turret_state else {
                    continue;
                };
                let aim =
                    FixedVec2::from_num(command.aim_x_milli as i32, command.aim_y_milli as i32);
                if !aim.is_near_zero() {
                    turret_state.desired_angle = wrap_radians(
                        angle_from_vector(aim) - Fx::FRAC_PI_2 - ship_rotation.radians,
                    );
                }
                turret_state.desired_angle += dt * Fx::from_num(command.turn_milli)
                    / Fx::from_num(1000)
                    * Fx::from_num(balance.combat.turret_manual_aim_speed);
                if command.station.op == StationControlOp::TurretAdjustAim {
                    turret_state.desired_angle +=
                        Fx::from_num(command.station.arg0) / Fx::from_num(1000);
                }
                turret_state.desired_angle = wrap_radians(turret_state.desired_angle);
                if command.station.op == StationControlOp::TurretFireToggle {
                    turret_state.fire_intent = !turret_state.fire_intent;
                } else {
                    turret_state.fire_intent = command.raw.pressed(netcode::INPUT_FIRE);
                }
                ship_controls.fire_pressed |= turret_state.fire_intent;
            }
            ShipControlMode::Reactor => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let Ok((_, _, _, reactor_state, _, _, _, _, _, _, _, _)) =
                    module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                let Some(mut reactor_state) = reactor_state else {
                    continue;
                };
                reactor_state.reaction_rate += dt
                    * Fx::from_num(balance.reactor.control_adjust_rate)
                    * Fx::from_num(command.reactor_delta_milli)
                    / Fx::from_num(1000);
                reactor_state.turbine_load += dt
                    * Fx::from_num(balance.reactor.control_adjust_rate)
                    * Fx::from_num(command.turbine_delta_milli)
                    / Fx::from_num(1000);
                reactor_state.reaction_rate = reactor_state
                    .reaction_rate
                    .clamp(Fx::from_num(0), Fx::from_num(1));
                reactor_state.turbine_load = reactor_state
                    .turbine_load
                    .clamp(Fx::from_num(0), Fx::from_num(1));
                match command.station.op {
                    StationControlOp::ReactorAdjustRate => {
                        reactor_state.reaction_rate = (reactor_state.reaction_rate
                            + Fx::from_num(command.station.arg0) / Fx::from_num(1000))
                        .clamp(Fx::from_num(0), Fx::from_num(1));
                    }
                    StationControlOp::ReactorAdjustTurbine => {
                        reactor_state.turbine_load = (reactor_state.turbine_load
                            + Fx::from_num(command.station.arg0) / Fx::from_num(1000))
                        .clamp(Fx::from_num(0), Fx::from_num(1));
                    }
                    _ => {}
                }
            }
            ShipControlMode::Logistics => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let candidate_ids = helpers::nearby_logistics_target_ids(
                    control_state.focused_module_id.unwrap_or_default(),
                    &candidate_query,
                );
                let Ok((
                    _,
                    runtime_module,
                    _,
                    _,
                    storage_cmd,
                    manipulator_cmd,
                    processor_cmd,
                    airlock_state,
                    junction_cmd,
                    valve_cmd,
                    drone_station_cmd,
                    _,
                )) = module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                if let Some(mut airlock_state) = airlock_state {
                    if command.raw.pressed(INPUT_SPACE_EDGE)
                        || command.station.op == StationControlOp::LogisticsToggleAirlock
                    {
                        airlock_state.open = !airlock_state.open;
                        mission_runtime.airlocks_cycled += 1;
                    }
                } else if let Some(mut storage_cmd) = storage_cmd
                    && (command.raw.pressed(INPUT_SPACE_EDGE)
                        || command.station.op == StationControlOp::LogisticsToggleStorageIntake)
                {
                    storage_cmd.allow_intake = !storage_cmd.allow_intake;
                }
                if let Some(mut manipulator_cmd) = manipulator_cmd {
                    if command.raw.pressed(netcode::INPUT_AUX_EDGE) {
                        manipulator_cmd.manual_mode = !manipulator_cmd.manual_mode;
                    }
                    if command.reactor_delta_milli != 0
                        || command.station.op == StationControlOp::LogisticsCycleResource
                    {
                        manipulator_cmd.resource_kind = match manipulator_cmd.resource_kind {
                            ResourceKind::RawSalvage => ResourceKind::RepairCharge,
                            ResourceKind::RepairCharge => ResourceKind::Fuel,
                            ResourceKind::Fuel => ResourceKind::Ammunition,
                            ResourceKind::Ammunition => ResourceKind::Oxygen,
                            ResourceKind::Oxygen => ResourceKind::RawSalvage,
                        };
                    }
                    if command.raw.pressed(INPUT_SPACE_EDGE)
                        || command.station.op == StationControlOp::LogisticsToggleManipulator
                    {
                        manipulator_cmd.transfer_enabled = !manipulator_cmd.transfer_enabled;
                    }
                    if !candidate_ids.is_empty()
                        && (command.raw.pressed(netcode::INPUT_PREV_EDGE)
                            || command.raw.pressed(netcode::INPUT_NEXT_EDGE)
                            || command.station.op
                                == StationControlOp::LogisticsCycleManipulatorTarget)
                    {
                        let direction = if command.station.op
                            == StationControlOp::LogisticsCycleManipulatorTarget
                        {
                            command.station.arg0 as i32
                        } else if command.raw.pressed(netcode::INPUT_PREV_EDGE) {
                            -1
                        } else {
                            1
                        };
                        let current_index = manipulator_cmd
                            .target_module_id
                            .and_then(|module_id| {
                                candidate_ids.iter().position(|id| *id == module_id)
                            })
                            .unwrap_or(0);
                        let next_index = ((current_index as i32 + direction)
                            .rem_euclid(candidate_ids.len() as i32))
                            as usize;
                        manipulator_cmd.target_module_id = Some(candidate_ids[next_index]);
                        manipulator_cmd.source_module_id = Some(runtime_module.module_id);
                    }
                }
                if let Some(mut drone_station_cmd) = drone_station_cmd
                    && (command.raw.pressed(netcode::INPUT_AUX_EDGE)
                        || command.station.op == StationControlOp::LogisticsToggleProcessor)
                {
                    drone_station_cmd.selected_task = match drone_station_cmd.selected_task {
                        DroneTask::Idle => DroneTask::Salvage,
                        DroneTask::Salvage => DroneTask::Logistics,
                        DroneTask::Logistics => DroneTask::Return,
                        DroneTask::Return => DroneTask::Idle,
                    };
                }
                if let Some(mut processor_cmd) = processor_cmd {
                    if command.raw.pressed(INPUT_SPACE_EDGE)
                        || command.station.op == StationControlOp::LogisticsToggleProcessor
                    {
                        processor_cmd.enabled = !processor_cmd.enabled;
                    }
                    if command.reactor_delta_milli != 0 {
                        processor_cmd.selected_recipe = match processor_cmd.selected_recipe {
                            ProcessorRecipe::RepairCharge => ProcessorRecipe::Ammunition,
                            ProcessorRecipe::Ammunition => ProcessorRecipe::Fuel,
                            ProcessorRecipe::Fuel => ProcessorRecipe::RepairCharge,
                        };
                    }
                }
                if command.raw.pressed(INPUT_SPACE_EDGE)
                    || command.station.op == StationControlOp::InfrastructureToggleBlocker
                {
                    if let Some(mut junction_cmd) = junction_cmd {
                        junction_cmd.open = !junction_cmd.open;
                    }
                    if let Some(mut valve_cmd) = valve_cmd {
                        valve_cmd.open = !valve_cmd.open;
                    }
                }
            }
            ShipControlMode::Computer => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let Ok((_, _, _, _, _, _, _, _, _, _, _, arch_runtime)) =
                    module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                let Some(mut arch_runtime) = arch_runtime else {
                    continue;
                };
                if command.raw.pressed(INPUT_SPACE_EDGE)
                    || command.station.op == StationControlOp::ComputerToggleEnabled
                {
                    arch_runtime.enabled = !arch_runtime.enabled;
                }
                if command.raw.pressed(INPUT_CYCLE_TEMPLATE)
                    || command.station.op == StationControlOp::ComputerCycleTemplate
                {
                    arch_runtime.program =
                        ArchProgram::from_template(arch_runtime.program.template.next());
                }
            }
        }
    }
}

/// Applies requested thrust and turning inputs to the player ship's control state for simulation.
pub(crate) fn apply_player_ship_controls(
    balance: Res<BalanceConfig>,
    time: Res<Time>,
    player_ship_query: Single<
        (
            &SimRotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &ShipMovementModel,
            &ShipPowerState,
            &mut ShipControlState,
            &mut ShipWeaponState,
            &MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    let (
        ship_rotation,
        mut linear_velocity,
        mut angular_velocity,
        movement_model,
        power_state,
        mut control_state,
        mut weapon_state,
        mission_state,
    ) = player_ship_query.into_inner();
    let dt = fx_from_time_delta(&time);
    let throttle_demand = control_state
        .throttle_demand
        .clamp(Fx::from_num(0), Fx::from_num(1));
    let mut turn_input = control_state
        .turn_input
        .clamp(Fx::from_num(-1), Fx::from_num(1));

    if mission_state.failed || mission_state.completed {
        turn_input = Fx::from_num(0);
        control_state.throttle_demand = Fx::from_num(0);
    }

    control_state.thrust_active = throttle_demand
        > Fx::from_num(balance.ship.throttle_activation_threshold)
        && !mission_state.failed
        && !mission_state.completed;
    control_state.turn_input = turn_input;
    weapon_state.cooldown_remaining = (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));

    let effective_turn_input = turn_input * power_state.engine_power_ratio;
    if effective_turn_input != Fx::from_num(0) && power_state.engines_powered {
        angular_velocity.radians_per_second = effective_turn_input * movement_model.turn_speed;
    } else {
        angular_velocity.radians_per_second = damp_scalar(
            angular_velocity.radians_per_second,
            movement_model.angular_damping,
            dt,
        );
    }

    if control_state.thrust_active && power_state.engines_powered {
        let forward = facing_vector(ship_rotation.radians);
        linear_velocity.value += forward
            * movement_model.thrust_acceleration
            * power_state.engine_power_ratio
            * throttle_demand
            * dt;
    }

    linear_velocity.value = damp_vec2(linear_velocity.value, movement_model.linear_damping, dt);
    linear_velocity.value = linear_velocity.value.clamp_length(movement_model.max_speed);
}

/// Integrates player ship velocity and rotation so control intent becomes actual ship motion.
pub(crate) fn integrate_player_ship_motion(
    time: Res<Time>,
    player_ship_query: Single<
        (
            &mut Transform,
            &mut SimPosition,
            &mut SimRotation,
            &LinearVelocity,
            &AngularVelocity,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    let (mut transform, mut position, mut rotation, linear_velocity, angular_velocity) =
        player_ship_query.into_inner();
    let dt = fx_from_time_delta(&time);

    rotation.radians += angular_velocity.radians_per_second * dt;
    position.value += linear_velocity.value * dt;
    clamp_position_to_arena(&mut position.value);

    transform.translation = render_translation(position.value, transform.translation.z);
    transform.rotation = Quat::from_rotation_z(rotation.radians.to_num::<f32>());
}

pub(crate) fn focus_station(
    control_state: &mut ShipboardControlState,
    entity: Entity,
    module_id: u64,
    kind: ModuleKind,
    family: StationFamily,
    mode: ShipControlMode,
) {
    control_state.mode = mode;
    control_state.focus_mode = StationFocusMode::Focused;
    control_state.focused_entity = Some(entity);
    control_state.focused_module_id = Some(module_id);
    control_state.focused_kind = Some(kind);
    control_state.focused_family = Some(family);
}
