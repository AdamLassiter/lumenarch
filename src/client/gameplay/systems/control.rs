use bevy::prelude::*;

use super::super::{
    super::state::{ClientAppState, MainCamera, ReturnButton},
    CAMERA_FOLLOW_LERP_RATE,
    components::{
        AngularVelocity,
        CurrentStation,
        InternalPosition,
        LinearVelocity,
        MissionState,
        PlayerShip,
        PlayerShipAssignment,
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
    },
    helpers::{
        Fx,
        clamp_position_to_arena,
        damp_scalar,
        damp_vec2,
        facing_vector,
        fx_from_time_delta,
        render_translation,
        update_ship_power_state,
    },
};

pub(crate) fn return_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<ReturnButton>),
    >,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.44, 0.20, 0.14));
                next_state.set(ClientAppState::Editing);
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
        next_state.set(ClientAppState::Editing);
    }
}

pub(crate) fn camera_follow_player_ship(
    time: Res<Time>,
    player_ship_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    camera_query: Single<&mut Transform, (With<Camera2d>, With<MainCamera>)>,
) {
    let ship_position = player_ship_query.into_inner().value.to_vec2();
    let mut camera_transform = camera_query.into_inner();
    let blend = 1.0 - (-CAMERA_FOLLOW_LERP_RATE * time.delta_secs()).exp();
    camera_transform.translation.x += (ship_position.x - camera_transform.translation.x) * blend;
    camera_transform.translation.y += (ship_position.y - camera_transform.translation.y) * blend;
}

pub(crate) fn toggle_shipboard_control_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    ship_query: Single<&mut ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    if !keys.just_pressed(KeyCode::KeyC) {
        return;
    }

    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let mut control_mode = ship_query.into_inner();
    control_mode.mode = match control_mode.mode {
        ShipControlMode::ShipFlight => ShipControlMode::Internal,
        ShipControlMode::Internal => ShipControlMode::ShipFlight,
    };
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
    if mode_query.into_inner().mode != ShipControlMode::Internal {
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
        ShipControlMode::ShipFlight => Color::srgba(0.82, 0.96, 0.62, 0.45),
        ShipControlMode::Internal => Color::srgb(0.82, 0.96, 0.62),
    };
    *visibility = Visibility::Visible;
}

pub(crate) fn apply_player_ship_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    control_mode_query: Single<&ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    player_ship_query: Single<
        (
            &SimRotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &ShipMovementModel,
            &ShipPowerModel,
            &mut ShipPowerState,
            &mut ShipControlState,
            &mut ShipWeaponState,
            &MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    let flight_mode = control_mode_query.into_inner().mode == ShipControlMode::ShipFlight;
    let (
        ship_rotation,
        mut linear_velocity,
        mut angular_velocity,
        movement_model,
        power_model,
        mut power_state,
        mut control_state,
        mut weapon_state,
        mission_state,
    ) = player_ship_query.into_inner();
    let dt = fx_from_time_delta(&time);
    let thrust_active =
        flight_mode && (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp));
    let fire_pressed = flight_mode && keys.pressed(KeyCode::Space);

    let mut turn_input = Fx::from_num(0);
    if flight_mode && (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) {
        turn_input += Fx::from_num(1);
    }
    if flight_mode && (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) {
        turn_input -= Fx::from_num(1);
    }

    if mission_state.failed || mission_state.completed {
        turn_input = Fx::from_num(0);
    }

    control_state.thrust_active =
        thrust_active && !mission_state.failed && !mission_state.completed;
    control_state.turn_input = turn_input;
    control_state.fire_pressed = fire_pressed && !mission_state.failed && !mission_state.completed;
    weapon_state.cooldown_remaining = (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));

    update_ship_power_state(
        dt,
        control_state.thrust_active,
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
        linear_velocity.value +=
            forward * movement_model.thrust_acceleration * power_state.engine_power_ratio * dt;
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
