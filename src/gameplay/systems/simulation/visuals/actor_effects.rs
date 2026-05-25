use super::*;

pub(super) fn hazard_intensity(value: Fx, medium: Fx, high: Fx) -> f32 {
    if value < medium || high <= medium {
        return 0.0;
    }
    ((value - medium) / (high - medium))
        .clamp(Fx::from_num(0), Fx::from_num(1))
        .to_num::<f32>()
}

pub(super) fn average_decompression_direction(atmosphere: &ShipAtmosphereState) -> Vec2 {
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

pub(super) fn service_link_should_draw(
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

pub(super) fn collect_active_work(
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

pub(super) fn update_turret_top_visuals(
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
