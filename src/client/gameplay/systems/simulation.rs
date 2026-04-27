use bevy::prelude::*;

use super::super::{
    super::{
        TILE_SIZE,
        state::{ClientAppState, DemoProgression, LastMissionReport},
    },
    HOSTILE_PROJECTILE_SPEED,
    HOSTILE_TARGET_RADIUS,
    MODULE_HIT_RADIUS,
    PROJECTILE_RADIUS,
    PROJECTILE_SPEED,
    SALVAGE_PICKUP_RADIUS,
    components::{
        CollectedSalvage,
        CurrentStation,
        DestroyedModule,
        HostileTarget,
        HostileTurretPlatform,
        HostileWeaponState,
        Integrity,
        MissionState,
        ModuleCondition,
        ModuleFieldEmitter,
        ModuleRuntimeState,
        PlayerFieldState,
        PlayerShip,
        Projectile,
        ProjectileFaction,
        RuntimeShipModule,
        SalvagePickup,
        SalvageWreck,
        ShipMovementModel,
        ShipPowerModel,
        ShipPowerState,
        ShipRoot,
        ShipWeaponState,
        ShipboardControlState,
        SimPosition,
        SimRotation,
        WeaponModule,
    },
    helpers::{
        Fx,
        angle_from_vector,
        dynamic_field_output,
        field_attenuation,
        fixed_radius_sq,
        fx_from_time_delta,
        is_inside_arena,
        local_field_distance,
        module_condition,
        module_effectiveness,
        render_translation,
        ship_movement_model_with_effective,
        ship_power_model_with_effective,
        spawn_player_projectile,
        spawn_projectile_entity,
    },
};
use crate::ship::ModuleKind;

pub(crate) fn sample_ship_fields(
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    player_query: Single<
        (&CurrentStation, &mut PlayerFieldState),
        With<super::super::components::ShipboardPlayer>,
    >,
) {
    let module_samples: Vec<_> = module_query
        .iter()
        .map(
            |(entity, runtime_module, emitter, integrity, runtime_state, destroyed)| {
                let outputs =
                    dynamic_field_output(emitter, runtime_state, integrity, destroyed.is_some());
                (
                    entity,
                    runtime_module.local_position,
                    outputs.0,
                    outputs.1,
                    outputs.2,
                    destroyed.is_some(),
                )
            },
        )
        .collect();

    for (entity, runtime_module, _, _, mut runtime_state, destroyed) in &mut module_query {
        if destroyed.is_some() {
            runtime_state.sampled_heat = Fx::from_num(0);
            runtime_state.sampled_electrical = Fx::from_num(0);
            continue;
        }

        let mut heat = Fx::from_num(0);
        let mut electrical = Fx::from_num(0);
        for (
            source_entity,
            source_pos,
            source_heat,
            source_cooling,
            source_electrical,
            source_destroyed,
        ) in &module_samples
        {
            if *source_destroyed || *source_entity == entity {
                continue;
            }
            let attenuation = field_attenuation(local_field_distance(
                runtime_module.local_position,
                *source_pos,
            ));
            if attenuation <= Fx::from_num(0) {
                continue;
            }
            heat += (*source_heat - *source_cooling) * attenuation;
            electrical += *source_electrical * attenuation;
        }
        runtime_state.sampled_heat = heat.max(Fx::from_num(0));
        runtime_state.sampled_electrical = electrical.max(Fx::from_num(0));
    }

    let (station, mut player_fields) = player_query.into_inner();
    let Some(player_pos) = module_query
        .iter()
        .find(|(_, runtime_module, _, _, _, _)| runtime_module.module_id == station.module_id)
        .map(|(_, runtime_module, _, _, _, _)| runtime_module.local_position)
    else {
        return;
    };

    let mut heat = Fx::from_num(0);
    let mut electrical = Fx::from_num(0);
    for (_, source_pos, source_heat, source_cooling, source_electrical, source_destroyed) in
        &module_samples
    {
        if *source_destroyed {
            continue;
        }
        let attenuation = field_attenuation(local_field_distance(player_pos, *source_pos));
        if attenuation <= Fx::from_num(0) {
            continue;
        }
        heat += (*source_heat - *source_cooling) * attenuation;
        electrical += *source_electrical * attenuation;
    }

    player_fields.local_heat = heat.max(Fx::from_num(0));
    player_fields.local_electrical = electrical.max(Fx::from_num(0));
    player_fields.heat_danger = player_fields.local_heat >= Fx::from_num(8);
    player_fields.electrical_danger = player_fields.local_electrical >= Fx::from_num(7);
}

pub(crate) fn update_module_runtime_state(
    time: Res<Time>,
    mut module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);

    for (runtime_module, integrity, mut runtime_state, destroyed) in &mut module_query {
        if destroyed.is_some() || integrity.current <= 0 {
            runtime_state.is_disabled = true;
            continue;
        }

        let kind_heat = match runtime_module.kind {
            ModuleKind::Reactor => Fx::from_num(1.4),
            ModuleKind::Engine => Fx::from_num(0.8),
            ModuleKind::Turret => Fx::from_num(0.6),
            ModuleKind::Battery => Fx::from_num(0.5),
            _ => Fx::from_num(0.2),
        };
        let damage_factor = Fx::from_num(1)
            - Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
        runtime_state.last_interaction_age += dt;
        runtime_state.current_heat = (runtime_state.current_heat
            + (kind_heat + runtime_state.sampled_heat * Fx::from_num(0.25) + damage_factor) * dt
            - Fx::from_num(0.55) * dt)
            .max(Fx::from_num(0));
        runtime_state.electrical_instability = (runtime_state.electrical_instability
            + (runtime_state.sampled_electrical * Fx::from_num(0.22)
                + damage_factor * Fx::from_num(0.8))
                * dt
            - Fx::from_num(0.35) * dt)
            .max(Fx::from_num(0));

        runtime_state.needs_attention = runtime_state.current_heat >= Fx::from_num(9)
            || runtime_state.electrical_instability >= Fx::from_num(8)
            || integrity.current < integrity.max;
        runtime_state.is_disabled = runtime_state.current_heat >= Fx::from_num(16)
            || runtime_state.electrical_instability >= Fx::from_num(14);
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
        mission_state
            .return_delay_remaining
            .get_or_insert(Fx::from_num(2.5));
        return;
    }

    if hostile_query.is_empty() {
        mission_state.encounter_cleared = true;
        mission_state.completed = mission_state.salvage_collected;
        if mission_state.completion_reason.is_none() {
            mission_state.completion_reason = Some("Encounter cleared".to_string());
        }
        if mission_state.completed {
            mission_state
                .return_delay_remaining
                .get_or_insert(Fx::from_num(2.5));
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
    player_ship_query: Single<
        (&SimPosition, &mut MissionState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    salvage_query: Query<
        (Entity, &SimPosition, &SalvagePickup),
        (With<SalvageWreck>, Without<CollectedSalvage>),
    >,
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
    module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
        Option<&WeaponModule>,
    )>,
) {
    let (children, mut movement_model, mut power_model, mut weapon_state, mut mission_state) =
        player_ship_query.into_inner();

    let mut live_modules = 0usize;
    let mut engine_count = 0u32;
    let mut reactor_count = 0u32;
    let mut battery_count = 0u32;
    let mut turret_count = 0u32;
    let mut core_alive = false;
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
            ModuleKind::Core => core_alive = true,
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

    *movement_model =
        ship_movement_model_with_effective(live_modules.max(1), engine_count, effective_engines);
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
    );
    weapon_state.turret_count = effective_turrets.to_num::<u32>();
    if weapon_state.turret_count == 0 {
        weapon_state.cooldown_remaining = Fx::from_num(0);
    }

    if !core_alive {
        mission_state.failed = true;
        mission_state.failure_reason = Some("Core destroyed".to_string());
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state
            .return_delay_remaining
            .get_or_insert(Fx::from_num(2.5));
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
            format!(
                "Recovered {} scrap from the wreck.",
                mission_state.salvage_scrap_awarded
            )
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

pub(crate) fn fire_player_weapons(
    mut commands: Commands,
    control_mode_query: Single<&ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    player_ship_query: Single<
        (
            &Children,
            &SimPosition,
            &SimRotation,
            &super::super::components::ShipControlState,
            &ShipPowerState,
            &mut ShipWeaponState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    weapon_query: Query<
        (
            &RuntimeShipModule,
            &ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
) {
    if control_mode_query.into_inner().mode != super::super::components::ShipControlMode::ShipFlight
    {
        return;
    }

    let (children, ship_position, ship_rotation, control_state, power_state, mut weapon_state) =
        player_ship_query.into_inner();

    if !control_state.fire_pressed
        || !power_state.weapons_powered
        || weapon_state.turret_count == 0
        || weapon_state.cooldown_remaining > Fx::from_num(0)
    {
        return;
    }

    let ship_forward = super::super::helpers::facing_vector(ship_rotation.radians);
    let projectile_velocity = ship_forward * Fx::from_num(PROJECTILE_SPEED);
    if projectile_velocity.is_near_zero() {
        return;
    }

    let muzzle_offset = ship_forward * Fx::from_num(TILE_SIZE * 0.35);
    let mut fired_any = false;
    for child in children.iter() {
        let Ok((weapon_module, runtime_state, destroyed)) = weapon_query.get(*child) else {
            continue;
        };
        if destroyed.is_some() || runtime_state.is_disabled {
            continue;
        }
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
                        runtime_state.current_heat += Fx::from_num(2);
                        runtime_state.electrical_instability += Fx::from_num(2);
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

pub(crate) fn update_destroyed_module_visuals(
    mut module_query: Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&DestroyedModule>,
            &mut Sprite,
            &mut Visibility,
        ),
        Or<(Changed<Integrity>, Changed<ModuleRuntimeState>)>,
    >,
) {
    for (runtime_module, integrity, runtime_state, destroyed, mut sprite, mut visibility) in
        &mut module_query
    {
        let condition = module_condition(integrity, runtime_state, destroyed.is_some());
        if condition == ModuleCondition::Destroyed {
            sprite.color = Color::srgba(0.28, 0.08, 0.08, 0.12);
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;
        sprite.color = match condition {
            ModuleCondition::Healthy => Color::WHITE,
            ModuleCondition::Degraded => Color::srgb(1.0, 0.88, 0.44),
            ModuleCondition::Disabled => Color::srgb(0.96, 0.50, 0.22),
            ModuleCondition::Destroyed => Color::WHITE,
        };

        if matches!(
            runtime_module.kind,
            ModuleKind::Hull | ModuleKind::HullCorner
        ) {
            sprite.color = match condition {
                ModuleCondition::Healthy => Color::WHITE,
                ModuleCondition::Degraded => Color::srgb(0.98, 0.78, 0.62),
                ModuleCondition::Disabled => Color::srgb(0.88, 0.48, 0.32),
                ModuleCondition::Destroyed => Color::WHITE,
            };
        }
    }
}
