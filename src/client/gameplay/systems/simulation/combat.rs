use bevy::prelude::*;

use crate::{
    client::{
        TILE_SIZE,
        gameplay::{
            HOSTILE_PROJECTILE_SPEED,
            HOSTILE_TARGET_RADIUS,
            MODULE_HIT_RADIUS,
            PROJECTILE_RADIUS,
            PROJECTILE_SPEED,
            components::{
                DestroyedModule,
                HostileTarget,
                HostileTurretPlatform,
                HostileWeaponState,
                Integrity,
                MissionState,
                ModuleRuntimeState,
                PlayerShip,
                Projectile,
                ProjectileFaction,
                RuntimeShipModule,
                ShipArchCommandState,
                ShipControlMode,
                ShipPowerState,
                ShipRoot,
                ShipWeaponState,
                ShipboardControlState,
                SimPosition,
                SimRotation,
                TurretCommandState,
                WeaponModule,
            },
            helpers::{
                Fx,
                angle_from_vector,
                fixed_radius_sq,
                fx_from_time_delta,
                is_inside_arena,
                render_translation,
                spawn_player_projectile,
                spawn_projectile_entity,
            },
        },
    },
    ship::ModuleKind,
};

pub(crate) fn fire_player_weapons(
    mut commands: Commands,
    control_mode_query: Single<&ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    player_ship_query: Single<
        (
            &Children,
            &SimPosition,
            &SimRotation,
            &ShipPowerState,
            &ShipArchCommandState,
            &mut ShipWeaponState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    weapon_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &ModuleRuntimeState,
            &TurretCommandState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
) {
    let control_mode = control_mode_query.into_inner();

    let (children, ship_position, ship_rotation, power_state, arch_commands, mut weapon_state) =
        player_ship_query.into_inner();

    let fire_requested =
        control_mode.mode == ShipControlMode::Turret || arch_commands.turret_auto_fire;
    if !fire_requested
        || !power_state.weapons_powered
        || weapon_state.turret_count == 0
        || weapon_state.cooldown_remaining > Fx::from_num(0)
    {
        return;
    }

    let mut fired_any = false;
    for child in children.iter() {
        let Ok((weapon_entity, weapon_module, runtime_state, turret_state, destroyed)) =
            weapon_query.get(*child)
        else {
            continue;
        };
        if destroyed.is_some() || runtime_state.is_disabled {
            continue;
        }
        let is_manual_turret = control_mode.mode == ShipControlMode::Turret
            && control_mode.focused_entity == Some(weapon_entity);
        if !is_manual_turret && !arch_commands.turret_auto_fire {
            continue;
        }
        if is_manual_turret && !turret_state.fire_intent {
            continue;
        }
        let world_angle = ship_rotation.radians + turret_state.actual_angle;
        let projectile_velocity = crate::client::gameplay::helpers::facing_vector(world_angle)
            * Fx::from_num(PROJECTILE_SPEED);
        if projectile_velocity.is_near_zero() {
            continue;
        }
        let muzzle_offset = crate::client::gameplay::helpers::facing_vector(world_angle)
            * Fx::from_num(TILE_SIZE * 0.35);
        let origin = ship_position.value
            + weapon_module.local_position.rotate(ship_rotation.radians)
            + muzzle_offset;
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
        weapon_state.cooldown_remaining =
            (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));
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
            weapon_state.heat_damage,
            weapon_state.electrical_damage,
            Color::srgb(0.96, 0.38, 0.24),
        );
        weapon_state.cooldown_remaining = weapon_state.cooldown_duration;
    }
}

pub(crate) fn aim_hostile_turrets(
    player_ship_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<
        (&SimPosition, &mut Transform),
        (With<HostileTarget>, With<HostileTurretPlatform>),
    >,
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
        (
            Entity,
            &RuntimeShipModule,
            &mut Integrity,
            &mut ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
        Without<HostileTarget>,
    >,
    player_ship_query: Single<
        (&SimPosition, &SimRotation, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
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
                    if projectile_pos.distance_sq(hostile_position.value) <= hostile_hit_distance_sq
                    {
                        hit_target = Some((
                            hostile_entity,
                            hostile_integrity.current - projectile.damage,
                        ));
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

                for (module_entity, runtime_module, integrity, _, destroyed) in
                    &mut player_module_query
                {
                    if destroyed.is_some() || integrity.current <= 0 {
                        continue;
                    }

                    let module_pos = ship_position.value
                        + runtime_module.local_position.rotate(ship_rotation.radians);
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
                    if let Ok((_, _, mut integrity, mut runtime_state, destroyed)) =
                        player_module_query.get_mut(module_entity)
                    {
                        integrity.current = remaining_integrity.max(0);
                        runtime_state.current_heat += projectile.heat_damage;
                        runtime_state.electrical_instability += projectile.electrical_damage;
                        runtime_state.needs_attention = true;
                        if integrity.current <= 0 && destroyed.is_none() {
                            commands.entity(module_entity).insert(DestroyedModule);
                            if module_kind == ModuleKind::Core {
                                mission_state.failed = true;
                                mission_state.failure_reason = Some("Core destroyed".to_string());
                                mission_state.encounter_cleared = false;
                                mission_state.completed = false;
                                mission_state.completion_reason = None;
                                mission_state
                                    .return_delay_remaining
                                    .get_or_insert(Fx::from_num(2.5));
                            }
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}
