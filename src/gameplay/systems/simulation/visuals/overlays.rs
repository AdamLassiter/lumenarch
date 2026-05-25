use super::*;

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
