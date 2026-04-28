use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use cordic::{atan2, sin};

use super::{FixedVec2, Fx, WideFx, fx_ratio, widen, wrap_radians};
use crate::client::{
    TILE_SIZE,
    gameplay::{
        ARENA_HEIGHT_TILES,
        ARENA_WIDTH_TILES,
        PROJECTILE_LIFETIME,
        components::{Projectile, ProjectileFaction, SimPosition},
    },
    state::PlayingCleanup,
};

pub(crate) fn spawn_player_projectile(
    commands: &mut Commands,
    origin: FixedVec2,
    velocity: FixedVec2,
) {
    spawn_projectile_entity(
        commands,
        origin,
        velocity,
        ProjectileFaction::Player,
        2,
        Fx::from_num(0),
        Fx::from_num(0),
        Color::srgb(0.98, 0.84, 0.30),
    );
}

pub(crate) fn spawn_projectile_entity(
    commands: &mut Commands,
    origin: FixedVec2,
    velocity: FixedVec2,
    faction: ProjectileFaction,
    damage: i32,
    heat_damage: Fx,
    electrical_damage: Fx,
    color: Color,
) {
    let velocity_angle = angle_from_vector(velocity);

    commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 6.0)),
        Transform {
            translation: render_translation(origin, 2.0),
            rotation: Quat::from_rotation_z(-velocity_angle.to_num::<f32>() + FRAC_PI_2),
            ..default()
        },
        SimPosition { value: origin },
        Projectile {
            velocity,
            remaining_life: Fx::from_num(PROJECTILE_LIFETIME),
            damage,
            faction,
            heat_damage,
            electrical_damage,
        },
        PlayingCleanup,
    ));
}

pub(crate) fn is_inside_arena(position: FixedVec2) -> bool {
    let arena_half_w = Fx::from_num(ARENA_WIDTH_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2);
    let arena_half_h = Fx::from_num(ARENA_HEIGHT_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2);

    position.x >= -arena_half_w
        && position.x <= arena_half_w
        && position.y >= -arena_half_h
        && position.y <= arena_half_h
}

pub(crate) fn damp_scalar(value: Fx, damping: Fx, dt: Fx) -> Fx {
    value * (Fx::from_num(1) / (Fx::from_num(1) + damping * dt))
}

pub(crate) fn damp_vec2(value: FixedVec2, damping: Fx, dt: Fx) -> FixedVec2 {
    value * (Fx::from_num(1) / (Fx::from_num(1) + damping * dt))
}

pub(crate) fn clamp_position_to_arena(position: &mut FixedVec2) {
    let arena_half_w = Fx::from_num(ARENA_WIDTH_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2)
        - Fx::from_num(TILE_SIZE);
    let arena_half_h = Fx::from_num(ARENA_HEIGHT_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2)
        - Fx::from_num(TILE_SIZE);

    position.x = position.x.clamp(-arena_half_w, arena_half_w);
    position.y = position.y.clamp(-arena_half_h, arena_half_h);
}

pub(crate) fn render_translation(position: FixedVec2, z: f32) -> Vec3 {
    Vec3::new(position.x.to_num::<f32>(), position.y.to_num::<f32>(), z)
}

pub(crate) fn facing_vector(radians: Fx) -> FixedVec2 {
    let radians = wrap_radians(radians);
    FixedVec2::new(-sin(radians), cordic::cos(radians))
}

pub(crate) fn angle_from_vector(vector: FixedVec2) -> Fx {
    if vector.is_near_zero() {
        return Fx::from_num(0);
    }

    Fx::from_num(atan2(widen(vector.y), widen(vector.x)))
}

pub(crate) fn fixed_radius_sq(radius: f32) -> WideFx {
    let radius = WideFx::from_num(radius);
    radius * radius
}
