use bevy::prelude::*;

use crate::ship::ModuleKind;

use super::{
    components::{
        AngularVelocity, CollectedSalvage, DestroyedModule, HostileTarget, HostileTurretPlatform,
        HostileWeaponState, Integrity, LinearVelocity, MissionState, PlayerShip, Projectile,
        ProjectileFaction, RuntimeShipModule, SalvagePickup, SalvageWreck, ShipControlState,
        ShipMovementModel, ShipPowerModel, ShipPowerState, ShipRoot, ShipWeaponState, SimPosition,
        SimRotation, WeaponModule,
    },
    helpers::{
        Fx, angle_from_vector, clamp_position_to_arena, damp_scalar, damp_vec2, facing_vector,
        fixed_radius_sq, format_fx0, format_fx1, format_fx2, fx_from_time_delta, is_inside_arena,
        mission_return_line, mission_status_line, render_translation, salvage_status_line,
        ship_movement_model, ship_power_model, spawn_player_projectile, spawn_projectile_entity,
        update_ship_power_state,
    },
    CAMERA_FOLLOW_LERP_RATE, HOSTILE_PROJECTILE_SPEED, HOSTILE_TARGET_RADIUS, MODULE_HIT_RADIUS,
    PROJECTILE_RADIUS, PROJECTILE_SPEED, SALVAGE_PICKUP_RADIUS,
};
use super::super::{
    state::{ClientAppState, DemoProgression, GameplayStatusText, LastMissionReport, MainCamera, ReturnButton},
    TILE_SIZE,
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

pub(crate) fn update_gameplay_status_text(
    player_ship_query: Single<
        (
            &SimPosition,
            &Children,
            &LinearVelocity,
            &AngularVelocity,
            &ShipMovementModel,
            &ShipPowerState,
            &ShipWeaponState,
            &MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    hostile_query: Query<Entity, With<HostileTarget>>,
    projectile_query: Query<Entity, With<Projectile>>,
    module_integrity_query: Query<(&Integrity, Option<&DestroyedModule>), With<RuntimeShipModule>>,
    salvage_query: Query<(&SimPosition, &SalvagePickup), (With<SalvageWreck>, Without<CollectedSalvage>)>,
    progression: Res<DemoProgression>,
    mut status_query: Query<&mut Text, With<GameplayStatusText>>,
) {
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        movement_model,
        power_state,
        weapon_state,
        mission_state,
    ) = player_ship_query.into_inner();
    let salvage_line = salvage_status_line(ship_position.value, mission_state, &salvage_query);
    let mut current_integrity = 0i32;
    let mut max_integrity = 0i32;
    let mut active_modules = 0usize;
    for child in children.iter() {
        let Ok((integrity, destroyed)) = module_integrity_query.get(*child) else {
            continue;
        };
        max_integrity += integrity.max;
        if destroyed.is_none() && integrity.current > 0 {
            current_integrity += integrity.current;
            active_modules += 1;
        }
    }

    for mut text in &mut status_query {
        let status_line = match mission_return_line(mission_state) {
            Some(return_line) => format!("{}\n{}", mission_status_line(mission_state), return_line),
            None => mission_status_line(mission_state).to_string(),
        };
        **text = format!(
            "Mission Status\nOutcome: {}\nPosition: {}, {}\nVelocity: {}\nTurn Rate: {}\nIntegrity\nHull / Systems: {} / {}\nActive Modules: {}\nPower\nEngine Output: {} ({}%)\nGeneration / Draw: {} / {}\nBattery Reserve: {}\nWeapons Online: {}\nCombat\nTurrets: {}  Cooldown: {}\nProjectiles: {}  Hostiles: {}\nSalvage: {}\nScrap Total: {}",
            status_line,
            format_fx0(ship_position.value.x),
            format_fx0(ship_position.value.y),
            format_fx1(linear_velocity.value.length()),
            format_fx2(angular_velocity.radians_per_second),
            current_integrity,
            max_integrity,
            active_modules,
            movement_model.engine_count,
            format_fx0(power_state.engine_power_ratio * Fx::from_num(100)),
            format_fx1(power_state.generation),
            format_fx1(power_state.draw),
            format_fx1(power_state.stored_energy),
            if power_state.weapons_powered { "yes" } else { "no" },
            weapon_state.turret_count,
            format_fx2(weapon_state.cooldown_remaining.max(Fx::from_num(0))),
            projectile_query.iter().len(),
            hostile_query.iter().len(),
            salvage_line,
            progression.scrap,
        );
    }
}

pub(crate) fn update_mission_state(
    hostile_query: Query<Entity, With<HostileTarget>>,
    player_ship_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    let mut mission_state = player_ship_query.into_inner();
    if mission_state.failed {
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state.return_delay_remaining.get_or_insert(Fx::from_num(2.5));
        return;
    }

    if hostile_query.is_empty() {
        mission_state.encounter_cleared = true;
        mission_state.completed = mission_state.salvage_collected;
        if mission_state.completion_reason.is_none() {
            mission_state.completion_reason = Some("Encounter cleared".to_string());
        }
        if mission_state.completed {
            mission_state.return_delay_remaining.get_or_insert(Fx::from_num(2.5));
        } else {
            mission_state.return_delay_remaining = None;
        }
    } else {
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state.completion_reason = None;
        mission_state.return_delay_remaining = None;
    }
}

pub(crate) fn collect_salvage(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    player_ship_query: Single<(&SimPosition, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    salvage_query: Query<(Entity, &SimPosition, &SalvagePickup), (With<SalvageWreck>, Without<CollectedSalvage>)>,
    mut progression: ResMut<DemoProgression>,
) {
    let (ship_position, mut mission_state) = player_ship_query.into_inner();
    if !mission_state.encounter_cleared || mission_state.failed || mission_state.salvage_collected {
        return;
    }
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    let pickup_radius_sq = fixed_radius_sq(SALVAGE_PICKUP_RADIUS);
    for (entity, salvage_position, salvage_pickup) in &salvage_query {
        if ship_position.value.distance_sq(salvage_position.value) <= pickup_radius_sq {
            progression.scrap += salvage_pickup.scrap_value;
            mission_state.salvage_collected = true;
            mission_state.salvage_scrap_awarded = salvage_pickup.scrap_value;
            mission_state.completed = true;
            mission_state.completion_reason = Some("Salvage recovered".to_string());
            mission_state.return_delay_remaining = Some(Fx::from_num(2.5));
            commands.entity(entity).insert(CollectedSalvage);
            commands.entity(entity).despawn_recursive();
            break;
        }
    }
}

pub(crate) fn sync_runtime_ship_state(
    player_ship_query: Single<
        (
            &Children,
            &mut ShipMovementModel,
            &mut ShipPowerModel,
            &mut ShipWeaponState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    module_query: Query<(&RuntimeShipModule, Option<&DestroyedModule>)>,
) {
    let (children, mut movement_model, mut power_model, mut weapon_state, mut mission_state) =
        player_ship_query.into_inner();

    let mut live_modules = 0usize;
    let mut engine_count = 0u32;
    let mut reactor_count = 0u32;
    let mut battery_count = 0u32;
    let mut turret_count = 0u32;
    let mut core_alive = false;

    for child in children.iter() {
        let Ok((runtime_module, destroyed)) = module_query.get(*child) else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }

        live_modules += 1;
        match runtime_module.kind {
            ModuleKind::Core => core_alive = true,
            ModuleKind::Engine => engine_count += 1,
            ModuleKind::Reactor => reactor_count += 1,
            ModuleKind::Battery => battery_count += 1,
            ModuleKind::Turret => turret_count += 1,
            _ => {}
        }
    }

    *movement_model = ship_movement_model(live_modules.max(1), engine_count);
    *power_model = ship_power_model(
        live_modules.max(1),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
    );
    weapon_state.turret_count = turret_count;
    if turret_count == 0 {
        weapon_state.cooldown_remaining = Fx::from_num(0);
    }

    if !core_alive {
        mission_state.failed = true;
        mission_state.failure_reason = Some("Core destroyed".to_string());
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state.return_delay_remaining.get_or_insert(Fx::from_num(2.5));
    }
}

pub(crate) fn return_after_mission_resolution(
    time: Res<Time>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    progression: Res<DemoProgression>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    let mut mission_state = mission_query.into_inner();
    let Some(delay) = mission_state.return_delay_remaining.as_mut() else {
        return;
    };

    *delay = (*delay - fx_from_time_delta(&time)).max(Fx::from_num(0));
    if *delay > Fx::from_num(0) {
        return;
    }

    let (headline, detail) = if mission_state.failed {
        (
            "Mission Failed".to_string(),
            mission_state
                .failure_reason
                .clone()
                .unwrap_or_else(|| "The ship was lost.".to_string()),
        )
    } else {
        let detail = if mission_state.salvage_collected {
            format!("Recovered {} scrap from the wreck.", mission_state.salvage_scrap_awarded)
        } else {
            "Encounter cleared, but no salvage was recovered.".to_string()
        };
        ("Mission Complete".to_string(), detail)
    };

    last_mission_report.headline = Some(headline);
    last_mission_report.detail = Some(detail);
    last_mission_report.scrap_awarded = mission_state.salvage_scrap_awarded;
    last_mission_report.total_scrap = progression.scrap;
    mission_state.return_delay_remaining = None;
    next_state.set(ClientAppState::Editing);
}

pub(crate) fn apply_player_ship_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
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
    let thrust_active = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    let fire_pressed = keys.pressed(KeyCode::Space);

    let mut turn_input = Fx::from_num(0);
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        turn_input += Fx::from_num(1);
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        turn_input -= Fx::from_num(1);
    }

    if mission_state.failed || mission_state.completed {
        turn_input = Fx::from_num(0);
    }

    control_state.thrust_active = thrust_active && !mission_state.failed && !mission_state.completed;
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

    linear_velocity.value = damp_vec2(
        linear_velocity.value,
        movement_model.linear_damping,
        dt,
    );
    linear_velocity.value = linear_velocity.value.clamp_length(movement_model.max_speed);
}

pub(crate) fn fire_player_weapons(
    mut commands: Commands,
    player_ship_query: Single<
        (
            &Children,
            &SimPosition,
            &SimRotation,
            &ShipControlState,
            &ShipPowerState,
            &mut ShipWeaponState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    weapon_query: Query<(&RuntimeShipModule, Option<&DestroyedModule>), With<WeaponModule>>,
) {
    let (children, ship_position, ship_rotation, control_state, power_state, mut weapon_state) =
        player_ship_query.into_inner();

    if !control_state.fire_pressed
        || !power_state.weapons_powered
        || weapon_state.turret_count == 0
        || weapon_state.cooldown_remaining > Fx::from_num(0)
    {
        return;
    }

    let ship_forward = facing_vector(ship_rotation.radians);
    let projectile_velocity = ship_forward * Fx::from_num(PROJECTILE_SPEED);
    if projectile_velocity.is_near_zero() {
        return;
    }

    let muzzle_offset = ship_forward * Fx::from_num(TILE_SIZE * 0.35);
    let mut fired_any = false;
    for child in children.iter() {
        let Ok((weapon_module, destroyed)) = weapon_query.get(*child) else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        let origin =
            ship_position.value + weapon_module.local_position.rotate(ship_rotation.radians) + muzzle_offset;
        spawn_player_projectile(&mut commands, origin, projectile_velocity);
        fired_any = true;
    }

    if fired_any {
        weapon_state.cooldown_remaining = weapon_state.cooldown_duration;
    }
}

pub(crate) fn fire_hostile_targets(
    mut commands: Commands,
    time: Res<Time>,
    player_ship_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<(&SimPosition, &mut HostileWeaponState), With<HostileTarget>>,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let player_position = player_ship_query.into_inner().value;
    let dt = fx_from_time_delta(&time);

    for (hostile_position, mut weapon_state) in &mut hostile_query {
        weapon_state.cooldown_remaining = (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));
        if weapon_state.cooldown_remaining > Fx::from_num(0) {
            continue;
        }

        let origin = hostile_position.value;
        let direction = (player_position - origin).normalized_or_zero();
        if direction.is_near_zero() {
            continue;
        }

        spawn_projectile_entity(
            &mut commands,
            origin,
            direction * Fx::from_num(HOSTILE_PROJECTILE_SPEED),
            ProjectileFaction::Hostile,
            3,
            Color::srgb(0.96, 0.38, 0.24),
        );
        weapon_state.cooldown_remaining = weapon_state.cooldown_duration;
    }
}

pub(crate) fn aim_hostile_turrets(
    player_ship_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<(&SimPosition, &mut Transform), (With<HostileTarget>, With<HostileTurretPlatform>)>,
) {
    let player_position = player_ship_query.into_inner().value;

    for (hostile_position, mut hostile_transform) in &mut hostile_query {
        let to_player = player_position - hostile_position.value;
        if to_player.is_near_zero() {
            continue;
        }

        let angle = angle_from_vector(to_player) - Fx::from_num(std::f32::consts::FRAC_PI_2);
        hostile_transform.rotation = Quat::from_rotation_z(angle.to_num::<f32>());
    }
}

pub(crate) fn advance_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &mut SimPosition, &mut Projectile)>,
) {
    let dt = fx_from_time_delta(&time);

    for (entity, mut transform, mut position, mut projectile) in &mut projectile_query {
        projectile.remaining_life -= dt;
        position.value += projectile.velocity * dt;
        transform.translation = render_translation(position.value, transform.translation.z);

        if projectile.remaining_life <= Fx::from_num(0) || !is_inside_arena(position.value) {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn handle_projectile_hits(
    mut commands: Commands,
    projectile_query: Query<(Entity, &SimPosition, &Projectile)>,
    mut hostile_query: Query<(Entity, &SimPosition, &mut Integrity), With<HostileTarget>>,
    mut player_module_query: Query<
        (Entity, &RuntimeShipModule, &mut Integrity, Option<&DestroyedModule>),
        Without<HostileTarget>,
    >,
    player_ship_query: Single<(&SimPosition, &SimRotation, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
) {
    let (ship_position, ship_rotation, mut mission_state) = player_ship_query.into_inner();
    let hostile_hit_distance_sq = fixed_radius_sq(HOSTILE_TARGET_RADIUS + PROJECTILE_RADIUS);
    let module_hit_distance_sq = fixed_radius_sq(MODULE_HIT_RADIUS + PROJECTILE_RADIUS);

    for (projectile_entity, projectile_position, projectile) in &projectile_query {
        let projectile_pos = projectile_position.value;
        match projectile.faction {
            ProjectileFaction::Player => {
                let mut hit_target = None;

                for (hostile_entity, hostile_position, hostile_integrity) in &mut hostile_query {
                    if projectile_pos.distance_sq(hostile_position.value) <= hostile_hit_distance_sq {
                        hit_target =
                            Some((hostile_entity, hostile_integrity.current - projectile.damage));
                        break;
                    }
                }

                if let Some((hostile_entity, remaining_integrity)) = hit_target {
                    if let Ok((_, _, mut integrity)) = hostile_query.get_mut(hostile_entity) {
                        integrity.current = remaining_integrity;
                        if integrity.current <= 0 {
                            commands.entity(hostile_entity).despawn_recursive();
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                }
            }
            ProjectileFaction::Hostile => {
                let mut hit_module = None;

                for (module_entity, runtime_module, integrity, destroyed) in &mut player_module_query {
                    if destroyed.is_some() || integrity.current <= 0 {
                        continue;
                    }

                    let module_pos =
                        ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
                    if projectile_pos.distance_sq(module_pos) <= module_hit_distance_sq {
                        hit_module = Some((
                            module_entity,
                            runtime_module.kind,
                            integrity.current - projectile.damage,
                        ));
                        break;
                    }
                }

                if let Some((module_entity, module_kind, remaining_integrity)) = hit_module {
                    if let Ok((_, _, mut integrity, destroyed)) =
                        player_module_query.get_mut(module_entity)
                    {
                        integrity.current = remaining_integrity;
                        if integrity.current <= 0 && destroyed.is_none() {
                            integrity.current = 0;
                            commands.entity(module_entity).insert(DestroyedModule);
                            if module_kind == ModuleKind::Core {
                                mission_state.failed = true;
                                mission_state.failure_reason =
                                    Some("Core destroyed".to_string());
                                mission_state.encounter_cleared = false;
                                mission_state.completed = false;
                                mission_state.completion_reason = None;
                                mission_state.return_delay_remaining.get_or_insert(Fx::from_num(2.5));
                            }
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}

pub(crate) fn integrate_player_ship_motion(
    time: Res<Time>,
    player_ship_query: Single<
        (&mut Transform, &mut SimPosition, &mut SimRotation, &LinearVelocity, &AngularVelocity),
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

pub(crate) fn update_destroyed_module_visuals(
    mut module_query: Query<
        (
            &RuntimeShipModule,
            &Integrity,
            Option<&DestroyedModule>,
            &mut Sprite,
            &mut Visibility,
        ),
        Changed<Integrity>,
    >,
) {
    for (runtime_module, integrity, destroyed, mut sprite, mut visibility) in &mut module_query {
        if destroyed.is_some() || integrity.current <= 0 {
            sprite.color = Color::srgba(0.28, 0.08, 0.08, 0.12);
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;
        let fraction = integrity.current as f32 / integrity.max.max(1) as f32;
        sprite.color = match runtime_module.kind {
            ModuleKind::Hull | ModuleKind::HullCorner => {
                Color::srgb(1.0, 0.55 + 0.45 * fraction, 0.55 + 0.45 * fraction)
            }
            _ => Color::srgb(1.0, 0.4 + 0.6 * fraction, 0.4 + 0.6 * fraction),
        };
    }
}
