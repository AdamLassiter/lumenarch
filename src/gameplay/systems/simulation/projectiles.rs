use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            DestroyedModule,
            HostileShip,
            HostileShipAi,
            HostileShipModule,
            HostileTarget,
            HostileTurretPlatform,
            Integrity,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            Projectile,
            ProjectileFaction,
            RuntimeShipModule,
            ShieldCommandState,
            ShipDamageSensorState,
            ShipRoot,
            SimPosition,
            SimRotation,
        },
        helpers::{Fx, fixed_radius_sq, fx_from_time_delta, is_inside_arena, render_translation},
        systems::simulation::helpers::{
            absorb_hostile_shield_hit,
            absorb_player_shield_hit,
            spawn_hostile_salvage,
        },
    },
    ship::ModuleKind,
};

/// Advances projectile motion and lifetime so shots travel through the arena predictably.
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

/// Applies projectile impacts to ships and scenery so fired shots have gameplay consequences.
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
            &ChildOf,
            &RuntimeShipModule,
            &mut Integrity,
            &mut ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
        With<HostileShipModule>,
    >,
    mut hostile_shield_query: Query<(&ChildOf, &mut ShieldCommandState), With<HostileShipModule>>,
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
    mut player_shield_query: Query<
        (&RuntimeShipModule, &mut ShieldCommandState),
        (
            With<RuntimeShipModule>,
            Without<HostileShipModule>,
            With<ShieldCommandState>,
        ),
    >,
    player_ship_query: Single<
        (
            &SimPosition,
            &SimRotation,
            &mut MissionState,
            &mut ShipDamageSensorState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    let (ship_position, ship_rotation, mut mission_state, mut damage_sensor) =
        player_ship_query.into_inner();
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
                    if absorb_hostile_shield_hit(
                        root_entity,
                        hostile_rotation.radians,
                        projectile,
                        &mut hostile_shield_query,
                    ) {
                        commands.entity(projectile_entity).despawn();
                        hit_hostile_module = Some((
                            root_entity,
                            Entity::PLACEHOLDER,
                            ModuleKind::Shield,
                            0,
                            hostile_position.value,
                            ai.salvage_reward,
                        ));
                        break;
                    }
                    for child in children.iter() {
                        let Ok((module_entity, parent, runtime_module, integrity, _, destroyed)) =
                            hostile_module_query.get(child)
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
                    if module_entity == Entity::PLACEHOLDER {
                        continue;
                    }
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
                                commands.entity(root_entity).despawn();
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
                            commands.entity(hostile_entity).despawn();
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                }
            }
            ProjectileFaction::Hostile => {
                if absorb_player_shield_hit(
                    ship_rotation.radians,
                    projectile,
                    &mut player_shield_query,
                ) {
                    commands.entity(projectile_entity).despawn();
                    continue;
                }
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
                            runtime_module.local_position,
                            integrity.current - projectile.damage,
                        ));
                        break;
                    }
                }

                if let Some((module_entity, module_kind, local_position, remaining_integrity)) =
                    hit_module
                {
                    if let Ok((_, _, mut integrity, mut runtime_state, destroyed)) =
                        player_module_query.get_mut(module_entity)
                    {
                        integrity.current = remaining_integrity.max(0);
                        runtime_state.current_heat += projectile.heat_damage;
                        runtime_state.electrical_instability += projectile.electrical_damage;
                        runtime_state.needs_attention = true;
                        damage_sensor.recent_direction = local_position.normalized_or_zero();
                        damage_sensor.recent_distance = local_position.length();
                        damage_sensor.recent_intensity = Fx::from_num(projectile.damage.max(1))
                            + projectile.heat_damage
                            + projectile.electrical_damage;
                        damage_sensor.recent_timer = Fx::from_num(1.2);
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
                                    .get_or_insert(Fx::from_num(
                                        balance.mission.return_delay_seconds,
                                    ));
                            }
                        }
                    }
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}
