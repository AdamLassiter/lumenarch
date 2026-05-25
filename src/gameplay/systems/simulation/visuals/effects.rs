use super::*;

/// Tints or hides module sprites to reflect heat, instability, disablement, and destruction.
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
                            ..Sprite::from_image(
                                asset_server.load("tiles/logistics/service_link.png"),
                            )
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
