use std::collections::HashMap;

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            ArenaBackdropLayer,
            CurrentStation,
            DestroyedModule,
            EngineFlameOverlay,
            EquippedSuit,
            EvaThrusterOverlay,
            HeldInteraction,
            InfrastructureRouteKind,
            Integrity,
            InteractionKind,
            ManipulatorModule,
            ModuleCondition,
            ModuleFieldEmitter,
            ModuleRuntimeState,
            ModuleWorkEffect,
            ModuleWorkProgressFill,
            ModuleWorkProgressRoot,
            ObservedLocalPlayerMarker,
            PlayerMotionState,
            PlayerReferenceFrame,
            PlayerShip,
            PlayerSuit,
            ReactorCommandState,
            ReactorGlowOverlay,
            ResourceKind,
            RuntimeFoundationVisual,
            RuntimeShipModule,
            ShipControlState,
            ShipInfrastructureState,
            ShipPowerState,
            ShipRoot,
            ShipboardPlayer,
            SimPosition,
            SimRotation,
            TurretCommandState,
            TurretTopSprite,
        },
        effects::{EngineFlameMaterial, ReactorGlowMaterial},
        helpers::{Fx, module_condition},
    },
    ship::ModuleKind,
    state::GameplayInfoPanelMode,
};

/// Draws contextual tactical overlays for the currently focused station or module.
/// Draws debug-only tactical overlays around the focused module so tuning field and manipulator ranges is easier.
pub(crate) fn draw_debug_overlay(
    hud_mode: Res<GameplayInfoPanelMode>,
    ship_query: Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            Option<&ShipInfrastructureState>,
        ),
        With<ShipRoot>,
    >,
    player_ship_query: Single<
        (Entity, &SimPosition, &SimRotation),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<&CurrentStation, With<ObservedLocalPlayerMarker>>,
    foundation_query: Query<(&RuntimeFoundationVisual, &ChildOf)>,
    module_query: Query<(
        Entity,
        &ChildOf,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut turret_top_query: Query<(&ChildOf, &mut Transform), With<TurretTopSprite>>,
    mut gizmos: Gizmos,
) {
    let (player_ship_entity, ship_position, ship_rotation) = player_ship_query.into_inner();
    let current_station = player_query.into_inner();
    update_turret_top_visuals(ship_rotation.radians, &module_query, &mut turret_top_query);

    if *hud_mode == GameplayInfoPanelMode::Tubes {
        draw_tubes_overlay(&ship_query, &foundation_query, &module_query, &mut gizmos);
        return;
    }

    if *hud_mode != GameplayInfoPanelMode::FocusedModule {
        return;
    }

    let focused_module_id = current_station.module_id;

    let field_radius = TILE_SIZE * 3.5;
    let manipulator_radius = TILE_SIZE * 2.5;

    for (_, parent, runtime_module, emitter, manipulator, _, destroyed) in &module_query {
        if parent.get() != player_ship_entity {
            continue;
        }
        if runtime_module.module_id != focused_module_id {
            continue;
        }
        if destroyed.is_some() {
            continue;
        }
        let world =
            ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
        let world_center = world.to_vec2();
        if emitter.heat_output > Fx::from_num(0) || emitter.cooling_output > Fx::from_num(0) {
            gizmos.circle_2d(
                world_center,
                field_radius,
                Color::srgba(1.0, 0.58, 0.24, 0.18),
            );
        }
        if emitter.electrical_output > Fx::from_num(0) || emitter.grounding_output > Fx::from_num(0)
        {
            gizmos.circle_2d(
                world_center,
                field_radius * 0.72,
                Color::srgba(0.32, 0.78, 1.0, 0.18),
            );
        }
        if manipulator.is_some() {
            gizmos.circle_2d(
                world_center,
                manipulator_radius,
                Color::srgba(0.72, 1.0, 0.58, 0.18),
            );
        }
    }

    let module_positions: Vec<_> = module_query
        .iter()
        .map(|(entity, parent, runtime_module, _, _, _, destroyed)| {
            (
                entity,
                parent.get(),
                runtime_module.module_id,
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians),
                destroyed.is_some(),
            )
        })
        .collect();

    for (_, parent, _, _, manipulator, _, destroyed) in &module_query {
        if parent.get() != player_ship_entity {
            continue;
        }
        let Some(manipulator) = manipulator else {
            continue;
        };
        if manipulator.source_module_id != Some(focused_module_id)
            && manipulator.target_module_id != Some(focused_module_id)
        {
            continue;
        }
        if destroyed.is_some() || !manipulator.active {
            continue;
        }
        let Some(source_id) = manipulator.source_module_id else {
            continue;
        };
        let Some(target_id) = manipulator.target_module_id else {
            continue;
        };
        let source = module_positions
            .iter()
            .find(|(_, parent, module_id, _, is_destroyed)| {
                *parent == player_ship_entity && *module_id == source_id && !*is_destroyed
            })
            .map(|(_, _, _, pos, _)| *pos);
        let target = module_positions
            .iter()
            .find(|(_, parent, module_id, _, is_destroyed)| {
                *parent == player_ship_entity && *module_id == target_id && !*is_destroyed
            })
            .map(|(_, _, _, pos, _)| *pos);
        if let (Some(source), Some(target)) = (source, target) {
            gizmos.line_2d(
                source.to_vec2(),
                target.to_vec2(),
                Color::srgb(0.72, 1.0, 0.58),
            );
        }
    }
}

fn draw_tubes_overlay(
    ship_query: &Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            Option<&ShipInfrastructureState>,
        ),
        With<ShipRoot>,
    >,
    foundation_query: &Query<(&RuntimeFoundationVisual, &ChildOf)>,
    module_query: &Query<(
        Entity,
        &ChildOf,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    gizmos: &mut Gizmos,
) {
    for (ship_entity, ship_position, ship_rotation, infrastructure) in ship_query {
        let Some(infrastructure) = infrastructure else {
            continue;
        };
        let grid_origin = ship_grid_origin(ship_entity, module_query);

        for network in &infrastructure.networks {
            let color = network
                .kind
                .map(tube_color)
                .unwrap_or(Color::srgba(0.85, 0.85, 0.85, 0.45));
            for tile in &network.tiles {
                let center =
                    ship_grid_to_world(*tile, grid_origin, ship_position, ship_rotation).to_vec2();
                gizmos.rect_2d(center, Vec2::splat(TILE_SIZE * 0.72), color);

                for neighbor in [(tile.0 + 1, tile.1), (tile.0, tile.1 + 1)] {
                    if network.tiles.contains(&neighbor) {
                        let other =
                            ship_grid_to_world(neighbor, grid_origin, ship_position, ship_rotation)
                                .to_vec2();
                        gizmos.line_2d(center, other, color);
                    }
                }
            }
        }

        for (foundation, parent) in foundation_query {
            if parent.get() != ship_entity {
                continue;
            }
            let Some(kind) = infrastructure_route_kind(foundation.kind) else {
                continue;
            };
            let center = ship_grid_to_world(
                (foundation.grid_x, foundation.grid_y),
                grid_origin,
                ship_position,
                ship_rotation,
            )
            .to_vec2();
            gizmos.circle_2d(center, TILE_SIZE * 0.16, tube_color(kind));
        }

        for (_, parent, runtime_module, _, _, _, destroyed) in module_query {
            if parent.get() != ship_entity || destroyed.is_some() {
                continue;
            }
            let Some(status) = infrastructure.status_for_module(runtime_module.module_id) else {
                continue;
            };
            let world =
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
            let color = if status.blocked_reason.is_some() {
                Color::srgb(0.96, 0.24, 0.18)
            } else if module_is_producer(runtime_module.kind) {
                Color::srgb(0.30, 0.95, 0.45)
            } else if status.power_required || !status.resource_networks.is_empty() {
                Color::srgb(1.0, 0.74, 0.26)
            } else {
                Color::srgba(0.72, 0.80, 0.90, 0.42)
            };
            gizmos.circle_2d(world.to_vec2(), TILE_SIZE * 0.34, color);
        }
    }
}

fn ship_grid_origin(
    ship_entity: Entity,
    module_query: &Query<(
        Entity,
        &ChildOf,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
) -> crate::gameplay::helpers::FixedVec2 {
    module_query
        .iter()
        .find_map(|(_, parent, runtime_module, _, _, _, _)| {
            (parent.get() == ship_entity).then(|| {
                runtime_module.local_position
                    - crate::gameplay::helpers::FixedVec2::from_num(
                        runtime_module.grid_x * TILE_SIZE as i32,
                        -runtime_module.grid_y * TILE_SIZE as i32,
                    )
            })
        })
        .unwrap_or_else(crate::gameplay::helpers::FixedVec2::zero)
}

fn ship_grid_to_world(
    (grid_x, grid_y): (i32, i32),
    grid_origin: crate::gameplay::helpers::FixedVec2,
    ship_position: &SimPosition,
    ship_rotation: &SimRotation,
) -> crate::gameplay::helpers::FixedVec2 {
    let local = crate::gameplay::helpers::FixedVec2::from_num(
        grid_x * TILE_SIZE as i32,
        -grid_y * TILE_SIZE as i32,
    );
    ship_position.value + (grid_origin + local).rotate(ship_rotation.radians)
}

fn infrastructure_route_kind(
    kind: crate::ship::ShipFoundationKind,
) -> Option<InfrastructureRouteKind> {
    match kind {
        crate::ship::ShipFoundationKind::Wire => Some(InfrastructureRouteKind::Power),
        crate::ship::ShipFoundationKind::OxygenDuct => Some(InfrastructureRouteKind::OxygenDuct),
        crate::ship::ShipFoundationKind::PipeRawSalvage => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::RawSalvage))
        }
        crate::ship::ShipFoundationKind::PipeRepairCharge => Some(
            InfrastructureRouteKind::Resource(ResourceKind::RepairCharge),
        ),
        crate::ship::ShipFoundationKind::PipeFuel => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::Fuel))
        }
        crate::ship::ShipFoundationKind::PipeAmmunition => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::Ammunition))
        }
        crate::ship::ShipFoundationKind::PipeOxygen => {
            Some(InfrastructureRouteKind::Resource(ResourceKind::Oxygen))
        }
        _ => None,
    }
}

fn tube_color(kind: InfrastructureRouteKind) -> Color {
    match kind {
        InfrastructureRouteKind::Power => Color::srgba(1.0, 0.90, 0.22, 0.72),
        InfrastructureRouteKind::OxygenDuct => Color::srgba(0.36, 0.78, 1.0, 0.72),
        InfrastructureRouteKind::Resource(ResourceKind::RawSalvage) => {
            Color::srgba(0.72, 0.78, 0.86, 0.70)
        }
        InfrastructureRouteKind::Resource(ResourceKind::RepairCharge) => {
            Color::srgba(0.36, 1.0, 0.58, 0.70)
        }
        InfrastructureRouteKind::Resource(ResourceKind::Fuel) => {
            Color::srgba(1.0, 0.48, 0.20, 0.70)
        }
        InfrastructureRouteKind::Resource(ResourceKind::Ammunition) => {
            Color::srgba(0.92, 0.28, 0.24, 0.70)
        }
        InfrastructureRouteKind::Resource(ResourceKind::Oxygen) => {
            Color::srgba(0.48, 0.96, 1.0, 0.70)
        }
    }
}

fn module_is_producer(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Reactor | ModuleKind::Battery | ModuleKind::Cargo | ModuleKind::O2Generator
    )
}

/// Tints or hides module sprites to reflect heat, instability, disablement, and destruction.
/// Tints destroyed or impaired ship visuals so players can read damage state directly from the scene.
pub(crate) fn update_destroyed_module_visuals(
    balance: Res<BalanceConfig>,
    mut visuals: ParamSet<(
        Query<'_, '_, (&'static RuntimeFoundationVisual, &'static mut Sprite)>,
        Query<
            '_,
            '_,
            (
                &'static RuntimeShipModule,
                &'static Integrity,
                &'static ModuleRuntimeState,
                Option<&'static DestroyedModule>,
                &'static mut Sprite,
                &'static mut Visibility,
            ),
        >,
    )>,
) {
    // SAFETY: Foundation visuals and runtime module sprites are distinct entity sets updated in separate
    // `ParamSet` branches, and the read branch is consumed before mutable module sprite access.
    let logistics_support_cells = visuals
        .p0()
        .iter()
        .filter_map(|(tile, _)| {
            (!matches!(
                tile.kind,
                crate::ship::ShipFoundationKind::Hull
                    | crate::ship::ShipFoundationKind::HullInnerCorner
                    | crate::ship::ShipFoundationKind::HullOuterCorner
            ))
            .then_some((tile.grid_x, tile.grid_y))
        })
        .collect::<Vec<_>>();
    let mut logistics_ghost_cells = Vec::new();

    for (runtime_module, integrity, runtime_state, destroyed, mut sprite, mut visibility) in
        &mut visuals.p1()
    {
        let condition = module_condition(integrity, runtime_state, destroyed.is_some(), &balance);
        if condition == ModuleCondition::Destroyed {
            let is_hull_fixture = matches!(
                runtime_module.kind,
                ModuleKind::Airlock | ModuleKind::Engine | ModuleKind::Turret
            );
            if is_hull_fixture {
                let has_logistics_support = logistics_support_cells
                    .contains(&(runtime_module.grid_x, runtime_module.grid_y));
                if has_logistics_support {
                    logistics_ghost_cells.push((runtime_module.grid_x, runtime_module.grid_y));
                    *visibility = Visibility::Hidden;
                } else {
                    sprite.color = Color::srgba(0.92, 0.70, 0.54, 0.28);
                    *visibility = Visibility::Visible;
                }
            } else {
                sprite.color = Color::srgba(0.66, 0.74, 0.82, 0.24);
                *visibility = Visibility::Visible;
            }
            continue;
        }

        *visibility = Visibility::Visible;
        let hot =
            runtime_state.current_heat >= Fx::from_num(balance.fields.degraded_heat_threshold);
        let electrical = runtime_state.electrical_instability
            >= Fx::from_num(balance.fields.degraded_electrical_threshold);
        sprite.color = match condition {
            ModuleCondition::Healthy => Color::WHITE,
            ModuleCondition::Degraded if hot && electrical => Color::srgb(0.96, 0.52, 0.90),
            ModuleCondition::Degraded if hot => Color::srgb(1.0, 0.80, 0.34),
            ModuleCondition::Degraded if electrical => Color::srgb(0.42, 0.86, 1.0),
            ModuleCondition::Degraded => Color::srgb(1.0, 0.88, 0.44),
            ModuleCondition::Disabled if hot && electrical => Color::srgb(0.88, 0.22, 0.72),
            ModuleCondition::Disabled if hot => Color::srgb(0.96, 0.50, 0.22),
            ModuleCondition::Disabled if electrical => Color::srgb(0.18, 0.72, 0.96),
            ModuleCondition::Disabled => Color::srgb(0.96, 0.50, 0.22),
            ModuleCondition::Destroyed => Color::WHITE,
        };

        if matches!(
            runtime_module.kind,
            ModuleKind::Hull
                | ModuleKind::HullInnerCorner
                | ModuleKind::HullOuterCorner
                | ModuleKind::Airlock
        ) {
            sprite.color = match condition {
                ModuleCondition::Healthy => Color::WHITE,
                ModuleCondition::Degraded if electrical => Color::srgb(0.62, 0.88, 0.98),
                ModuleCondition::Degraded => Color::srgb(0.98, 0.78, 0.62),
                ModuleCondition::Disabled if electrical => Color::srgb(0.44, 0.72, 0.92),
                ModuleCondition::Disabled => Color::srgb(0.88, 0.48, 0.32),
                ModuleCondition::Destroyed => Color::WHITE,
            };
        }
    }

    for (tile, mut sprite) in &mut visuals.p0() {
        if matches!(
            tile.kind,
            crate::ship::ShipFoundationKind::Hull
                | crate::ship::ShipFoundationKind::HullInnerCorner
                | crate::ship::ShipFoundationKind::HullOuterCorner
        ) {
            sprite.color = Color::WHITE;
            continue;
        }
        if logistics_ghost_cells.contains(&(tile.grid_x, tile.grid_y)) {
            sprite.color = Color::srgba(0.72, 0.80, 0.88, 0.26);
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

/// Drives the reactor's front-layer presentation glow from its live runtime state.
pub(crate) fn sync_reactor_glow_visuals(
    time: Res<Time>,
    mut reactor_materials: ResMut<Assets<ReactorGlowMaterial>>,
    module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleRuntimeState,
        Option<&ReactorCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut reactor_glow_query: Query<
        (
            &ChildOf,
            &MeshMaterial2d<ReactorGlowMaterial>,
            &mut Visibility,
            &mut Transform,
        ),
        (
            With<ReactorGlowOverlay>,
            Without<EngineFlameOverlay>,
            Without<ModuleWorkEffect>,
            Without<ModuleWorkProgressRoot>,
            Without<ModuleWorkProgressFill>,
            Without<EvaThrusterOverlay>,
        ),
    >,
) {
    let pulse_phase = time.elapsed_secs_wrapped() * 5.2;

    for (parent, material_handle, mut visibility, mut transform) in &mut reactor_glow_query {
        let Ok((_, _, runtime_state, reactor, destroyed)) = module_query.get(parent.get()) else {
            continue;
        };
        if destroyed.is_some() {
            *visibility = Visibility::Hidden;
            continue;
        }
        let Some(reactor) = reactor else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let intensity = ((reactor.reaction_rate * Fx::from_num(0.45))
            + (reactor.power_output / Fx::from_num(20)) * Fx::from_num(0.25)
            + (runtime_state.current_heat / Fx::from_num(16)) * Fx::from_num(0.20)
            + Fx::from_num(0.10))
        .clamp(Fx::from_num(0), Fx::from_num(1));
        let pulse = 0.5 + 0.5 * pulse_phase.sin();
        let alpha = (0.28 + intensity.to_num::<f32>() * (0.46 + pulse * 0.20)).clamp(0.30, 0.86);
        if let Some(material) = reactor_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.intensity = intensity.to_num::<f32>();
            material.params.alpha = alpha;
        }
        transform.scale = Vec3::splat(1.12 + intensity.to_num::<f32>() * 0.34 + pulse * 0.08);
        *visibility = Visibility::Visible;
    }
}

/// Shows and scales engine exhaust only while the player's ship is actually under thrust.
pub(crate) fn sync_engine_flame_visuals(
    time: Res<Time>,
    ship_query: Single<(&ShipControlState, &ShipPowerState), (With<PlayerShip>, With<ShipRoot>)>,
    mut engine_materials: ResMut<Assets<EngineFlameMaterial>>,
    mut flame_growth: Local<f32>,
    mut engine_flame_query: Query<
        (
            &ChildOf,
            &MeshMaterial2d<EngineFlameMaterial>,
            &mut Visibility,
            &mut Transform,
        ),
        (
            With<EngineFlameOverlay>,
            Without<ReactorGlowOverlay>,
            Without<ModuleWorkEffect>,
            Without<ModuleWorkProgressRoot>,
            Without<ModuleWorkProgressFill>,
            Without<EvaThrusterOverlay>,
        ),
    >,
) {
    let (ship_controls, ship_power) = ship_query.into_inner();
    let engine_alpha = if ship_controls.thrust_active && ship_power.engines_powered {
        ((ship_power.engine_power_ratio * Fx::from_num(0.85)) + Fx::from_num(0.15))
            .clamp(Fx::from_num(0), Fx::from_num(1))
            .to_num::<f32>()
    } else {
        0.0
    };
    if engine_alpha <= 0.01 {
        *flame_growth = 0.0;
    } else {
        *flame_growth =
            (*flame_growth + time.delta_secs() * (0.9 + engine_alpha * 1.1)).clamp(0.0, 1.0);
    }
    for (_parent, material_handle, mut visibility, mut transform) in &mut engine_flame_query {
        if engine_alpha <= 0.01 {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        if let Some(material) = engine_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.growth = *flame_growth;
            material.params.intensity = engine_alpha;
            material.params.alpha = 0.30 + engine_alpha * 0.66;
        }
        transform.scale = Vec3::new(1.0, 0.92 + engine_alpha * 1.34, 1.0);
    }
}

/// Syncs repair/extraction sparks and progress bars to the player's current held interaction.
pub(crate) fn sync_module_work_effect_visuals(
    time: Res<Time>,
    player_query: Query<
        (&PlayerMotionState, &EquippedSuit, &HeldInteraction),
        With<ShipboardPlayer>,
    >,
    mut work_visuals: ParamSet<(
        Query<
            '_,
            '_,
            (
                &'static ChildOf,
                &'static mut Sprite,
                &'static mut Visibility,
                &'static mut Transform,
            ),
            With<ModuleWorkEffect>,
        >,
        Query<'_, '_, (&'static ChildOf, &'static mut Visibility), With<ModuleWorkProgressRoot>>,
        Query<
            '_,
            '_,
            (
                &'static ChildOf,
                &'static mut Sprite,
                &'static mut Visibility,
                &'static mut Transform,
            ),
            With<ModuleWorkProgressFill>,
        >,
    )>,
) {
    // SAFETY: Work spark, root, and fill entities carry distinct marker components and are spawned as
    // separate children; each `ParamSet` branch mutates only one visual role at a time.
    let active_work = collect_active_work(&player_query);

    for (parent, mut sprite, mut visibility, mut transform) in &mut work_visuals.p0() {
        let Some((kind, progress)) = active_work.get(&parent.get()).copied() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = Visibility::Visible;
        let base = match kind {
            InteractionKind::Repair => Color::srgba(0.36, 0.94, 0.74, 0.65),
            InteractionKind::Extract => Color::srgba(1.0, 0.72, 0.28, 0.65),
            _ => Color::srgba(1.0, 1.0, 1.0, 0.0),
        };
        sprite.color =
            base.with_alpha(0.28 + (time.elapsed_secs_wrapped() * 16.0).sin().abs() * 0.45);
        transform.translation.x = (time.elapsed_secs_wrapped() * 18.0).sin() * 4.0;
        transform.translation.y = (time.elapsed_secs_wrapped() * 13.0).cos() * 4.0;
        transform.scale = Vec3::splat(0.85 + progress.to_num::<f32>() * 0.4);
    }

    for (parent, mut visibility) in &mut work_visuals.p1() {
        *visibility = if active_work.contains_key(&parent.get()) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (parent, mut sprite, mut visibility, mut transform) in &mut work_visuals.p2() {
        let Some((kind, progress)) = active_work.get(&parent.get()).copied() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let width = (20.0 * progress.to_num::<f32>()).clamp(0.5, 20.0);
        sprite.custom_size = Some(Vec2::new(width, 2.0));
        sprite.color = match kind {
            InteractionKind::Repair => Color::srgb(0.36, 0.92, 0.72),
            InteractionKind::Extract => Color::srgb(1.0, 0.70, 0.26),
            _ => Color::WHITE,
        };
        transform.translation.x = -10.0 + width * 0.5;
        *visibility = Visibility::Visible;
    }
}

/// Shows suit thrusters while an EVA-suited actor is accelerating in open space.
pub(crate) fn sync_eva_thruster_visuals(
    player_query: Query<
        (&PlayerMotionState, &EquippedSuit, &HeldInteraction),
        With<ShipboardPlayer>,
    >,
    mut eva_thruster_query: Query<
        (&ChildOf, &EvaThrusterOverlay, &mut Sprite, &mut Visibility),
        (
            Without<ReactorGlowOverlay>,
            Without<EngineFlameOverlay>,
            Without<ModuleWorkEffect>,
            Without<ModuleWorkProgressRoot>,
            Without<ModuleWorkProgressFill>,
        ),
    >,
) {
    for (parent, overlay, mut sprite, mut visibility) in &mut eva_thruster_query {
        let Ok((motion, suit, _)) = player_query.get(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let active = suit.suit == PlayerSuit::Eva
            && matches!(motion.frame, PlayerReferenceFrame::World)
            && motion.world_velocity.length() > Fx::from_num(0.6);
        if !active {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let side_boost = if overlay.side < 0 { 0.92 } else { 0.78 };
        sprite.color = Color::srgba(0.58, 0.88, 1.0, 0.32 + side_boost * 0.44);
    }
}

fn collect_active_work(
    player_query: &Query<
        (&PlayerMotionState, &EquippedSuit, &HeldInteraction),
        With<ShipboardPlayer>,
    >,
) -> HashMap<Entity, (InteractionKind, Fx)> {
    let mut active_work = HashMap::new();
    for (_motion, _suit, held) in player_query {
        if let (Some(target), Some(kind)) = (held.target, held.kind) {
            if !matches!(kind, InteractionKind::Repair | InteractionKind::Extract) {
                continue;
            }
            let progress = if held.required > Fx::from_num(0) {
                (held.progress / held.required).clamp(Fx::from_num(0), Fx::from_num(1))
            } else {
                Fx::from_num(0)
            };
            active_work
                .entry(target)
                .and_modify(
                    |(existing_kind, existing_progress): &mut (InteractionKind, Fx)| {
                        if progress > *existing_progress {
                            *existing_kind = kind;
                            *existing_progress = progress;
                        }
                    },
                )
                .or_insert((kind, progress));
        }
    }
    active_work
}

/// Applies parallax offsets to the combat backdrop based on the camera position.
/// Applies a small camera-relative parallax drift to backdrop layers so the arena feels deeper in motion.
pub(crate) fn sync_backdrop_parallax(
    camera_query: Single<&Transform, With<Camera2d>>,
    mut backdrop_query: Query<(&ArenaBackdropLayer, &mut Transform), Without<Camera2d>>,
) {
    let camera = camera_query.into_inner();
    for (layer, mut transform) in &mut backdrop_query {
        transform.translation.x = layer.base_translation.x + camera.translation.x * layer.depth;
        transform.translation.y = layer.base_translation.y + camera.translation.y * layer.depth;
        transform.translation.z = layer.base_translation.z;
    }
}

fn update_turret_top_visuals(
    _ship_rotation: Fx,
    module_query: &Query<(
        Entity,
        &ChildOf,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    turret_top_query: &mut Query<(&ChildOf, &mut Transform), With<TurretTopSprite>>,
) {
    for (parent, mut transform) in turret_top_query.iter_mut() {
        let parent_entity = parent.get();
        let Ok((_, _, runtime_module, _, _, turret_state, destroyed)) =
            module_query.get(parent_entity)
        else {
            continue;
        };
        if destroyed.is_some() || runtime_module.kind != ModuleKind::Turret {
            continue;
        }
        let actual_local_angle = turret_state
            .map(|state| state.actual_angle)
            .unwrap_or(Fx::from_num(0));
        let base_rotation = -Fx::from_num(runtime_module.rotation_quadrants as i32) * Fx::FRAC_PI_2;
        transform.rotation =
            Quat::from_rotation_z((actual_local_angle - base_rotation).to_num::<f32>());
    }
}
