use super::*;

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
        let mut effective_battery_flow = Fx::from_num(0);
        let mut effective_turrets = Fx::from_num(0);
        let mut effective_helm = Fx::from_num(1);
        let mut shield_count = 0u32;

        for child in children.iter() {
            let Ok((runtime_module, integrity, runtime_state, destroyed, weapon_module)) =
                module_query.get(child)
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
                    effective_battery_flow += effectiveness;
                }
                ModuleKind::Turret => {
                    turret_count += 1;
                    if weapon_module.is_some() && !runtime_state.is_disabled {
                        effective_turrets += effectiveness;
                    }
                }
                ModuleKind::Cockpit => {
                    effective_helm = effective_helm.max(effectiveness.max(Fx::from_num(1)));
                }
                ModuleKind::Shield => {
                    shield_count += 1;
                }
                _ => {}
            }
        }

        *movement_model = ship_movement_model_with_effective(
            live_modules.max(1),
            engine_count,
            effective_engines,
            effective_helm,
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
            effective_battery_flow.max(Fx::from_num(battery_count.max(1))),
            effective_battery_flow.max(Fx::from_num(battery_count.max(1))),
            effective_engines,
            effective_turrets,
            &balance,
        );
        weapon_state.turret_count = effective_turrets.to_num::<u32>();
        weapon_state.shield_count = shield_count;
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
            &ChildOf,
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
            &ChildOf,
            &RuntimeShipModule,
            &ModuleRuntimeState,
            &WeaponModule,
            &TurretCommandState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
    mut storage_query: Query<(&ChildOf, &mut StorageModule)>,
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
        let mut cooldown_after_shot = Fx::from_num(0);
        for child in children.iter() {
            let Ok((parent, weapon_module, runtime_state, weapon_stats, turret_state, destroyed)) =
                weapon_query.get(child)
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
            if weapon_stats.requires_ammo
                && !helpers::consume_ship_resource(
                    &mut storage_query,
                    children,
                    crate::gameplay::components::ResourceKind::Ammunition,
                    weapon_stats.ammo_per_shot.max(1),
                )
            {
                continue;
            }

            let world_angle = ship_rotation.radians + turret_state.actual_angle;
            let projectile_velocity = facing_vector(world_angle)
                * Fx::from_num(balance.combat.hostile_projectile_speed)
                * weapon_stats.projectile_speed_multiplier;
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
                weapon_stats
                    .damage
                    .max(balance.combat.hostile_projectile_damage),
                Fx::from_num(balance.combat.hostile_projectile_heat_damage),
                Fx::from_num(balance.combat.hostile_projectile_electrical_damage),
                if weapon_stats.requires_ammo {
                    Color::srgb(0.90, 0.46, 0.30)
                } else {
                    Color::srgb(0.96, 0.38, 0.24)
                },
            );
            fired_any = true;
            cooldown_after_shot = cooldown_after_shot.max(
                Fx::from_num(balance.combat.hostile_fire_cooldown)
                    * weapon_stats.cooldown_multiplier,
            );
        }

        if fired_any {
            weapon_state.cooldown_remaining =
                cooldown_after_shot.max(weapon_state.cooldown_duration);
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
