use bevy::{prelude::*, window::PrimaryWindow};

use super::super::{
    super::state::{AbortEncounterButton, ClientAppState, MainCamera},
    CAMERA_FOLLOW_LERP_RATE,
    components::{
        AngularVelocity,
        CurrentStation,
        InternalPosition,
        LinearVelocity,
        ManipulatorCommandState,
        MissionState,
        PlayerShip,
        PlayerShipAssignment,
        ProcessorCommandState,
        ReactorCommandState,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipControlMode,
        ShipControlState,
        ShipInteriorMap,
        ShipMovementModel,
        ShipPowerModel,
        ShipPowerState,
        ShipRoot,
        ShipWeaponState,
        ShipboardControlState,
        ShipboardMarker,
        ShipboardPlayer,
        SimPosition,
        SimRotation,
        StationFamily,
        StationFocusMode,
        StorageCommandState,
        TurretCommandState,
    },
    helpers::{
        Fx,
        angle_from_vector,
        clamp_position_to_arena,
        damp_scalar,
        damp_vec2,
        facing_vector,
        fx_from_time_delta,
        render_translation,
        update_ship_power_state,
        wrap_radians,
    },
};
use crate::ship::ModuleKind;

const INTERIOR_CAMERA_SCALE: f32 = 0.58;
const EXTERIOR_CAMERA_SCALE: f32 = 1.0;

pub(crate) fn return_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<AbortEncounterButton>,
        ),
    >,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.44, 0.20, 0.14));
                next_state.set(ClientAppState::Docked);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.64, 0.34, 0.22));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.52, 0.27, 0.18));
            }
        }
    }
}

pub(crate) fn return_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        next_state.set(ClientAppState::Docked);
    }
}

pub(crate) fn camera_follow_player_ship(
    time: Res<Time>,
    ship_query: Single<
        (&SimPosition, &SimRotation, &ShipboardControlState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<&InternalPosition, (With<ShipboardPlayer>, With<PlayerShipAssignment>)>,
    module_query: Query<&RuntimeShipModule>,
    camera_query: Single<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, With<MainCamera>),
    >,
) {
    let (ship_position, ship_rotation, control_state) = ship_query.into_inner();
    let player_position = player_query.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();

    let ship_world = ship_position.value;
    let (desired_center, desired_rotation, desired_scale) = match control_state.mode {
        ShipControlMode::Interior => (
            ship_world + player_position.local_position.rotate(ship_rotation.radians),
            ship_rotation.radians.to_num::<f32>(),
            INTERIOR_CAMERA_SCALE,
        ),
        ShipControlMode::Reactor | ShipControlMode::Logistics | ShipControlMode::Computer => {
            let focus_pos = control_state
                .focused_entity
                .and_then(|entity| module_query.get(entity).ok())
                .map(|runtime_module| {
                    ship_world + runtime_module.local_position.rotate(ship_rotation.radians)
                })
                .unwrap_or(ship_world);
            (
                focus_pos,
                ship_rotation.radians.to_num::<f32>(),
                INTERIOR_CAMERA_SCALE,
            )
        }
        ShipControlMode::Cockpit | ShipControlMode::Turret => {
            (ship_world, 0.0, EXTERIOR_CAMERA_SCALE)
        }
    };

    let blend = 1.0 - (-CAMERA_FOLLOW_LERP_RATE * time.delta_secs()).exp();
    let desired_center = desired_center.to_vec2();
    camera_transform.translation.x += (desired_center.x - camera_transform.translation.x) * blend;
    camera_transform.translation.y += (desired_center.y - camera_transform.translation.y) * blend;
    let current_angle = camera_transform.rotation.to_euler(EulerRot::XYZ).2;
    let angle_delta = wrap_angle_f32(desired_rotation - current_angle);
    camera_transform.rotation = Quat::from_rotation_z(current_angle + angle_delta * blend);
    projection.scale += (desired_scale - projection.scale) * blend;
}

pub(crate) fn toggle_shipboard_control_mode(
    keys: Res<ButtonInput<KeyCode>>,
    ship_query: Single<(&mut ShipboardControlState, &Children), (With<PlayerShip>, With<ShipRoot>)>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    station_query: Single<&CurrentStation, (With<ShipboardPlayer>, With<PlayerShipAssignment>)>,
    module_query: Query<(Entity, &RuntimeShipModule)>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let (mut control_state, children) = ship_query.into_inner();
    let current_station = station_query.into_inner();
    if control_state.mode == ShipControlMode::Interior {
        if current_station.kind != ModuleKind::Cockpit {
            return;
        }
        if let Some((entity, runtime_module)) = children
            .iter()
            .find_map(|entity| module_query.get(*entity).ok())
            .filter(|(_, runtime_module)| runtime_module.kind == ModuleKind::Cockpit)
        {
            focus_station(
                &mut control_state,
                entity,
                runtime_module.module_id,
                runtime_module.kind,
                StationFamily::Cockpit,
                ShipControlMode::Cockpit,
            );
        }
    }
}

pub(crate) fn exit_focused_station(
    keys: Res<ButtonInput<KeyCode>>,
    ship_query: Single<&mut ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    if !keys.just_pressed(KeyCode::Escape) && !keys.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let mut control_state = ship_query.into_inner();
    if control_state.focus_mode == StationFocusMode::Focused {
        control_state.mode = ShipControlMode::Interior;
        control_state.focus_mode = StationFocusMode::Internal;
        control_state.focused_entity = None;
        control_state.focused_module_id = None;
        control_state.focused_kind = None;
        control_state.focused_family = None;
    }
}

pub(crate) fn move_shipboard_player(
    keys: Res<ButtonInput<KeyCode>>,
    ship_query: Single<&ShipInteriorMap, (With<PlayerShip>, With<ShipRoot>)>,
    mode_query: Single<&ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<
        (&mut InternalPosition, &mut CurrentStation),
        (With<ShipboardPlayer>, With<PlayerShipAssignment>),
    >,
) {
    if mode_query.into_inner().mode != ShipControlMode::Interior {
        return;
    }
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let mut delta = (0, 0);
    if keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) {
        delta.1 -= 1;
    } else if keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown) {
        delta.1 += 1;
    } else if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
        delta.0 -= 1;
    } else if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
        delta.0 += 1;
    } else {
        return;
    }

    let interior = ship_query.into_inner();
    let (mut position, mut station) = player_query.into_inner();
    let target_x = position.grid_x + delta.0;
    let target_y = position.grid_y + delta.1;
    let Some((node_index, node)) = interior
        .walkable_nodes
        .iter()
        .enumerate()
        .find(|(_, node)| node.grid_x == target_x && node.grid_y == target_y)
    else {
        return;
    };

    position.node_index = node_index;
    position.grid_x = node.grid_x;
    position.grid_y = node.grid_y;
    position.local_position = node.local_position;
    station.module_id = node.module_id;
    station.kind = node.kind;
}

pub(crate) fn sync_shipboard_player_visual(
    ship_query: Single<&ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<
        (
            &InternalPosition,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
        ),
        (With<ShipboardMarker>, With<ShipboardPlayer>),
    >,
) {
    let control_state = ship_query.into_inner();
    let (position, mut transform, mut sprite, mut visibility) = player_query.into_inner();
    transform.translation = render_translation(position.local_position, 6.0);
    sprite.color = match control_state.mode {
        ShipControlMode::Interior => Color::srgb(0.82, 0.96, 0.62),
        ShipControlMode::Cockpit | ShipControlMode::Turret => Color::srgba(0.82, 0.96, 0.62, 0.20),
        ShipControlMode::Reactor | ShipControlMode::Logistics | ShipControlMode::Computer => {
            Color::srgb(0.92, 0.96, 0.68)
        }
    };
    *visibility = if matches!(
        control_state.mode,
        ShipControlMode::Cockpit | ShipControlMode::Turret
    ) {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}

pub(crate) fn update_station_command_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    ship_query: Single<
        (
            &SimPosition,
            &SimRotation,
            &MissionState,
            &mut ShipboardControlState,
            &mut ShipControlState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut TurretCommandState>,
        Option<&mut ReactorCommandState>,
        Option<&mut StorageCommandState>,
        Option<&mut ManipulatorCommandState>,
        Option<&mut ProcessorCommandState>,
        Option<&mut RuntimeArchComputer>,
    )>,
    candidate_query: Query<&RuntimeShipModule>,
) {
    let dt = fx_from_time_delta(&time);
    let (ship_position, ship_rotation, mission_state, control_state, mut ship_controls) =
        ship_query.into_inner();

    ship_controls.fire_pressed = false;
    if mission_state.failed || mission_state.completed {
        ship_controls.throttle_demand = Fx::from_num(0);
        ship_controls.turn_input = Fx::from_num(0);
        return;
    }

    if control_state.mode != ShipControlMode::Cockpit {
        ship_controls.throttle_demand = Fx::from_num(0);
        ship_controls.turn_input = Fx::from_num(0);
    }

    match control_state.mode {
        ShipControlMode::Interior => {}
        ShipControlMode::Cockpit => {
            if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
                ship_controls.throttle_demand = (ship_controls.throttle_demand
                    + dt * Fx::from_num(0.9))
                .clamp(Fx::from_num(0), Fx::from_num(1));
            }
            if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
                ship_controls.throttle_demand = (ship_controls.throttle_demand
                    - dt * Fx::from_num(0.9))
                .clamp(Fx::from_num(0), Fx::from_num(1));
            }
            ship_controls.turn_input = Fx::from_num(0);
            if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
                ship_controls.turn_input += Fx::from_num(1);
            }
            if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
                ship_controls.turn_input -= Fx::from_num(1);
            }

            if mouse_buttons.pressed(MouseButton::Left)
                && let Some((cursor_x, cursor_y)) = cursor_normalized(&window_query)
            {
                ship_controls.turn_input = Fx::from_num(-cursor_x);
                ship_controls.throttle_demand =
                    Fx::from_num(((1.0 - cursor_y) * 0.5).clamp(0.0, 1.0));
            }
            ship_controls.turn_input = ship_controls
                .turn_input
                .clamp(Fx::from_num(-1), Fx::from_num(1));
        }
        ShipControlMode::Turret => {
            let Some(focused_entity) = control_state.focused_entity else {
                return;
            };
            let Ok((_, runtime_module, turret_state, _, _, _, _, _)) =
                module_query.get_mut(focused_entity)
            else {
                return;
            };
            let Some(mut turret_state) = turret_state else {
                return;
            };
            if let Some(cursor_world) = cursor_world_position(&window_query, &camera_query) {
                let turret_world = ship_position.value
                    + runtime_module.local_position.rotate(ship_rotation.radians);
                let to_cursor = cursor_world - turret_world;
                if !to_cursor.is_near_zero() {
                    turret_state.desired_angle = wrap_radians(
                        angle_from_vector(to_cursor) - Fx::FRAC_PI_2 - ship_rotation.radians,
                    );
                }
            }
            if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
                turret_state.desired_angle += dt * Fx::from_num(1.8);
            }
            if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
                turret_state.desired_angle -= dt * Fx::from_num(1.8);
            }
            turret_state.desired_angle = wrap_radians(turret_state.desired_angle);
            turret_state.fire_intent =
                keys.pressed(KeyCode::Space) || mouse_buttons.pressed(MouseButton::Left);
        }
        ShipControlMode::Reactor => {
            let Some(focused_entity) = control_state.focused_entity else {
                return;
            };
            let Ok((_, _, _, reactor_state, _, _, _, _)) = module_query.get_mut(focused_entity)
            else {
                return;
            };
            let Some(mut reactor_state) = reactor_state else {
                return;
            };
            if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
                reactor_state.reaction_rate += dt * Fx::from_num(0.45);
            }
            if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
                reactor_state.reaction_rate -= dt * Fx::from_num(0.45);
            }
            if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
                reactor_state.turbine_load += dt * Fx::from_num(0.45);
            }
            if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
                reactor_state.turbine_load -= dt * Fx::from_num(0.45);
            }
            reactor_state.reaction_rate = reactor_state
                .reaction_rate
                .clamp(Fx::from_num(0), Fx::from_num(1));
            reactor_state.turbine_load = reactor_state
                .turbine_load
                .clamp(Fx::from_num(0), Fx::from_num(1));
        }
        ShipControlMode::Logistics => {
            let Some(focused_entity) = control_state.focused_entity else {
                return;
            };
            let candidate_ids = nearby_logistics_target_ids(
                control_state.focused_module_id.unwrap_or_default(),
                &candidate_query,
            );
            let Ok((_, runtime_module, _, _, storage_cmd, manipulator_cmd, processor_cmd, _)) =
                module_query.get_mut(focused_entity)
            else {
                return;
            };
            if let Some(mut storage_cmd) = storage_cmd
                && keys.just_pressed(KeyCode::Space)
            {
                storage_cmd.allow_intake = !storage_cmd.allow_intake;
            }
            if let Some(mut manipulator_cmd) = manipulator_cmd {
                if keys.just_pressed(KeyCode::KeyM) {
                    manipulator_cmd.manual_mode = !manipulator_cmd.manual_mode;
                }
                if keys.just_pressed(KeyCode::KeyR) {
                    manipulator_cmd.resource_kind = match manipulator_cmd.resource_kind {
                        crate::client::gameplay::components::ResourceKind::RawSalvage => {
                            crate::client::gameplay::components::ResourceKind::RepairCharge
                        }
                        crate::client::gameplay::components::ResourceKind::RepairCharge => {
                            crate::client::gameplay::components::ResourceKind::RawSalvage
                        }
                    };
                }
                if keys.just_pressed(KeyCode::Space) {
                    manipulator_cmd.transfer_enabled = !manipulator_cmd.transfer_enabled;
                }
                if !candidate_ids.is_empty()
                    && (keys.just_pressed(KeyCode::BracketLeft)
                        || keys.just_pressed(KeyCode::BracketRight))
                {
                    let direction = if keys.just_pressed(KeyCode::BracketLeft) {
                        -1
                    } else {
                        1
                    };
                    let current_index = manipulator_cmd
                        .target_module_id
                        .and_then(|module_id| candidate_ids.iter().position(|id| *id == module_id))
                        .unwrap_or(0);
                    let next_index = ((current_index as i32 + direction)
                        .rem_euclid(candidate_ids.len() as i32))
                        as usize;
                    manipulator_cmd.target_module_id = Some(candidate_ids[next_index]);
                    manipulator_cmd.source_module_id = Some(runtime_module.module_id);
                }
            }
            if let Some(mut processor_cmd) = processor_cmd {
                if keys.just_pressed(KeyCode::Space) {
                    processor_cmd.enabled = !processor_cmd.enabled;
                }
                if keys.just_pressed(KeyCode::KeyR) {
                    processor_cmd.selected_recipe =
                        crate::client::gameplay::components::ProcessorRecipe::RepairCharge;
                }
            }
        }
        ShipControlMode::Computer => {
            let Some(focused_entity) = control_state.focused_entity else {
                return;
            };
            let Ok((_, _, _, _, _, _, _, arch_runtime)) = module_query.get_mut(focused_entity)
            else {
                return;
            };
            let Some(mut arch_runtime) = arch_runtime else {
                return;
            };
            if keys.just_pressed(KeyCode::Space) {
                arch_runtime.enabled = !arch_runtime.enabled;
            }
            if keys.just_pressed(KeyCode::KeyT) {
                arch_runtime.program = crate::ship::arch::ArchProgram::from_template(
                    arch_runtime.program.template.next(),
                );
            }
        }
    }
}

pub(crate) fn apply_player_ship_controls(
    time: Res<Time>,
    player_ship_query: Single<
        (
            &SimRotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &ShipMovementModel,
            &ShipPowerModel,
            &mut ShipPowerState,
            &ShipboardControlState,
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
        power_model,
        mut power_state,
        control_mode,
        mut control_state,
        mut weapon_state,
        mission_state,
    ) = player_ship_query.into_inner();
    let dt = fx_from_time_delta(&time);
    let throttle_demand = if control_mode.mode == ShipControlMode::Cockpit {
        control_state
            .throttle_demand
            .clamp(Fx::from_num(0), Fx::from_num(1))
    } else {
        Fx::from_num(0)
    };
    let mut turn_input = if control_mode.mode == ShipControlMode::Cockpit {
        control_state
            .turn_input
            .clamp(Fx::from_num(-1), Fx::from_num(1))
    } else {
        Fx::from_num(0)
    };

    if mission_state.failed || mission_state.completed {
        turn_input = Fx::from_num(0);
        control_state.throttle_demand = Fx::from_num(0);
    }

    control_state.thrust_active =
        throttle_demand > Fx::from_num(0.05) && !mission_state.failed && !mission_state.completed;
    control_state.turn_input = turn_input;
    control_state.fire_pressed = false;
    weapon_state.cooldown_remaining = (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));

    update_ship_power_state(
        dt,
        throttle_demand,
        turn_input,
        power_model,
        &mut power_state,
    );

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

fn wrap_angle_f32(angle: f32) -> f32 {
    let mut angle = angle;
    while angle <= -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }
    while angle > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    angle
}

fn cursor_normalized(window: &Window) -> Option<(f32, f32)> {
    let cursor = window.cursor_position()?;
    let x = (cursor.x / window.width()) * 2.0 - 1.0;
    let y = (cursor.y / window.height()) * 2.0 - 1.0;
    Some((x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0)))
}

fn cursor_world_position(
    window: &Window,
    camera_query: &(&Camera, &GlobalTransform),
) -> Option<crate::client::gameplay::helpers::FixedVec2> {
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = camera_query;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    Some(crate::client::gameplay::helpers::FixedVec2::from_vec2(
        world,
    ))
}

fn nearby_logistics_target_ids(
    focused_module_id: u64,
    candidate_query: &Query<&RuntimeShipModule>,
) -> Vec<u64> {
    candidate_query
        .iter()
        .filter(|runtime_module| {
            runtime_module.kind == ModuleKind::Cargo
                || runtime_module.kind == ModuleKind::Processor
                || runtime_module.kind == ModuleKind::Airlock
        })
        .filter(|runtime_module| runtime_module.module_id != focused_module_id)
        .map(|runtime_module| runtime_module.module_id)
        .collect()
}
