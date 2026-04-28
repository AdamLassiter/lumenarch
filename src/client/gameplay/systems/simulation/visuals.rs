use bevy::prelude::*;

use crate::{
    client::{
        TILE_SIZE,
        gameplay::{
            components::{
                DestroyedModule,
                Integrity,
                ManipulatorModule,
                ModuleCondition,
                ModuleFieldEmitter,
                ModuleRuntimeState,
                PlayerShip,
                RuntimeShipModule,
                ShipRoot,
                SimPosition,
                SimRotation,
                TurretCommandState,
                TurretTopSprite,
            },
            helpers::{Fx, module_condition},
        },
        state::DebugOverlayState,
    },
    ship::ModuleKind,
};

pub(crate) fn toggle_debug_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_overlay: ResMut<DebugOverlayState>,
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_overlay.enabled = !debug_overlay.enabled;
    }
}

pub(crate) fn draw_debug_overlay(
    debug_overlay: Res<DebugOverlayState>,
    player_ship_query: Single<(&SimPosition, &SimRotation), (With<PlayerShip>, With<ShipRoot>)>,
    module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut turret_top_query: Query<(&Parent, &mut Transform), With<TurretTopSprite>>,
    mut gizmos: Gizmos,
) {
    let (ship_position, ship_rotation) = player_ship_query.into_inner();
    update_turret_top_visuals(ship_rotation.radians, &module_query, &mut turret_top_query);

    if !debug_overlay.enabled {
        return;
    }

    let field_radius = TILE_SIZE * 3.5;
    let manipulator_radius = TILE_SIZE * 2.5;

    for (_, runtime_module, emitter, manipulator, _, destroyed) in &module_query {
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
        .map(|(entity, runtime_module, _, _, _, destroyed)| {
            (
                entity,
                runtime_module.module_id,
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians),
                destroyed.is_some(),
            )
        })
        .collect();

    for (_, _, _, manipulator, _, destroyed) in &module_query {
        let Some(manipulator) = manipulator else {
            continue;
        };
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
            .find(|(_, module_id, _, is_destroyed)| *module_id == source_id && !*is_destroyed)
            .map(|(_, _, pos, _)| *pos);
        let target = module_positions
            .iter()
            .find(|(_, module_id, _, is_destroyed)| *module_id == target_id && !*is_destroyed)
            .map(|(_, _, pos, _)| *pos);
        if let (Some(source), Some(target)) = (source, target) {
            gizmos.line_2d(
                source.to_vec2(),
                target.to_vec2(),
                Color::srgb(0.72, 1.0, 0.58),
            );
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
        let hot = runtime_state.current_heat >= Fx::from_num(9);
        let electrical = runtime_state.electrical_instability >= Fx::from_num(8);
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
            ModuleKind::Hull | ModuleKind::HullCorner
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
}

fn update_turret_top_visuals(
    _ship_rotation: Fx,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        Option<&ManipulatorModule>,
        Option<&TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    turret_top_query: &mut Query<(&Parent, &mut Transform), With<TurretTopSprite>>,
) {
    for (parent, mut transform) in turret_top_query.iter_mut() {
        let parent_entity = parent.get();
        let Ok((_, runtime_module, _, _, turret_state, destroyed)) =
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
