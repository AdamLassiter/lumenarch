use bevy::prelude::*;

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        ARENA_HEIGHT_TILES,
        ARENA_WIDTH_TILES,
        components::{
            ArenaBackdropLayer,
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState,
            Integrity,
            SimPosition,
        },
        helpers::{FixedVec2, Fx, angle_from_vector, render_translation},
    },
    state::{EncounterBackdrop, EncounterSpec, PlayingCleanup},
};

/// Assembles the encounter arena backdrop, walls, and static threats so combat starts with a complete playspace.
pub(crate) fn spawn_test_arena(
    commands: &mut Commands,
    balance: &BalanceConfig,
    encounter: &EncounterSpec,
    platform_hostile_count: u32,
) {
    let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
    let backdrop = backdrop_color(&encounter.arena_variant, &encounter.backdrop);

    spawn_backdrop_layers(
        commands,
        arena_width,
        arena_height,
        &encounter.backdrop,
        backdrop,
    );
    spawn_arena_walls(commands, arena_width, arena_height);
    spawn_hostile_platforms(
        commands,
        balance,
        platform_hostile_count,
        encounter.ambient_heat_pressure,
        encounter.ambient_electrical_pressure,
    );
}

fn backdrop_color(arena_variant: &str, backdrop: &EncounterBackdrop) -> Color {
    let [r, g, b] = backdrop.haze_tint;
    match arena_variant {
        "salvage" | "cache" => Color::srgb(r.max(0.08), g.max(0.11), b.max(0.10)),
        "hostile" => Color::srgb(r.max(0.11), g.max(0.08), b.max(0.09)),
        "unstable" | "storm" => Color::srgb(r.max(0.07), g.max(0.08), b.max(0.13)),
        _ => Color::srgb(r.max(0.07), g.max(0.09), b.max(0.13)),
    }
}

fn spawn_backdrop_layers(
    commands: &mut Commands,
    arena_width: f32,
    arena_height: f32,
    backdrop: &EncounterBackdrop,
    base_color: Color,
) {
    let bounds = Vec2::new(arena_width, arena_height);
    spawn_backdrop_sprite(
        commands,
        base_color,
        Vec2::new(arena_width * 1.4, arena_height * 1.4),
        Vec3::new(0.0, 0.0, -25.0),
        0.04,
    );
    spawn_backdrop_sprite(
        commands,
        Color::srgba(
            backdrop.haze_tint[0],
            backdrop.haze_tint[1],
            backdrop.haze_tint[2],
            0.18,
        ),
        Vec2::new(arena_width * 0.95, arena_height * 0.78),
        Vec3::new(-arena_width * 0.08, arena_height * 0.06, -24.7),
        0.08,
    );
    spawn_backdrop_sprite(
        commands,
        Color::srgba(
            backdrop.galaxy_tint[0],
            backdrop.galaxy_tint[1],
            backdrop.galaxy_tint[2],
            0.16 + backdrop.galaxy_arc_strength * 0.10,
        ),
        Vec2::new(arena_width * 0.82, arena_height * 0.16),
        Vec3::new(0.0, arena_height * 0.12, -24.4),
        0.14,
    );

    let star_count = backdrop.star_density.max(24);
    let dust_count = backdrop.dust_density.max(8);
    let mut seed = if backdrop.seed == 0 {
        0xC0FFEE_u64
    } else {
        backdrop.seed
    };

    for index in 0..star_count {
        let x = rand_range(&mut seed, -bounds.x * 0.62, bounds.x * 0.62);
        let y = rand_range(&mut seed, -bounds.y * 0.62, bounds.y * 0.62);
        let depth = 0.10 + (index % 3) as f32 * backdrop.parallax_strength * 0.15;
        let size = 1.5 + rand_range(&mut seed, 0.0, 2.8);
        let alpha = 0.35 + rand_range(&mut seed, 0.0, 0.45);
        spawn_backdrop_sprite(
            commands,
            Color::srgba(0.82, 0.90, 1.0, alpha),
            Vec2::splat(size),
            Vec3::new(x, y, -24.0 + depth),
            depth,
        );
    }

    let cluster_count = ((dust_count as f32) / 3.5).ceil() as u32;
    let mut remaining = dust_count;
    for cluster_index in 0..cluster_count.max(1) {
        if remaining == 0 {
            break;
        }
        let clusters_left = cluster_count.saturating_sub(cluster_index).max(1);
        let min_pieces = 2u32.min(remaining);
        let max_pieces = 4u32.min(remaining - min_pieces + 1) + min_pieces - 1;
        let target_for_cluster = ((remaining as f32) / clusters_left as f32).ceil() as u32;
        let pieces = target_for_cluster.clamp(min_pieces, max_pieces.max(min_pieces));
        remaining = remaining.saturating_sub(pieces);

        let center_x = rand_range(&mut seed, -bounds.x * 0.42, bounds.x * 0.42);
        let center_y = rand_range(&mut seed, -bounds.y * 0.38, bounds.y * 0.38);
        let cluster_rotation = rand_range(&mut seed, -0.65, 0.65);
        let spread_x = rand_range(&mut seed, 10.0, 28.0);
        let spread_y = rand_range(&mut seed, 8.0, 18.0);

        for piece_index in 0..pieces {
            let x = center_x + rand_range(&mut seed, -spread_x, spread_x);
            let y = center_y + rand_range(&mut seed, -spread_y, spread_y);
            let width = if piece_index == 0 && rand_range(&mut seed, 0.0, 1.0) > 0.65 {
                rand_range(&mut seed, 26.0, 38.0)
            } else {
                rand_range(&mut seed, 10.0, 24.0)
            };
            let height = rand_range(&mut seed, 3.0, 7.0);
            let rotation = cluster_rotation + rand_range(&mut seed, -0.25, 0.25);
            let tint_mix = rand_range(&mut seed, 0.0, 1.0);
            let color = Color::srgba(
                backdrop.galaxy_tint[0] * (0.52 + tint_mix * 0.34),
                backdrop.galaxy_tint[1] * (0.52 + tint_mix * 0.34),
                backdrop.galaxy_tint[2] * (0.52 + tint_mix * 0.34),
                0.07 + rand_range(&mut seed, 0.0, 0.08),
            );
            commands.spawn((
                Sprite::from_color(color, Vec2::new(width, height)),
                Transform {
                    translation: Vec3::new(x, y, -23.7),
                    rotation: Quat::from_rotation_z(rotation),
                    ..default()
                },
                ArenaBackdropLayer {
                    depth: 0.22 + backdrop.parallax_strength * 0.18,
                    base_translation: Vec3::new(x, y, -23.7),
                },
                PlayingCleanup,
            ));
        }
    }
}

fn spawn_backdrop_sprite(
    commands: &mut Commands,
    color: Color,
    size: Vec2,
    translation: Vec3,
    depth: f32,
) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_translation(translation),
        ArenaBackdropLayer {
            depth,
            base_translation: translation,
        },
        PlayingCleanup,
    ));
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

fn rand_unit(seed: &mut u64) -> f32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*seed >> 16) as u32) as f32 / u32::MAX as f32
}

fn rand_range(seed: &mut u64, min: f32, max: f32) -> f32 {
    min + (max - min) * rand_unit(seed)
}
