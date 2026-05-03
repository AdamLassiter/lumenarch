use bevy::prelude::*;

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        ARENA_HEIGHT_TILES,
        ARENA_WIDTH_TILES,
        components::{
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState,
            Integrity,
            SimPosition,
        },
        helpers::{FixedVec2, Fx, angle_from_vector, render_translation},
    },
    state::PlayingCleanup,
};

pub(super) fn spawn_test_arena(
    commands: &mut Commands,
    balance: &BalanceConfig,
    arena_variant: &str,
    platform_hostile_count: u32,
    ambient_heat_pressure: i32,
    ambient_electrical_pressure: i32,
) {
    let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
    let backdrop = match arena_variant {
        "salvage" | "cache" => Color::srgb(0.08, 0.11, 0.10),
        "hostile" => Color::srgb(0.11, 0.08, 0.09),
        "unstable" | "storm" => Color::srgb(0.07, 0.08, 0.13),
        _ => Color::srgb(0.07, 0.09, 0.13),
    };

    commands.spawn((
        Sprite::from_color(backdrop, Vec2::new(arena_width, arena_height)),
        Transform::from_xyz(0.0, 0.0, -20.0),
        PlayingCleanup,
    ));

    spawn_arena_walls(commands, arena_width, arena_height);
    spawn_hostile_platforms(
        commands,
        balance,
        platform_hostile_count,
        ambient_heat_pressure,
        ambient_electrical_pressure,
    );
}

fn spawn_arena_walls(commands: &mut Commands, arena_width: f32, arena_height: f32) {
    let wall_thickness = 8.0;
    let half_w = arena_width * 0.5;
    let half_h = arena_height * 0.5;
    let wall_color = Color::srgb(0.26, 0.30, 0.38);

    for (translation, size) in [
        (
            Vec3::new(0.0, half_h + wall_thickness * 0.5, -19.0),
            Vec2::new(arena_width + wall_thickness * 2.0, wall_thickness),
        ),
        (
            Vec3::new(0.0, -(half_h + wall_thickness * 0.5), -19.0),
            Vec2::new(arena_width + wall_thickness * 2.0, wall_thickness),
        ),
        (
            Vec3::new(-(half_w + wall_thickness * 0.5), 0.0, -19.0),
            Vec2::new(wall_thickness, arena_height),
        ),
        (
            Vec3::new(half_w + wall_thickness * 0.5, 0.0, -19.0),
            Vec2::new(wall_thickness, arena_height),
        ),
    ] {
        commands.spawn((
            Sprite::from_color(wall_color, size),
            Transform::from_translation(translation),
            PlayingCleanup,
        ));
    }
}

fn spawn_hostile_platforms(
    commands: &mut Commands,
    balance: &BalanceConfig,
    hostile_count: u32,
    ambient_heat_pressure: i32,
    ambient_electrical_pressure: i32,
) {
    let platforms = [
        (
            FixedVec2::from_num(-220.0, 120.0),
            Color::srgb(0.82, 0.28, 0.22),
            Fx::from_num(3.0),
            Fx::from_num(1.0),
        ),
        (
            FixedVec2::from_num(210.0, 40.0),
            Color::srgb(0.24, 0.72, 0.96),
            Fx::from_num(0.8),
            Fx::from_num(3.2),
        ),
        (
            FixedVec2::from_num(160.0, -150.0),
            Color::srgb(0.92, 0.58, 0.26),
            Fx::from_num(2.2),
            Fx::from_num(2.2),
        ),
        (
            FixedVec2::from_num(-80.0, -160.0),
            Color::srgb(0.82, 0.48, 0.20),
            Fx::from_num(2.4),
            Fx::from_num(1.8),
        ),
    ];

    for (index, (position, color, heat_damage, electrical_damage)) in
        platforms.into_iter().enumerate()
    {
        if index >= hostile_count as usize {
            break;
        }
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(30.0)),
            Transform {
                translation: render_translation(position, 4.0),
                rotation: Quat::from_rotation_z(
                    (angle_from_vector(FixedVec2::from_num(0.0, 1.0)) - Fx::FRAC_PI_2)
                        .to_num::<f32>(),
                ),
                ..default()
            },
            SimPosition { value: position },
            Integrity { current: 8, max: 8 },
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState {
                cooldown_remaining: Fx::from_num(0.4),
                cooldown_duration: Fx::from_num(balance.combat.hostile_fire_cooldown),
                heat_damage: heat_damage + Fx::from_num(ambient_heat_pressure) * Fx::from_num(0.2),
                electrical_damage: electrical_damage
                    + Fx::from_num(ambient_electrical_pressure) * Fx::from_num(0.2),
            },
            PlayingCleanup,
        ));
    }
}
