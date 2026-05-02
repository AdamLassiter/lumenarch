use std::{collections::BTreeSet, ops::DerefMut};

use bevy::{ecs::relationship::Relationship, log, prelude::*};
use bevy_ggrs::PlayerInputs;

use super::super::{
    super::state::{AbortEncounterButton, ClientAppState, MainCamera, PlayingCleanup},
    components::{
        AirlockCommandState,
        AngularVelocity,
        CarriedResource,
        CurrentStation,
        DestroyedModule,
        InternalPosition,
        LinearVelocity,
        LooseCargo,
        ManipulatorCommandState,
        MissionState,
        ObservedLocalPlayerMarker,
        PlayerFieldState,
        PlayerHandleComponent,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerShip,
        PlayerShipAssignment,
        ProcessorCommandState,
        ReactorCommandState,
        ResourceKind,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipArchCommandState,
        ShipAtmosphereState,
        ShipControlMode,
        ShipControlState,
        ShipInertiaField,
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
        StorageModule,
        TurretCommandState,
    },
    helpers::{
        FixedVec2,
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
use crate::{
    balance::BalanceConfig,
    netcode::{self, LumenGgrsConfig},
    ship::ModuleKind,
};

const INTERIOR_CAMERA_SCALE: f32 = 0.58;
const EXTERIOR_CAMERA_SCALE: f32 = 1.0;
const PLAYER_WALK_ACCELERATION: f32 = 260.0;
const PLAYER_WALK_MAX_SPEED: f32 = 120.0;
const PLAYER_WALK_DAMPING: f32 = 8.0;
const PLAYER_EVA_ACCELERATION: f32 = 180.0;
const PLAYER_EVA_MAX_SPEED: f32 = 140.0;
const PLAYER_EVA_DAMPING: f32 = 1.6;
const PLAYER_INTERACT_RADIUS: f32 = 28.0;
const PLAYER_CARGO_PICKUP_RADIUS: f32 = 20.0;
const PLAYER_COLLISION_RADIUS: f32 = 7.0;

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
                log::info!("Abort Encounter button pressed");
                log::info!("Returning to Docked state");
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
        log::info!("Tab key pressed");
        log::info!("Returning to Docked state");
    }
}

pub(crate) fn camera_follow_player_ship(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    ship_query: Single<&ShipboardControlState, With<ObservedLocalPlayerMarker>>,
    player_query: Single<(&PlayerMotionState, &CurrentStation), With<ObservedLocalPlayerMarker>>,
    ship_frame_query: Query<(Entity, &SimPosition, &SimRotation), With<ShipRoot>>,
    module_query: Query<(&RuntimeShipModule, &ChildOf)>,
    camera_query: Single<(&mut Transform, &mut Projection), (With<Camera2d>, With<MainCamera>)>,
) {
    let control_state = ship_query.into_inner();
    let (player_motion, current_station) = player_query.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();

    let Projection::Orthographic(projection) = projection.deref_mut() else {
        return;
    };

    let active_ship = match player_motion.frame {
        PlayerReferenceFrame::World => None,
        PlayerReferenceFrame::Ship(ship_entity) => ship_frame_query.get(ship_entity).ok(),
    };
    let player_world = player_motion.world_position;
    let (desired_center, desired_rotation, desired_scale) = match control_state.mode {
        ShipControlMode::Interior => {
            if let Some((_, _, ship_rotation)) = active_ship {
                (
                    player_world,
                    ship_rotation.radians.to_num::<f32>(),
                    INTERIOR_CAMERA_SCALE,
                )
            } else {
                (player_world, 0.0, INTERIOR_CAMERA_SCALE)
            }
        }
        ShipControlMode::Reactor | ShipControlMode::Logistics | ShipControlMode::Computer => {
            let focus_pos = control_state
                .focused_entity
                .and_then(|entity| module_query.get(entity).ok())
                .and_then(|(runtime_module, parent)| {
                    ship_frame_query
                        .get(parent.get())
                        .ok()
                        .map(|(_, ship_pos, ship_rot)| {
                            ship_pos.value + runtime_module.local_position.rotate(ship_rot.radians)
                        })
                })
                .unwrap_or(player_world);
            let desired_rotation = active_ship
                .map(|(_, _, ship_rotation)| ship_rotation.radians.to_num::<f32>())
                .unwrap_or(0.0);
            (focus_pos, desired_rotation, INTERIOR_CAMERA_SCALE)
        }
        ShipControlMode::Cockpit | ShipControlMode::Turret => {
            if let Some(ship_entity) = match player_motion.frame {
                PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
                PlayerReferenceFrame::World => None,
            } {
                if let Ok((_, ship_position, _)) = ship_frame_query.get(ship_entity) {
                    (ship_position.value, 0.0, EXTERIOR_CAMERA_SCALE)
                } else {
                    (player_world, 0.0, EXTERIOR_CAMERA_SCALE)
                }
            } else if let Some((_, parent)) = module_query
                .iter()
                .find(|(runtime_module, _)| runtime_module.module_id == current_station.module_id)
            {
                if let Ok((_, ship_position, _)) = ship_frame_query.get(parent.get()) {
                    (ship_position.value, 0.0, EXTERIOR_CAMERA_SCALE)
                } else {
                    (player_world, 0.0, EXTERIOR_CAMERA_SCALE)
                }
            } else {
                (player_world, 0.0, EXTERIOR_CAMERA_SCALE)
            }
        }
    };

    let blend = 1.0 - (-balance.combat.camera_follow_lerp_rate * time.delta_secs()).exp();
    let desired_center = desired_center.to_vec2();
    camera_transform.translation.x += (desired_center.x - camera_transform.translation.x) * blend;
    camera_transform.translation.y += (desired_center.y - camera_transform.translation.y) * blend;
    let current_angle = camera_transform.rotation.to_euler(EulerRot::XYZ).2;
    let angle_delta = wrap_angle_f32(desired_rotation - current_angle);
    camera_transform.rotation = Quat::from_rotation_z(current_angle + angle_delta * blend);
    projection.scale += (desired_scale - projection.scale) * blend;
}

pub(crate) fn toggle_shipboard_control_mode(
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &mut ShipboardControlState,
            &CurrentStation,
        ),
        With<PlayerShipAssignment>,
    >,
    module_query: Query<(Entity, &RuntimeShipModule)>,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (handle, mut control_state, current_station) in players {
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        if !command.raw.pressed(netcode::INPUT_TOGGLE_STATION) {
            continue;
        }
        if control_state.mode == ShipControlMode::Interior {
            if current_station.kind != ModuleKind::Cockpit {
                continue;
            }
            if let Some((entity, runtime_module)) = module_query
                .iter()
                .find(|(_, runtime_module)| runtime_module.module_id == current_station.module_id)
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
}

pub(crate) fn exit_focused_station(
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    mut ship_query: Query<(&PlayerHandleComponent, &mut ShipboardControlState)>,
) {
    let mut players: Vec<_> = ship_query.iter_mut().collect();
    players.sort_by_key(|(handle, _)| handle.handle);
    for (handle, mut control_state) in players {
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        if !command.raw.pressed(netcode::INPUT_EXIT_STATION) {
            continue;
        }
        if control_state.focus_mode == StationFocusMode::Focused {
            control_state.mode = ShipControlMode::Interior;
            control_state.focus_mode = StationFocusMode::Internal;
            control_state.focused_entity = None;
            control_state.focused_module_id = None;
            control_state.focused_kind = None;
            control_state.focused_family = None;
        }
    }
}

pub(crate) fn update_player_reference_frame(
    ship_query: Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipInertiaField,
        ),
        With<ShipRoot>,
    >,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &mut PlayerMotionState,
            &mut InternalPosition,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (_, mut motion, mut internal_position) in players {
        let previous_frame = motion.frame;
        let mut best_ship = None;
        let mut best_distance_sq = None;

        for (ship_entity, ship_position, ship_rotation, ship_velocity, inertia_field) in &ship_query
        {
            let offset = motion.world_position - ship_position.value;
            let distance_sq = offset.length_sq();
            let radius_sq = fixed_square(inertia_field.radius);
            if distance_sq <= radius_sq
                && best_distance_sq.is_none_or(|current| distance_sq < current)
            {
                best_distance_sq = Some(distance_sq);
                let local_position = offset.rotate(-ship_rotation.radians);
                let relative_velocity = motion.world_velocity - ship_velocity.value;
                let local_velocity = relative_velocity.rotate(-ship_rotation.radians);
                best_ship = Some((ship_entity, local_position, local_velocity));
            }
        }

        match best_ship {
            Some((ship_entity, local_position, local_velocity)) => {
                motion.frame = PlayerReferenceFrame::Ship(ship_entity);
                if !matches!(previous_frame, PlayerReferenceFrame::Ship(current) if current == ship_entity)
                {
                    motion.local_position = local_position;
                    motion.local_velocity = local_velocity;
                }
                internal_position.local_position = local_position;
                internal_position.grid_x = (local_position.x / Fx::from_num(32)).to_num::<i32>();
                internal_position.grid_y = (-local_position.y / Fx::from_num(32)).to_num::<i32>();
            }
            None => {
                if let PlayerReferenceFrame::Ship(ship_entity) = previous_frame
                    && let Ok((_, _, ship_rotation, ship_velocity, _)) = ship_query.get(ship_entity)
                {
                    motion.world_velocity =
                        ship_velocity.value + motion.local_velocity.rotate(ship_rotation.radians);
                }
                motion.frame = PlayerReferenceFrame::World;
                motion.local_velocity = FixedVec2::zero();
            }
        }
    }
}

pub(crate) fn move_shipboard_player(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    ship_query: Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipAtmosphereState,
        ),
        With<ShipRoot>,
    >,
    module_query: Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &ShipboardControlState,
            &mut PlayerMotionState,
            &mut InternalPosition,
            &PlayerFieldState,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let dt = fx_from_time_delta(&time);
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _, _, _)| handle.handle);
    for (handle, control_state, mut motion, mut position, player_fields) in players {
        if control_state.mode != ShipControlMode::Interior {
            anchor_player_to_focused_station(
                &mut motion,
                &mut position,
                control_state,
                &ship_query,
                &module_query,
            );
            continue;
        }

        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        let input =
            FixedVec2::from_num(command.move_x as i32, command.move_y as i32).normalized_or_zero();

        match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => {
                let Ok((_, ship_position, ship_rotation, ship_velocity, atmosphere_state)) =
                    ship_query.get(ship_entity)
                else {
                    motion.frame = PlayerReferenceFrame::World;
                    continue;
                };
                let movement_multiplier = if player_fields.oxygen_critical {
                    Fx::from_num(balance.atmosphere.critical_oxygen_speed_multiplier)
                } else if player_fields.oxygen_warning {
                    Fx::from_num(balance.atmosphere.low_oxygen_speed_multiplier)
                } else {
                    Fx::from_num(1)
                };
                motion.local_velocity +=
                    input * Fx::from_num(PLAYER_WALK_ACCELERATION) * movement_multiplier * dt;
                motion.local_velocity =
                    damp_vec2(motion.local_velocity, Fx::from_num(PLAYER_WALK_DAMPING), dt)
                        .clamp_length(Fx::from_num(PLAYER_WALK_MAX_SPEED) * movement_multiplier);
                let local_velocity = motion.local_velocity;
                let desired_local_position = motion.local_position + local_velocity * dt;
                let collision_tiles =
                    ship_collision_tiles(ship_entity, atmosphere_state, &module_query);
                motion.local_position = resolve_ship_local_motion(
                    motion.local_position,
                    desired_local_position,
                    &collision_tiles,
                );
                motion.world_position =
                    ship_position.value + motion.local_position.rotate(ship_rotation.radians);
                motion.world_velocity =
                    ship_velocity.value + local_velocity.rotate(ship_rotation.radians);
                position.local_position = motion.local_position;
                position.grid_x = (motion.local_position.x / Fx::from_num(32)).to_num::<i32>();
                position.grid_y = (-motion.local_position.y / Fx::from_num(32)).to_num::<i32>();
            }
            PlayerReferenceFrame::World => {
                motion.world_velocity += input * Fx::from_num(PLAYER_EVA_ACCELERATION) * dt;
                motion.world_velocity =
                    damp_vec2(motion.world_velocity, Fx::from_num(PLAYER_EVA_DAMPING), dt)
                        .clamp_length(Fx::from_num(PLAYER_EVA_MAX_SPEED));
                let world_velocity = motion.world_velocity;
                motion.world_position += world_velocity * dt;
                clamp_position_to_arena(&mut motion.world_position);
            }
        }
    }
}

pub(crate) fn sync_shipboard_player_visual(
    ship_query: Single<&ShipboardControlState, With<ObservedLocalPlayerMarker>>,
    mut player_query: Query<
        (
            Option<&ObservedLocalPlayerMarker>,
            &PlayerMotionState,
            &CarriedResource,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
        ),
        (With<ShipboardMarker>, With<ShipboardPlayer>),
    >,
) {
    let control_state = ship_query.into_inner();
    for (observed, position, carried, mut transform, mut sprite, mut visibility) in
        &mut player_query
    {
        transform.translation = match position.frame {
            PlayerReferenceFrame::Ship(_) => Vec3::new(
                position.local_position.x.to_num::<f32>(),
                position.local_position.y.to_num::<f32>(),
                6.0,
            ),
            PlayerReferenceFrame::World => render_translation(position.world_position, 6.0),
        };
        if observed.is_some() {
            sprite.color = match control_state.mode {
                ShipControlMode::Interior if carried.kind.is_some() => {
                    Color::srgb(0.98, 0.88, 0.52)
                }
                ShipControlMode::Interior => Color::srgb(0.82, 0.96, 0.62),
                ShipControlMode::Cockpit | ShipControlMode::Turret => {
                    Color::srgba(0.82, 0.96, 0.62, 0.20)
                }
                ShipControlMode::Reactor
                | ShipControlMode::Logistics
                | ShipControlMode::Computer => Color::srgb(0.92, 0.96, 0.68),
            };
            *visibility = if matches!(
                control_state.mode,
                ShipControlMode::Cockpit | ShipControlMode::Turret
            ) {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

pub(crate) fn sync_player_reference_frame_parenting(
    mut commands: Commands,
    player_query: Single<
        (Entity, &PlayerMotionState, Option<&ChildOf>),
        With<ObservedLocalPlayerMarker>,
    >,
) {
    let (player_entity, motion, parent) = player_query.into_inner();
    match motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) => {
            if parent.map(ChildOf::get) != Some(ship_entity) {
                commands
                    .entity(player_entity)
                    .set_parent_in_place(ship_entity);
            }
        }
        PlayerReferenceFrame::World => {
            if parent.is_some() {
                commands.entity(player_entity).remove_parent_in_place();
            }
        }
    }
}

pub(crate) fn update_current_station(
    ship_query: Query<(Entity, &SimPosition, &SimRotation), With<ShipRoot>>,
    module_query: Query<(&RuntimeShipModule, &ChildOf)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &PlayerMotionState,
            &mut CurrentStation,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (_, motion, mut station) in players {
        let Some(active_ship) = (match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
            PlayerReferenceFrame::World => None,
        }) else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
            continue;
        };

        let Ok((_, ship_position, ship_rotation)) = ship_query.get(active_ship) else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
            continue;
        };

        let mut nearest = None;
        let mut nearest_distance_sq = None;
        let max_distance_sq = fixed_square(Fx::from_num(PLAYER_INTERACT_RADIUS));
        for (runtime_module, parent) in &module_query {
            if parent.get() != active_ship {
                continue;
            }
            let world_position =
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
            let distance_sq = motion.world_position.distance_sq(world_position);
            if distance_sq > max_distance_sq {
                continue;
            }
            if nearest_distance_sq.is_none_or(|current| distance_sq < current) {
                nearest_distance_sq = Some(distance_sq);
                nearest = Some((runtime_module.module_id, runtime_module.kind));
            }
        }

        if let Some((module_id, kind)) = nearest {
            station.module_id = module_id;
            station.kind = kind;
        } else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
        }
    }
}

pub(crate) fn handle_player_cargo_interaction(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    session_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    local_handle: Option<Res<netcode::LocalPlayerHandle>>,
    player_ship_query: Single<(Entity, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<
        (&PlayerMotionState, &CurrentStation, &mut CarriedResource),
        (With<ObservedLocalPlayerMarker>, With<PlayerShipAssignment>),
    >,
    mut loose_cargo_query: Query<(Entity, &SimPosition, &LooseCargo)>,
    mut storage_query: Query<(&RuntimeShipModule, &ChildOf, &mut StorageModule)>,
) {
    let session_input = netcode::local_player_input(session_inputs, local_handle);
    let pickup_pressed =
        keys.just_pressed(KeyCode::KeyF) || session_input.pressed(netcode::INPUT_PICKUP);
    let drop_pressed =
        keys.just_pressed(KeyCode::KeyG) || session_input.pressed(netcode::INPUT_DROP);
    if !pickup_pressed && !drop_pressed {
        return;
    }

    let (player_ship_entity, mut mission_state) = player_ship_query.into_inner();
    let (motion, current_station, mut carried) = player_query.into_inner();

    if drop_pressed {
        let Some(kind) = carried.kind else {
            return;
        };
        let amount = carried.amount.max(1);
        commands.spawn((
            Sprite::from_color(cargo_color(kind), Vec2::splat(10.0)),
            Transform::from_translation(render_translation(motion.world_position, 5.2)),
            SimPosition {
                value: motion.world_position,
            },
            LooseCargo { kind, amount },
            PlayingCleanup,
        ));
        mission_state.recent_action = Some(format!("Dropped {} {}", amount, resource_label(kind)));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }

    if carried.kind.is_none() {
        for (entity, position, cargo) in &mut loose_cargo_query {
            if motion.world_position.distance_sq(position.value)
                <= fixed_square(Fx::from_num(PLAYER_CARGO_PICKUP_RADIUS))
            {
                carried.kind = Some(cargo.kind);
                carried.amount = cargo.amount;
                mission_state.recent_action = Some(format!(
                    "Picked up {} {}",
                    cargo.amount,
                    resource_label(cargo.kind)
                ));
                mission_state.recent_action_timer = Fx::from_num(1.5);
                commands.entity(entity).despawn();
                return;
            }
        }

        let Some(active_ship) = (match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) if ship_entity != player_ship_entity => {
                Some(ship_entity)
            }
            _ => None,
        }) else {
            return;
        };

        for (runtime_module, parent, mut storage) in &mut storage_query {
            if parent.get() != active_ship || runtime_module.module_id != current_station.module_id
            {
                continue;
            }
            if let Some((kind, amount)) = take_first_available(&mut storage.inventory) {
                carried.kind = Some(kind);
                carried.amount = amount;
                mission_state.recent_action = Some(format!(
                    "Extracted {} {} from hostile ship",
                    amount,
                    resource_label(kind)
                ));
                mission_state.recent_action_timer = Fx::from_num(1.5);
                return;
            }
        }
        return;
    }

    let Some(kind) = carried.kind else {
        return;
    };
    let Some(active_ship) = (match motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) if ship_entity == player_ship_entity => {
            Some(ship_entity)
        }
        _ => None,
    }) else {
        return;
    };

    for (runtime_module, parent, mut storage) in &mut storage_query {
        if parent.get() != active_ship || runtime_module.module_id != current_station.module_id {
            continue;
        }
        if storage.inventory.total_units() + carried.amount > storage.capacity {
            continue;
        }
        storage.inventory.add(kind, carried.amount);
        if kind == ResourceKind::RawSalvage {
            mission_state.recovered_raw_salvage += carried.amount;
            mission_state.salvage_scrap_awarded += carried.amount;
            mission_state.salvage_collected = true;
            mission_state.completion_reason = Some("Cargo recovered aboard".to_string());
        } else {
            mission_state.processed_repair_charge += carried.amount;
        }
        mission_state.recent_action = Some(format!(
            "Deposited {} {} aboard ship",
            carried.amount,
            resource_label(kind)
        ));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }
}

pub(crate) fn update_station_command_input(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    ship_query: Single<(&SimPosition, &SimRotation), (With<PlayerShip>, With<ShipRoot>)>,
    player_control_query: Query<(&PlayerHandleComponent, &ShipboardControlState)>,
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
        Option<&mut RuntimeArchComputer>,
    )>,
    candidate_query: Query<&RuntimeShipModule>,
) {
    let dt = fx_from_time_delta(&time);
    let (_, ship_rotation) = ship_query.into_inner();
    let mut ship_controls = ship_control_query.into_inner();
    let mut mission_runtime = mission_query.into_inner();
    let mut control_states: Vec<_> = player_control_query.iter().collect();
    control_states.sort_by_key(|(handle, _)| handle.handle);
    let mut claimed_entities = BTreeSet::new();

    ship_controls.fire_pressed = false;
    ship_controls.throttle_demand = Fx::from_num(0);
    ship_controls.turn_input = Fx::from_num(0);
    if mission_runtime.failed || mission_runtime.completed {
        return;
    }

    for (handle, control_state) in control_states {
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
            }
            ShipControlMode::Turret => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let Ok((_, _, turret_state, _, _, _, _, _, _)) =
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
                turret_state.desired_angle +=
                    dt * Fx::from_num(command.turn_milli) / Fx::from_num(1000) * Fx::from_num(1.8);
                turret_state.desired_angle = wrap_radians(turret_state.desired_angle);
                turret_state.fire_intent = command.raw.pressed(netcode::INPUT_FIRE);
                ship_controls.fire_pressed |= turret_state.fire_intent;
            }
            ShipControlMode::Reactor => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let Ok((_, _, _, reactor_state, _, _, _, _, _)) =
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
            }
            ShipControlMode::Logistics => {
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                if !claimed_entities.insert(focused_entity) {
                    continue;
                }
                let candidate_ids = nearby_logistics_target_ids(
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
                    _,
                )) = module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                if let Some(mut airlock_state) = airlock_state {
                    if command.raw.pressed(netcode::INPUT_SPACE_EDGE) {
                        airlock_state.open = !airlock_state.open;
                        mission_runtime.airlocks_cycled += 1;
                    }
                } else if let Some(mut storage_cmd) = storage_cmd
                    && command.raw.pressed(netcode::INPUT_SPACE_EDGE)
                {
                    storage_cmd.allow_intake = !storage_cmd.allow_intake;
                }
                if let Some(mut manipulator_cmd) = manipulator_cmd {
                    if command.raw.pressed(netcode::INPUT_AUX_EDGE) {
                        manipulator_cmd.manual_mode = !manipulator_cmd.manual_mode;
                    }
                    if command.reactor_delta_milli != 0 {
                        manipulator_cmd.resource_kind = match manipulator_cmd.resource_kind {
                            crate::gameplay::components::ResourceKind::RawSalvage => {
                                crate::gameplay::components::ResourceKind::RepairCharge
                            }
                            crate::gameplay::components::ResourceKind::RepairCharge => {
                                crate::gameplay::components::ResourceKind::Fuel
                            }
                            crate::gameplay::components::ResourceKind::Fuel => {
                                crate::gameplay::components::ResourceKind::Ammunition
                            }
                            crate::gameplay::components::ResourceKind::Ammunition => {
                                crate::gameplay::components::ResourceKind::RawSalvage
                            }
                        };
                    }
                    if command.raw.pressed(netcode::INPUT_SPACE_EDGE) {
                        manipulator_cmd.transfer_enabled = !manipulator_cmd.transfer_enabled;
                    }
                    if !candidate_ids.is_empty()
                        && (command.raw.pressed(netcode::INPUT_PREV_EDGE)
                            || command.raw.pressed(netcode::INPUT_NEXT_EDGE))
                    {
                        let direction = if command.raw.pressed(netcode::INPUT_PREV_EDGE) {
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
                if let Some(mut processor_cmd) = processor_cmd {
                    if command.raw.pressed(netcode::INPUT_SPACE_EDGE) {
                        processor_cmd.enabled = !processor_cmd.enabled;
                    }
                    if command.reactor_delta_milli != 0 {
                        processor_cmd.selected_recipe = match processor_cmd.selected_recipe {
                            crate::gameplay::components::ProcessorRecipe::RepairCharge => {
                                crate::gameplay::components::ProcessorRecipe::Ammunition
                            }
                            crate::gameplay::components::ProcessorRecipe::Ammunition => {
                                crate::gameplay::components::ProcessorRecipe::Fuel
                            }
                            crate::gameplay::components::ProcessorRecipe::Fuel => {
                                crate::gameplay::components::ProcessorRecipe::RepairCharge
                            }
                        };
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
                let Ok((_, _, _, _, _, _, _, _, arch_runtime)) =
                    module_query.get_mut(focused_entity)
                else {
                    continue;
                };
                let Some(mut arch_runtime) = arch_runtime else {
                    continue;
                };
                if command.raw.pressed(netcode::INPUT_SPACE_EDGE) {
                    arch_runtime.enabled = !arch_runtime.enabled;
                }
                if command.raw.pressed(netcode::INPUT_CYCLE_TEMPLATE) {
                    arch_runtime.program = crate::ship::arch::ArchProgram::from_template(
                        arch_runtime.program.template.next(),
                    );
                }
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
            &ShipArchCommandState,
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
        arch_commands,
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

    control_state.thrust_active =
        throttle_demand > Fx::from_num(0.05) && !mission_state.failed && !mission_state.completed;
    control_state.turn_input = turn_input;
    weapon_state.cooldown_remaining = (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));

    update_ship_power_state(
        dt,
        throttle_demand,
        turn_input,
        if control_state.fire_pressed || arch_commands.turret_auto_fire {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        },
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
) -> Option<crate::gameplay::helpers::FixedVec2> {
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = camera_query;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    Some(crate::gameplay::helpers::FixedVec2::from_vec2(world))
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

fn fixed_square(value: Fx) -> crate::gameplay::helpers::WideFx {
    crate::gameplay::helpers::widen(value) * crate::gameplay::helpers::widen(value)
}

fn take_first_available(
    inventory: &mut crate::gameplay::components::ResourceInventory,
) -> Option<(ResourceKind, u32)> {
    if inventory.raw_salvage > 0 {
        inventory.raw_salvage -= 1;
        Some((ResourceKind::RawSalvage, 1))
    } else if inventory.repair_charge > 0 {
        inventory.repair_charge -= 1;
        Some((ResourceKind::RepairCharge, 1))
    } else if inventory.fuel > 0 {
        inventory.fuel -= 1;
        Some((ResourceKind::Fuel, 1))
    } else if inventory.ammunition > 0 {
        inventory.ammunition -= 1;
        Some((ResourceKind::Ammunition, 1))
    } else {
        None
    }
}

fn resource_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::RawSalvage => "raw salvage",
        ResourceKind::RepairCharge => "repair charge",
        ResourceKind::Fuel => "fuel",
        ResourceKind::Ammunition => "ammunition",
    }
}

fn cargo_color(kind: ResourceKind) -> Color {
    match kind {
        ResourceKind::RawSalvage => Color::srgb(0.90, 0.78, 0.34),
        ResourceKind::RepairCharge => Color::srgb(0.38, 0.88, 0.98),
        ResourceKind::Fuel => Color::srgb(0.98, 0.52, 0.22),
        ResourceKind::Ammunition => Color::srgb(0.86, 0.86, 0.90),
    }
}

fn anchor_player_to_focused_station(
    motion: &mut PlayerMotionState,
    position: &mut InternalPosition,
    control_state: &ShipboardControlState,
    ship_query: &Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipAtmosphereState,
        ),
        With<ShipRoot>,
    >,
    module_query: &Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) {
    let Some(focused_entity) = control_state.focused_entity else {
        return;
    };
    let Ok((parent, runtime_module, _, _)) = module_query.get(focused_entity) else {
        return;
    };
    let Ok((ship_entity, ship_position, ship_rotation, ship_velocity, _)) =
        ship_query.get(parent.get())
    else {
        return;
    };

    motion.frame = PlayerReferenceFrame::Ship(ship_entity);
    motion.local_position = runtime_module.local_position;
    motion.local_velocity = FixedVec2::zero();
    motion.world_position =
        ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
    motion.world_velocity = ship_velocity.value;

    position.grid_x = runtime_module.grid_x;
    position.grid_y = runtime_module.grid_y;
    position.local_position = runtime_module.local_position;
}

#[derive(Clone, Copy)]
struct ShipCollisionTile {
    center: FixedVec2,
    exterior_edges: u8,
    solid: bool,
    opening: bool,
}

fn ship_collision_tiles(
    ship_entity: Entity,
    atmosphere_state: &ShipAtmosphereState,
    module_query: &Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> Vec<ShipCollisionTile> {
    let module_state = module_query
        .iter()
        .filter(|(parent, _, _, _)| parent.get() == ship_entity)
        .map(|(_, runtime_module, airlock_state, destroyed)| {
            (
                runtime_module.module_id,
                (
                    destroyed.is_some(),
                    airlock_state.is_some_and(|state| state.open),
                ),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    atmosphere_state
        .tiles
        .iter()
        .map(|tile| {
            let (destroyed, airlock_open) = module_state
                .get(&tile.module_id)
                .copied()
                .unwrap_or((false, false));
            let solid = is_hull_kind(tile.kind) && !destroyed;
            let opening = tile.exterior_edges != 0 && (destroyed || airlock_open);
            ShipCollisionTile {
                center: tile.local_position,
                exterior_edges: tile.exterior_edges,
                solid,
                opening,
            }
        })
        .collect()
}

fn resolve_ship_local_motion(
    start: FixedVec2,
    desired: FixedVec2,
    collision_tiles: &[ShipCollisionTile],
) -> FixedVec2 {
    let mut resolved = start;
    let try_x = FixedVec2::new(desired.x, resolved.y);
    if !movement_blocked(resolved, try_x, collision_tiles) {
        resolved.x = desired.x;
    }
    let try_y = FixedVec2::new(resolved.x, desired.y);
    if !movement_blocked(resolved, try_y, collision_tiles) {
        resolved.y = desired.y;
    }
    resolved
}

fn movement_blocked(from: FixedVec2, to: FixedVec2, collision_tiles: &[ShipCollisionTile]) -> bool {
    if collision_tiles.iter().any(|tile| {
        tile.solid && point_overlaps_tile(to, tile.center, Fx::from_num(PLAYER_COLLISION_RADIUS))
    }) {
        return true;
    }

    let from_tile = collision_tiles
        .iter()
        .find(|tile| !tile.solid && point_inside_tile(from, tile.center));
    let to_tile = collision_tiles
        .iter()
        .find(|tile| !tile.solid && point_inside_tile(to, tile.center));

    match (from_tile, to_tile) {
        (None, Some(tile)) => !allows_exterior_crossing(from, tile),
        (Some(tile), None) => !allows_exterior_crossing(to, tile),
        _ => false,
    }
}

fn point_overlaps_tile(point: FixedVec2, tile_center: FixedVec2, radius: Fx) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half + radius
        && (point.y - tile_center.y).abs() <= tile_half + radius
}

fn point_inside_tile(point: FixedVec2, tile_center: FixedVec2) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half && (point.y - tile_center.y).abs() <= tile_half
}

fn allows_exterior_crossing(point: FixedVec2, tile: &ShipCollisionTile) -> bool {
    if !tile.opening || tile.exterior_edges == 0 {
        return false;
    }

    let tile_half = Fx::from_num(16);
    let left = tile.center.x - tile_half;
    let right = tile.center.x + tile_half;
    let top = tile.center.y + tile_half;
    let bottom = tile.center.y - tile_half;

    let mut exterior_mask = 0u8;
    if point.y > top {
        exterior_mask |= 1;
    }
    if point.x > right {
        exterior_mask |= 1 << 1;
    }
    if point.y < bottom {
        exterior_mask |= 1 << 2;
    }
    if point.x < left {
        exterior_mask |= 1 << 3;
    }

    exterior_mask != 0 && (tile.exterior_edges & exterior_mask) != 0
}

fn is_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )
}
