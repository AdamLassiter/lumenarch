use bevy::prelude::*;

use crate::{
    client::{
        TILE_SIZE,
        balance::BalanceConfig,
        gameplay::{
            components::{
                AngularVelocity,
                DestroyedModule,
                HostileShip,
                HostileShipAi,
                HostileShipModule,
                HostileTarget,
                HostileTurretPlatform,
                HostileWeaponState,
                Integrity,
                LinearVelocity,
                MissionState,
                ModuleRuntimeState,
                PlayerShip,
                Projectile,
                ProjectileFaction,
                RuntimeShipModule,
                ShipArchCommandState,
                ShipControlMode,
                ShipMovementModel,
                ShipPowerModel,
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
                clamp_position_to_arena,
                damp_scalar,
                damp_vec2,
                facing_vector,
                fixed_radius_sq,
                fx_from_time_delta,
                is_inside_arena,
                module_effectiveness,
                render_translation,
                ship_movement_model_with_effective,
                ship_power_model_with_effective,
                spawn_player_projectile,
                spawn_projectile_entity,
                update_ship_power_state,
                wrap_radians,
            },
        },
        state::PlayingCleanup,
    },
    ship::ModuleKind,
};

pub(crate) fn fire_player_weapons(
    mut commands: Commands,
    balance: Res<BalanceConfig>,
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
        let projectile_velocity =
            facing_vector(world_angle) * Fx::from_num(balance.combat.projectile_speed);
        if projectile_velocity.is_near_zero() {
            continue;
        }
        let muzzle_offset = facing_vector(world_angle)
            * Fx::from_num(TILE_SIZE * balance.combat.muzzle_offset_tiles);
        let origin = ship_position.value
            + weapon_module.local_position.rotate(ship_rotation.radians)
            + muzzle_offset;
        spawn_player_projectile(&mut commands, origin, projectile_velocity, &balance);
        fired_any = true;
    }

    if fired_any {
        weapon_state.cooldown_remaining = weapon_state.cooldown_duration;
    }
}

pub(crate) fn sync_hostile_ship_state(
    balance: Res<BalanceConfig>,
    mut hostile_query: Query<
        (
            &Children,
            &mut ShipMovementModel,
            &mut ShipPowerModel,
            &mut ShipWeaponState,
        ),
        (With<HostileShip>, With<ShipRoot>),
    >,
    module_query: Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
) {
    for (children, mut movement_model, mut power_model, mut weapon_state) in &mut hostile_query {
        let mut live_modules = 0usize;
        let mut engine_count = 0u32;
        let mut reactor_count = 0u32;
        let mut battery_count = 0u32;
        let mut turret_count = 0u32;
        let mut effective_engines = Fx::from_num(0);
        let mut effective_reactors = Fx::from_num(0);
        let mut effective_batteries = Fx::from_num(0);
        let mut effective_turrets = Fx::from_num(0);

        for child in children.iter() {
            let Ok((runtime_module, integrity, runtime_state, destroyed, weapon_module)) =
                module_query.get(*child)
            else {
                continue;
            };
            if destroyed.is_some() {
                continue;
            }

            live_modules += 1;
            let effectiveness = module_effectiveness(integrity, runtime_state, false);
            match runtime_module.kind {
                ModuleKind::Engine => {
                    engine_count += 1;
                    effective_engines += effectiveness;
                }
                ModuleKind::Reactor => {
                    reactor_count += 1;
                    effective_reactors += effectiveness;
                }
                ModuleKind::Battery => {
                    battery_count += 1;
                    effective_batteries += effectiveness;
                }
                ModuleKind::Turret => {
                    turret_count += 1;
                    if weapon_module.is_some() && !runtime_state.is_disabled {
                        effective_turrets += effectiveness;
                    }
                }
                _ => {}
            }
        }

        *movement_model = ship_movement_model_with_effective(
            live_modules.max(1),
            engine_count,
            effective_engines,
            &balance,
        );
        *power_model = ship_power_model_with_effective(
            live_modules.max(1),
            reactor_count,
            battery_count,
            engine_count,
            turret_count,
            effective_reactors,
            effective_batteries,
            effective_engines,
            effective_turrets,
            &balance,
        );
        weapon_state.turret_count = effective_turrets.to_num::<u32>();
        if weapon_state.turret_count == 0 {
            weapon_state.cooldown_remaining = Fx::from_num(0);
        }
    }
}

pub(crate) fn drive_hostile_ships(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<
        (
            Entity,
            &Children,
            &HostileShipAi,
            &SimPosition,
            &SimRotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &ShipMovementModel,
            &ShipPowerModel,
            &mut ShipPowerState,
            &mut ShipWeaponState,
        ),
        (With<HostileShip>, With<ShipRoot>, With<HostileTarget>),
    >,
    mut turret_query: Query<
        (
            &Parent,
            &RuntimeShipModule,
            &ModuleRuntimeState,
            &mut TurretCommandState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let player_position = player_query.into_inner().value;
    let dt = fx_from_time_delta(&time);

    for (
        ship_entity,
        _children,
        ai,
        ship_position,
        ship_rotation,
        mut linear_velocity,
        mut angular_velocity,
        movement_model,
        power_model,
        mut power_state,
        mut weapon_state,
    ) in &mut hostile_query
    {
        let to_player = player_position - ship_position.value;
        if to_player.is_near_zero() {
            continue;
        }
        let distance = to_player.length();
        let desired_heading = angle_from_vector(to_player) - Fx::FRAC_PI_2;
        let angle_error = wrap_radians(desired_heading - ship_rotation.radians);
        let turn_input = angle_error
            .clamp(Fx::from_num(-1), Fx::from_num(1))
            .max(-ai.aggression)
            .min(ai.aggression);

        let mut throttle = if distance
            > ai.preferred_range * Fx::from_num(balance.hostile_ai.far_range_multiplier)
        {
            Fx::from_num(1)
        } else if distance
            < ai.preferred_range * Fx::from_num(balance.hostile_ai.near_range_multiplier)
        {
            Fx::from_num(balance.hostile_ai.close_throttle)
        } else {
            Fx::from_num(balance.hostile_ai.cruise_throttle)
        };
        if angle_error.abs() > Fx::from_num(balance.hostile_ai.turn_slowdown_angle) {
            throttle *= Fx::from_num(balance.hostile_ai.turn_slowdown_multiplier);
        }
        let wants_fire =
            angle_error.abs() <= Fx::from_num(balance.hostile_ai.firing_angle_threshold);

        weapon_state.cooldown_remaining =
            (weapon_state.cooldown_remaining - dt).max(Fx::from_num(0));
        update_ship_power_state(
            dt,
            throttle,
            turn_input,
            if wants_fire {
                Fx::from_num(1)
            } else {
                Fx::from_num(0)
            },
            power_model,
            &mut power_state,
        );

        let effective_turn_input = turn_input * power_state.engine_power_ratio;
        if effective_turn_input.abs() > Fx::from_num(0.01) && power_state.engines_powered {
            angular_velocity.radians_per_second = effective_turn_input * movement_model.turn_speed;
        } else {
            angular_velocity.radians_per_second = damp_scalar(
                angular_velocity.radians_per_second,
                movement_model.angular_damping,
                dt,
            );
        }

        if throttle > Fx::from_num(0.05) && power_state.engines_powered {
            let forward = facing_vector(ship_rotation.radians);
            linear_velocity.value += forward
                * movement_model.thrust_acceleration
                * power_state.engine_power_ratio
                * throttle
                * dt;
        }
        linear_velocity.value = damp_vec2(linear_velocity.value, movement_model.linear_damping, dt);
        linear_velocity.value = linear_velocity.value.clamp_length(movement_model.max_speed);

        for (parent, runtime_module, runtime_state, mut turret_state, destroyed) in
            &mut turret_query
        {
            if parent.get() != ship_entity || destroyed.is_some() || runtime_state.is_disabled {
                continue;
            }
            let turret_world =
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
            let to_player = player_position - turret_world;
            if to_player.is_near_zero() {
                continue;
            }
            turret_state.desired_angle =
                wrap_radians(angle_from_vector(to_player) - Fx::FRAC_PI_2 - ship_rotation.radians);
            turret_state.fire_intent = wants_fire;
        }
    }
}

pub(crate) fn integrate_hostile_ship_motion(
    time: Res<Time>,
    mut hostile_query: Query<
        (
            &mut Transform,
            &mut SimPosition,
            &mut SimRotation,
            &LinearVelocity,
            &AngularVelocity,
        ),
        (With<HostileShip>, With<ShipRoot>, With<HostileTarget>),
    >,
) {
    let dt = fx_from_time_delta(&time);

    for (mut transform, mut position, mut rotation, linear_velocity, angular_velocity) in
        &mut hostile_query
    {
        rotation.radians += angular_velocity.radians_per_second * dt;
        position.value += linear_velocity.value * dt;
        clamp_position_to_arena(&mut position.value);

        transform.translation = render_translation(position.value, transform.translation.z);
        transform.rotation = Quat::from_rotation_z(rotation.radians.to_num::<f32>());
    }
}

pub(crate) fn fire_hostile_ship_weapons(
    mut commands: Commands,
    balance: Res<BalanceConfig>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<
        (
            Entity,
            &Children,
            &SimPosition,
            &SimRotation,
            &ShipPowerState,
            &mut ShipWeaponState,
        ),
        (With<HostileShip>, With<ShipRoot>, With<HostileTarget>),
    >,
    weapon_query: Query<
        (
            &Parent,
            &RuntimeShipModule,
            &ModuleRuntimeState,
            &TurretCommandState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    for (ship_entity, children, ship_position, ship_rotation, power_state, mut weapon_state) in
        &mut hostile_query
    {
        if !power_state.weapons_powered
            || weapon_state.turret_count == 0
            || weapon_state.cooldown_remaining > Fx::from_num(0)
        {
            continue;
        }

        let mut fired_any = false;
        for child in children.iter() {
            let Ok((parent, weapon_module, runtime_state, turret_state, destroyed)) =
                weapon_query.get(*child)
            else {
                continue;
            };
            if parent.get() != ship_entity
                || destroyed.is_some()
                || runtime_state.is_disabled
                || !turret_state.fire_intent
            {
                continue;
            }

            let world_angle = ship_rotation.radians + turret_state.actual_angle;
            let projectile_velocity =
                facing_vector(world_angle) * Fx::from_num(balance.combat.hostile_projectile_speed);
            if projectile_velocity.is_near_zero() {
                continue;
            }
            let muzzle_offset = facing_vector(world_angle)
                * Fx::from_num(TILE_SIZE * balance.combat.muzzle_offset_tiles);
            let origin = ship_position.value
                + weapon_module.local_position.rotate(ship_rotation.radians)
                + muzzle_offset;
            spawn_projectile_entity(
                &mut commands,
                origin,
                projectile_velocity,
                &balance,
                ProjectileFaction::Hostile,
                balance.combat.hostile_projectile_damage,
                Fx::from_num(balance.combat.hostile_projectile_heat_damage),
                Fx::from_num(balance.combat.hostile_projectile_electrical_damage),
                Color::srgb(0.96, 0.38, 0.24),
            );
            fired_any = true;
        }

        if fired_any {
            weapon_state.cooldown_remaining = weapon_state.cooldown_duration;
        }
    }
}

pub(crate) fn fire_hostile_targets(
    mut commands: Commands,
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    player_ship_query: Single<&SimPosition, (With<PlayerShip>, With<ShipRoot>)>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut hostile_query: Query<(&SimPosition, &mut HostileWeaponState), With<HostileTurretPlatform>>,
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
            direction * Fx::from_num(balance.combat.hostile_projectile_speed),
            &balance,
            ProjectileFaction::Hostile,
            balance.combat.hostile_projectile_damage,
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
    balance: Res<BalanceConfig>,
    projectile_query: Query<(Entity, &SimPosition, &Projectile)>,
    hostile_root_query: Query<
        (
            Entity,
            &Children,
            &SimPosition,
            &SimRotation,
            &HostileShipAi,
        ),
        (With<HostileShip>, With<ShipRoot>, With<HostileTarget>),
    >,
    mut hostile_module_query: Query<
        (
            Entity,
            &Parent,
            &RuntimeShipModule,
            &mut Integrity,
            &mut ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
        With<HostileShipModule>,
    >,
    mut static_hostile_query: Query<
        (Entity, &SimPosition, &mut Integrity),
        (
            With<HostileTurretPlatform>,
            Without<HostileShipModule>,
            Without<RuntimeShipModule>,
        ),
    >,
    mut player_module_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &mut Integrity,
            &mut ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
        (With<RuntimeShipModule>, Without<HostileShipModule>),
    >,
    player_ship_query: Single<
        (&SimPosition, &SimRotation, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    let (ship_position, ship_rotation, mut mission_state) = player_ship_query.into_inner();
    let hostile_hit_distance_sq =
        fixed_radius_sq(balance.combat.hostile_target_radius + balance.combat.projectile_radius);
    let module_hit_distance_sq =
        fixed_radius_sq(balance.combat.module_hit_radius + balance.combat.projectile_radius);

    for (projectile_entity, projectile_position, projectile) in &projectile_query {
        let projectile_pos = projectile_position.value;
        match projectile.faction {
            ProjectileFaction::Player => {
                let mut hit_hostile_module = None;
                for (root_entity, children, hostile_position, hostile_rotation, ai) in
                    &hostile_root_query
                {
                    for child in children.iter() {
                        let Ok((module_entity, parent, runtime_module, integrity, _, destroyed)) =
                            hostile_module_query.get(*child)
                        else {
                            continue;
                        };
                        if parent.get() != root_entity
                            || destroyed.is_some()
                            || integrity.current <= 0
                        {
                            continue;
                        }
                        let module_pos = hostile_position.value
                            + runtime_module
                                .local_position
                                .rotate(hostile_rotation.radians);
                        if projectile_pos.distance_sq(module_pos) <= module_hit_distance_sq {
                            hit_hostile_module = Some((
                                root_entity,
                                module_entity,
                                runtime_module.kind,
                                integrity.current - projectile.damage,
                                hostile_position.value,
                                ai.salvage_reward,
                            ));
                            break;
                        }
                    }
                    if hit_hostile_module.is_some() {
                        break;
                    }
                }

                if let Some((
                    root_entity,
                    module_entity,
                    module_kind,
                    remaining_integrity,
                    root_position,
                    salvage_reward,
                )) = hit_hostile_module
                {
                    if let Ok((_, _, _, mut integrity, mut runtime_state, destroyed)) =
                        hostile_module_query.get_mut(module_entity)
                    {
                        integrity.current = remaining_integrity.max(0);
                        runtime_state.current_heat += projectile.heat_damage;
                        runtime_state.electrical_instability += projectile.electrical_damage;
                        runtime_state.needs_attention = true;
                        if integrity.current <= 0 && destroyed.is_none() {
                            commands.entity(module_entity).insert(DestroyedModule);
                            if module_kind == ModuleKind::Core {
                                spawn_hostile_salvage(&mut commands, root_position, salvage_reward);
                                commands.entity(root_entity).despawn_recursive();
                            }
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                    continue;
                }

                let mut hit_static_target = None;
                for (hostile_entity, hostile_position, hostile_integrity) in
                    &mut static_hostile_query
                {
                    if projectile_pos.distance_sq(hostile_position.value) <= hostile_hit_distance_sq
                    {
                        hit_static_target = Some((
                            hostile_entity,
                            hostile_integrity.current - projectile.damage,
                        ));
                        break;
                    }
                }

                if let Some((hostile_entity, remaining_integrity)) = hit_static_target {
                    if let Ok((_, _, mut integrity)) = static_hostile_query.get_mut(hostile_entity)
                    {
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

fn spawn_hostile_salvage(commands: &mut Commands, position: SimVec, salvage_reward: u32) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.90, 0.72, 0.28), Vec2::new(30.0, 26.0)),
        Transform::from_translation(render_translation(position, 3.0)),
        SimPosition { value: position },
        crate::client::gameplay::components::SalvagePickup {
            scrap_value: salvage_reward,
        },
        crate::client::gameplay::components::SalvageWreck,
        PlayingCleanup,
    ));
}

type SimVec = crate::client::gameplay::helpers::FixedVec2;
