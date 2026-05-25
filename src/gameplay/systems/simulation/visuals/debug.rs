use super::*;

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
    focused_tile_query: Single<&PlayerFocusedTile, With<ObservedLocalPlayerMarker>>,
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
    let focused_tile = focused_tile_query.into_inner();
    update_turret_top_visuals(ship_rotation.radians, &module_query, &mut turret_top_query);
    draw_observed_focused_tile(focused_tile, &ship_query, &module_query, &mut gizmos);

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

pub(super) fn draw_observed_focused_tile(
    focused_tile: &PlayerFocusedTile,
    ship_query: &Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            Option<&ShipInfrastructureState>,
        ),
        With<ShipRoot>,
    >,
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
    let Some(ship_entity) = focused_tile.ship else {
        return;
    };
    let Ok((_, ship_position, ship_rotation, _)) = ship_query.get(ship_entity) else {
        return;
    };
    let grid_origin = ship_grid_origin(ship_entity, module_query);
    let center = ship_grid_to_world(
        (focused_tile.grid_x, focused_tile.grid_y),
        grid_origin,
        ship_position,
        ship_rotation,
    )
    .to_vec2();
    gizmos.rect_2d(
        center,
        Vec2::splat(TILE_SIZE * 0.96),
        Color::srgba(0.86, 0.96, 1.0, 0.36),
    );
}

pub(super) fn draw_tubes_overlay(
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

pub(super) fn ship_grid_origin(
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

pub(super) fn ship_grid_to_world(
    (grid_x, grid_y): (i32, i32),
    grid_origin: crate::helpers::FixedVec2,
    ship_position: &SimPosition,
    ship_rotation: &SimRotation,
) -> crate::helpers::FixedVec2 {
    let local =
        crate::helpers::FixedVec2::from_num(grid_x * TILE_SIZE as i32, -grid_y * TILE_SIZE as i32);
    ship_position.value + (grid_origin + local).rotate(ship_rotation.radians)
}

pub(super) fn infrastructure_route_kind(
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

pub(super) fn tube_color(kind: InfrastructureRouteKind) -> Color {
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

pub(super) fn module_is_producer(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Reactor | ModuleKind::Battery | ModuleKind::Cargo | ModuleKind::O2Generator
    )
}
