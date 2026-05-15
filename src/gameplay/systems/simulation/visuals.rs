use std::collections::HashMap;

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            ArenaBackdropLayer,
            BatteryPulseOverlay,
            CurrentStation,
            DecompressionAirLinesOverlay,
            DestroyedModule,
            ElectricalArcOverlay,
            EngineFlameOverlay,
            EquippedSuit,
            EvaThrusterOverlay,
            FabricatorDustOverlay,
            HeatFlameOverlay,
            HeldInteraction,
            InfrastructureRouteKind,
            Integrity,
            InteractionKind,
            LinearVelocity,
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
            ProcessorModule,
            ReactorCommandState,
            ReactorGlowOverlay,
            ResourceKind,
            RuntimeFoundationVisual,
            RuntimeShipModule,
            ServiceLinkOverlay,
            ShipAtmosphereState,
            ShipControlState,
            ShipInfrastructureState,
            ShipPowerState,
            ShipRoot,
            ShipSpeedLinesOverlay,
            ShipboardPlayer,
            SimPosition,
            SimRotation,
            SpaceBackdropLayer,
            TurretCommandState,
            TurretFlashOverlay,
            TurretFlashPulse,
            TurretTopSprite,
        },
        effects::{
            AirLinesMaterial,
            BatteryPulseMaterial,
            ElectricArcsMaterial,
            EngineFlameMaterial,
            FabricatorDustMaterial,
            ReactorGlowMaterial,
            SmallFlamesMaterial,
            SpaceBackdropMaterial,
            SpeedLinesMaterial,
            TurretFlashMaterial,
        },
        helpers::{Fx, component_service_coords, module_condition},
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
            if status.power_required || !status.service_statuses.is_empty() {
                for coord in
                    component_service_coords((runtime_module.grid_x, runtime_module.grid_y))
                {
                    let center =
                        ship_grid_to_world(coord, grid_origin, ship_position, ship_rotation)
                            .to_vec2();
                    gizmos.rect_2d(
                        center,
                        Vec2::splat(TILE_SIZE * 0.88),
                        Color::srgba(0.82, 0.92, 1.0, 0.18),
                    );
                }
            }
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
) -> crate::helpers::FixedVec2 {
    module_query
        .iter()
        .find_map(|(_, parent, runtime_module, _, _, _, _)| {
            (parent.get() == ship_entity).then(|| {
                runtime_module.local_position
                    - crate::helpers::FixedVec2::from_num(
                        runtime_module.grid_x * TILE_SIZE as i32,
                        -runtime_module.grid_y * TILE_SIZE as i32,
                    )
            })
        })
        .unwrap_or_else(crate::helpers::FixedVec2::zero)
}

fn ship_grid_to_world(
    (grid_x, grid_y): (i32, i32),
    grid_origin: crate::helpers::FixedVec2,
    ship_position: &SimPosition,
    ship_rotation: &SimRotation,
) -> crate::helpers::FixedVec2 {
    let local =
        crate::helpers::FixedVec2::from_num(grid_x * TILE_SIZE as i32, -grid_y * TILE_SIZE as i32);
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
        sprite.color = match condition {
            ModuleCondition::Healthy => Color::WHITE,
            ModuleCondition::Degraded => Color::WHITE,
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
                ModuleCondition::Degraded => Color::WHITE,
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

/// Draws subtle gameplay service links from modules to their connected service-port route tiles.
pub(crate) fn sync_service_link_visuals(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ship_query: Query<&ShipInfrastructureState, With<ShipRoot>>,
    module_query: Query<(
        Entity,
        &ChildOf,
        &RuntimeShipModule,
        Option<&DestroyedModule>,
    )>,
    mut link_query: Query<(
        &ChildOf,
        &ServiceLinkOverlay,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    for (_, _, _, _, mut visibility) in &mut link_query {
        *visibility = Visibility::Hidden;
    }

    for (entity, parent, runtime_module, destroyed) in &module_query {
        if destroyed.is_some() {
            continue;
        }
        let Ok(infrastructure) = ship_query.get(parent.get()) else {
            continue;
        };
        let Some(status) = infrastructure.status_for_module(runtime_module.module_id) else {
            continue;
        };
        for service in &status.service_statuses {
            let Some(service_coord) = service.service_coord else {
                continue;
            };
            if !service_link_should_draw(service, destroyed.is_some()) {
                continue;
            }
            let delta = Vec2::new(
                (service_coord.0 - runtime_module.grid_x) as f32 * TILE_SIZE,
                -((service_coord.1 - runtime_module.grid_y) as f32) * TILE_SIZE,
            );
            let direction = if delta.length_squared() <= 0.01 {
                Vec2::Y
            } else {
                delta.normalize()
            };
            let length = if delta.length_squared() <= 0.01 {
                TILE_SIZE * 0.34
            } else {
                TILE_SIZE * 0.58
            };
            let midpoint = direction * (length * 0.5);
            let tint = tube_color(service.route_kind).with_alpha(0.36);
            let mut found = false;
            for (link_parent, link, mut sprite, mut transform, mut visibility) in &mut link_query {
                if link_parent.get() != entity || link.route_kind != service.route_kind {
                    continue;
                }
                sprite.color = tint;
                sprite.custom_size = Some(Vec2::new(8.0, length));
                transform.translation = Vec3::new(midpoint.x, midpoint.y, 0.31);
                transform.rotation = Quat::from_rotation_z(
                    direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2,
                );
                *visibility = Visibility::Visible;
                found = true;
                break;
            }
            if !found {
                let route_kind = service.route_kind;
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Sprite {
                            color: tint,
                            custom_size: Some(Vec2::new(8.0, length)),
                            ..Sprite::from_image(asset_server.load("tiles/service_link.png"))
                        },
                        Transform {
                            translation: Vec3::new(midpoint.x, midpoint.y, 0.31),
                            rotation: Quat::from_rotation_z(
                                direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2,
                            ),
                            ..default()
                        },
                        Visibility::Visible,
                        ServiceLinkOverlay { route_kind },
                    ));
                });
            }
        }
    }
}

/// Lazily creates shader-backed module overlays for transient and hazard presentation effects.
pub(crate) fn spawn_missing_effect_overlays(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay_mesh: Local<Option<Handle<Mesh>>>,
    mut turret_materials: ResMut<Assets<TurretFlashMaterial>>,
    mut battery_materials: ResMut<Assets<BatteryPulseMaterial>>,
    mut dust_materials: ResMut<Assets<FabricatorDustMaterial>>,
    mut arc_materials: ResMut<Assets<ElectricArcsMaterial>>,
    mut flame_materials: ResMut<Assets<SmallFlamesMaterial>>,
    module_query: Query<(Entity, &RuntimeShipModule, Option<&Children>)>,
    turret_flash_query: Query<&ChildOf, With<TurretFlashOverlay>>,
    battery_query: Query<&ChildOf, With<BatteryPulseOverlay>>,
    dust_query: Query<&ChildOf, With<FabricatorDustOverlay>>,
    arc_query: Query<&ChildOf, With<ElectricalArcOverlay>>,
    flame_query: Query<&ChildOf, With<HeatFlameOverlay>>,
) {
    let mesh = overlay_mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(TILE_SIZE * 0.92, TILE_SIZE * 0.92)))
        .clone();
    for (entity, runtime_module, _children) in &module_query {
        if runtime_module.kind == ModuleKind::Turret
            && !turret_flash_query
                .iter()
                .any(|parent| parent.get() == entity)
        {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(turret_materials.add(TurretFlashMaterial::default())),
                    Transform::from_xyz(0.0, -TILE_SIZE * 0.34, 0.34),
                    Visibility::Hidden,
                    TurretFlashOverlay,
                ));
            });
        }
        if runtime_module.kind == ModuleKind::Battery
            && !battery_query.iter().any(|parent| parent.get() == entity)
        {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(battery_materials.add(BatteryPulseMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 0.19),
                    Visibility::Hidden,
                    BatteryPulseOverlay,
                ));
            });
        }
        if runtime_module.kind == ModuleKind::Processor
            && !dust_query.iter().any(|parent| parent.get() == entity)
        {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(dust_materials.add(FabricatorDustMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 0.28),
                    Visibility::Hidden,
                    FabricatorDustOverlay,
                ));
            });
        }
        if !arc_query.iter().any(|parent| parent.get() == entity) {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(arc_materials.add(ElectricArcsMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 0.33),
                    Visibility::Hidden,
                    ElectricalArcOverlay,
                ));
            });
        }
        if !flame_query.iter().any(|parent| parent.get() == entity) {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(flame_materials.add(SmallFlamesMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 0.32),
                    Visibility::Hidden,
                    HeatFlameOverlay,
                ));
            });
        }
    }
}

/// Drives turret muzzle flashes from the short-lived pulse stamped when a projectile is fired.
pub(crate) fn sync_turret_flash_visuals(
    time: Res<Time>,
    mut flash_materials: ResMut<Assets<TurretFlashMaterial>>,
    mut module_query: Query<(
        &RuntimeShipModule,
        Option<&DestroyedModule>,
        Option<&mut TurretFlashPulse>,
    )>,
    mut flash_query: Query<(
        &ChildOf,
        &MeshMaterial2d<TurretFlashMaterial>,
        &mut Visibility,
        &mut Transform,
    )>,
) {
    let dt = Fx::from_num(time.delta_secs());
    for (parent, material_handle, mut visibility, mut transform) in &mut flash_query {
        let Ok((runtime_module, destroyed, pulse)) = module_query.get_mut(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(mut pulse) = pulse else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if destroyed.is_some() || pulse.remaining <= Fx::from_num(0) {
            *visibility = Visibility::Hidden;
            pulse.remaining = Fx::from_num(0);
            continue;
        }
        pulse.remaining = (pulse.remaining - dt).max(Fx::from_num(0));
        let intensity = if pulse.duration > Fx::from_num(0) {
            (pulse.remaining / pulse.duration)
                .clamp(Fx::from_num(0), Fx::from_num(1))
                .to_num::<f32>()
        } else {
            0.0
        };
        if let Some(material) = flash_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.primary_color = Vec4::new(1.0, 0.82, 0.34, 1.0);
            material.params.secondary_color = Vec4::new(1.0, 0.30, 0.12, 1.0);
            material.params.intensity = intensity;
            material.params.alpha = 0.86 * intensity;
        }
        transform.translation.y = -TILE_SIZE * 0.28;
        transform.scale = Vec3::splat(0.72 + intensity * 0.54);
        transform.rotation = Quat::from_rotation_z(
            -(runtime_module.rotation_quadrants as f32) * std::f32::consts::FRAC_PI_2,
        );
        *visibility = Visibility::Visible;
    }
}

/// Pulses batteries according to reserve level and local network flow so stored power feels alive.
pub(crate) fn sync_battery_pulse_visuals(
    time: Res<Time>,
    ship_query: Query<&ShipInfrastructureState, With<ShipRoot>>,
    mut battery_materials: ResMut<Assets<BatteryPulseMaterial>>,
    module_query: Query<(&ChildOf, &RuntimeShipModule, Option<&DestroyedModule>)>,
    mut battery_query: Query<(
        &ChildOf,
        &MeshMaterial2d<BatteryPulseMaterial>,
        &mut Visibility,
        &mut Transform,
    )>,
) {
    for (parent, material_handle, mut visibility, mut transform) in &mut battery_query {
        let Ok((ship_parent, runtime_module, destroyed)) = module_query.get(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if destroyed.is_some() || runtime_module.kind != ModuleKind::Battery {
            *visibility = Visibility::Hidden;
            continue;
        }
        let Ok(infrastructure) = ship_query.get(ship_parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(status) = infrastructure.status_for_module(runtime_module.module_id) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(network) = status
            .power_network
            .and_then(|id| infrastructure.network(id))
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let reserve_ratio = if network.reserve_capacity > Fx::from_num(0) {
            (network.reserve / network.reserve_capacity)
                .clamp(Fx::from_num(0), Fx::from_num(1))
                .to_num::<f32>()
        } else {
            0.0
        };
        let flow_ratio = if network.demand > Fx::from_num(0) {
            (network.flow / network.demand)
                .clamp(Fx::from_num(0), Fx::from_num(1))
                .to_num::<f32>()
        } else {
            0.0
        };
        let intensity = (reserve_ratio * 0.72 + flow_ratio * 0.28).clamp(0.0, 1.0);
        if intensity <= 0.02 {
            *visibility = Visibility::Hidden;
            continue;
        }
        if let Some(material) = battery_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.primary_color = Vec4::new(0.38, 0.96, 0.72, 1.0);
            material.params.secondary_color = Vec4::new(0.10, 0.42, 0.30, 1.0);
            material.params.intensity = intensity;
            material.params.alpha = 0.18 + intensity * 0.38;
        }
        transform.scale = Vec3::splat(0.86 + intensity * 0.20);
        *visibility = Visibility::Visible;
    }
}

/// Shows fabricator dust and smoke while processors are actively converting routed material.
pub(crate) fn sync_fabricator_dust_visuals(
    time: Res<Time>,
    mut dust_materials: ResMut<Assets<FabricatorDustMaterial>>,
    module_query: Query<(
        &RuntimeShipModule,
        &ProcessorModule,
        Option<&DestroyedModule>,
    )>,
    mut dust_query: Query<(
        &ChildOf,
        &MeshMaterial2d<FabricatorDustMaterial>,
        &mut Visibility,
        &mut Transform,
    )>,
) {
    for (parent, material_handle, mut visibility, mut transform) in &mut dust_query {
        let Ok((runtime_module, processor, destroyed)) = module_query.get(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if destroyed.is_some() || runtime_module.kind != ModuleKind::Processor || !processor.active
        {
            *visibility = Visibility::Hidden;
            continue;
        }
        let progress = (processor.progress / processor.duration.max(Fx::from_num(1)))
            .clamp(Fx::from_num(0), Fx::from_num(1))
            .to_num::<f32>();
        if let Some(material) = dust_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.primary_color = Vec4::new(0.78, 0.72, 0.62, 1.0);
            material.params.secondary_color = Vec4::new(0.34, 0.32, 0.30, 1.0);
            material.params.intensity = 0.45 + progress * 0.55;
            material.params.alpha = 0.28 + progress * 0.24;
        }
        transform.scale = Vec3::splat(0.92 + progress * 0.18);
        *visibility = Visibility::Visible;
    }
}

/// Replaces flat hazard tinting with heat flames and electrical arc overlays on live modules.
/// SAFETY: Arc and flame overlays are separate child entities, but both mutate `Visibility`;
/// `ParamSet` keeps the two overlay passes disjoint at runtime while still allowing modules to have both effects.
pub(crate) fn sync_hazard_effect_visuals(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mut arc_materials: ResMut<Assets<ElectricArcsMaterial>>,
    mut flame_materials: ResMut<Assets<SmallFlamesMaterial>>,
    module_query: Query<(
        &RuntimeShipModule,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    mut overlay_queries: ParamSet<(
        Query<(
            &ChildOf,
            &MeshMaterial2d<ElectricArcsMaterial>,
            &mut Visibility,
        )>,
        Query<(
            &ChildOf,
            &MeshMaterial2d<SmallFlamesMaterial>,
            &mut Visibility,
        )>,
    )>,
) {
    for (parent, material_handle, mut visibility) in &mut overlay_queries.p0() {
        let Ok((_runtime_module, runtime_state, destroyed)) = module_query.get(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if destroyed.is_some() {
            *visibility = Visibility::Hidden;
            continue;
        }
        let intensity = hazard_intensity(
            runtime_state.electrical_instability,
            Fx::from_num(balance.fields.degraded_electrical_threshold),
            Fx::from_num(12),
        );
        if intensity <= 0.01 {
            *visibility = Visibility::Hidden;
            continue;
        }
        if let Some(material) = arc_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.primary_color = Vec4::new(0.54, 0.92, 1.0, 1.0);
            material.params.secondary_color = Vec4::new(0.16, 0.28, 1.0, 1.0);
            material.params.intensity = intensity;
            material.params.alpha = 0.22 + intensity * 0.48;
        }
        *visibility = Visibility::Visible;
    }

    for (parent, material_handle, mut visibility) in &mut overlay_queries.p1() {
        let Ok((_runtime_module, runtime_state, destroyed)) = module_query.get(parent.get()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if destroyed.is_some() {
            *visibility = Visibility::Hidden;
            continue;
        }
        let intensity = hazard_intensity(
            runtime_state.current_heat,
            Fx::from_num(balance.fields.degraded_heat_threshold),
            Fx::from_num(16),
        );
        if intensity <= 0.01 {
            *visibility = Visibility::Hidden;
            continue;
        }
        if let Some(material) = flame_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.primary_color = Vec4::new(1.0, 0.54, 0.16, 1.0);
            material.params.secondary_color = Vec4::new(0.84, 0.12, 0.05, 1.0);
            material.params.intensity = intensity;
            material.params.alpha = 0.20 + intensity * 0.46;
        }
        *visibility = Visibility::Visible;
    }
}

/// Lazily creates and syncs ship-space decompression air lines and movement speed lines.
/// SAFETY: Air-line and speed-line overlays are distinct child entities, but both mutate
/// `Visibility` and `Transform`; `ParamSet` serializes those passes so Bevy never aliases them.
pub(crate) fn sync_ship_environment_effect_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay_mesh: Local<Option<Handle<Mesh>>>,
    mut air_materials: ResMut<Assets<AirLinesMaterial>>,
    mut speed_materials: ResMut<Assets<SpeedLinesMaterial>>,
    ship_query: Query<
        (
            Entity,
            &ShipAtmosphereState,
            &LinearVelocity,
            Option<&PlayerShip>,
        ),
        With<ShipRoot>,
    >,
    mut overlay_queries: ParamSet<(
        Query<(
            &ChildOf,
            &MeshMaterial2d<AirLinesMaterial>,
            &mut Visibility,
            &mut Transform,
        )>,
        Query<(
            &ChildOf,
            &MeshMaterial2d<SpeedLinesMaterial>,
            &mut Visibility,
            &mut Transform,
        )>,
    )>,
) {
    let mesh = overlay_mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(TILE_SIZE * 7.0, TILE_SIZE * 4.0)))
        .clone();
    for (ship_entity, atmosphere, velocity, player_ship) in &ship_query {
        if !overlay_queries
            .p0()
            .iter()
            .any(|(parent, _, _, _)| parent.get() == ship_entity)
        {
            commands.entity(ship_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(air_materials.add(AirLinesMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 1.06),
                    Visibility::Hidden,
                    DecompressionAirLinesOverlay,
                ));
            });
        }
        if player_ship.is_some()
            && !overlay_queries
                .p1()
                .iter()
                .any(|(parent, _, _, _)| parent.get() == ship_entity)
        {
            commands.entity(ship_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(speed_materials.add(SpeedLinesMaterial::default())),
                    Transform::from_xyz(0.0, 0.0, 1.04),
                    Visibility::Hidden,
                    ShipSpeedLinesOverlay,
                ));
            });
        }

        for (parent, material_handle, mut visibility, mut transform) in &mut overlay_queries.p0() {
            if parent.get() != ship_entity {
                continue;
            }
            let intensity = if atmosphere.venting_tiles == 0 {
                0.0
            } else {
                (atmosphere.venting_tiles as f32 / 5.0).clamp(0.18, 1.0)
            };
            if intensity <= 0.01 {
                *visibility = Visibility::Hidden;
                continue;
            }
            let direction = average_decompression_direction(atmosphere);
            if let Some(material) = air_materials.get_mut(&material_handle.0) {
                material.params.time = time.elapsed_secs_wrapped();
                material.params.primary_color = Vec4::new(0.78, 0.94, 1.0, 1.0);
                material.params.secondary_color = Vec4::new(0.32, 0.68, 1.0, 1.0);
                material.params.direction = direction;
                material.params.intensity = intensity;
                material.params.alpha = 0.14 + intensity * 0.24;
            }
            transform.scale = Vec3::splat(0.8 + intensity * 0.3);
            *visibility = Visibility::Visible;
        }

        for (parent, material_handle, mut visibility, mut transform) in &mut overlay_queries.p1() {
            if parent.get() != ship_entity {
                continue;
            }
            let speed = velocity.value.length().to_num::<f32>();
            let intensity = ((speed - 16.0) / 80.0).clamp(0.0, 1.0);
            if intensity <= 0.01 {
                *visibility = Visibility::Hidden;
                continue;
            }
            let direction = if velocity.value.is_near_zero() {
                Vec2::Y
            } else {
                let v = velocity.value.to_vec2().normalize();
                Vec2::new(-v.x, -v.y)
            };
            if let Some(material) = speed_materials.get_mut(&material_handle.0) {
                material.params.time = time.elapsed_secs_wrapped();
                material.params.primary_color = Vec4::new(0.82, 0.92, 1.0, 1.0);
                material.params.secondary_color = Vec4::new(0.48, 0.64, 0.92, 1.0);
                material.params.direction = direction;
                material.params.intensity = intensity;
                material.params.alpha = 0.08 + intensity * 0.16;
            }
            transform.scale = Vec3::splat(1.0 + intensity * 0.24);
            *visibility = Visibility::Visible;
        }
    }
}

fn hazard_intensity(value: Fx, medium: Fx, high: Fx) -> f32 {
    if value < medium || high <= medium {
        return 0.0;
    }
    ((value - medium) / (high - medium))
        .clamp(Fx::from_num(0), Fx::from_num(1))
        .to_num::<f32>()
}

fn average_decompression_direction(atmosphere: &ShipAtmosphereState) -> Vec2 {
    let mut total = Vec2::ZERO;
    for vector in &atmosphere.decompression_vectors {
        total += vector.to_vec2();
    }
    if total.length_squared() <= 0.001 {
        Vec2::Y
    } else {
        total.normalize()
    }
}

fn service_link_should_draw(
    service: &crate::gameplay::components::InfrastructureServiceStatus,
    destroyed: bool,
) -> bool {
    !destroyed
        && service.network_id.is_some()
        && service.service_coord.is_some()
        && service.blocked_reason.is_none()
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
    time: Res<Time>,
    camera_query: Single<&Transform, With<Camera2d>>,
    mut backdrop_materials: ResMut<Assets<SpaceBackdropMaterial>>,
    mut backdrop_queries: ParamSet<(
        Query<
            (
                &ArenaBackdropLayer,
                &MeshMaterial2d<SpaceBackdropMaterial>,
                &mut Transform,
            ),
            (With<SpaceBackdropLayer>, Without<Camera2d>),
        >,
        Query<
            (&ArenaBackdropLayer, &mut Transform),
            (Without<SpaceBackdropLayer>, Without<Camera2d>),
        >,
    )>,
) {
    // SAFETY: Shader and sprite backdrop entities are mutually exclusive by `SpaceBackdropLayer`,
    // so each `ParamSet` branch mutates a disjoint set of transforms.
    let camera = camera_query.into_inner();
    let camera_offset = Vec2::new(camera.translation.x, camera.translation.y);
    for (layer, material_handle, mut transform) in &mut backdrop_queries.p0() {
        transform.translation.x = layer.base_translation.x + camera.translation.x * layer.depth;
        transform.translation.y = layer.base_translation.y + camera.translation.y * layer.depth;
        transform.translation.z = layer.base_translation.z;
        if let Some(material) = backdrop_materials.get_mut(&material_handle.0) {
            material.params.time = time.elapsed_secs_wrapped();
            material.params.camera_offset = camera_offset * layer.depth;
        }
    }
    for (layer, mut transform) in &mut backdrop_queries.p1() {
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

#[cfg(test)]
mod tests {
    use super::service_link_should_draw;
    use crate::gameplay::components::{InfrastructureRouteKind, InfrastructureServiceStatus};

    fn service(
        network_id: Option<u32>,
        service_coord: Option<(i32, i32)>,
        blocked_reason: Option<&str>,
    ) -> InfrastructureServiceStatus {
        InfrastructureServiceStatus {
            route_kind: InfrastructureRouteKind::Power,
            network_id,
            service_coord,
            required: true,
            blocked_reason: blocked_reason.map(str::to_string),
        }
    }

    #[test]
    fn service_links_draw_only_for_connected_unblocked_live_services() {
        assert!(service_link_should_draw(
            &service(Some(1), Some((0, 0)), None),
            false
        ));
        assert!(!service_link_should_draw(
            &service(None, Some((0, 0)), None),
            false
        ));
        assert!(!service_link_should_draw(
            &service(Some(1), None, None),
            false
        ));
        assert!(!service_link_should_draw(
            &service(Some(1), Some((0, 0)), Some("closed valve")),
            false
        ));
        assert!(!service_link_should_draw(
            &service(Some(1), Some((0, 0)), None),
            true
        ));
    }
}
