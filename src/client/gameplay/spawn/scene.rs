use bevy::prelude::*;

use super::{
    super::{
        super::{
            TILE_SIZE,
            state::{
                EditorShip,
                GameplayAlertsText,
                GameplayInspectionText,
                GameplayStatusText,
                PlayingCleanup,
                ReturnButton,
            },
        },
        ARENA_HEIGHT_TILES,
        ARENA_WIDTH_TILES,
        HOSTILE_FIRE_COOLDOWN,
        components::{
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState,
            Integrity,
            SalvagePickup,
            SalvageWreck,
            SimPosition,
        },
        helpers::{FixedVec2, Fx, gameplay_status_line},
    },
    ship::spawn_runtime_ship,
};

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            PlayingCleanup,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    top: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Runtime Slice"),
                    TextFont {
                        font: title_font,
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                panel.spawn((
                    Text::new(gameplay_status_line(&editor_ship.ship)),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    GameplayStatusText,
                ));

                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.52, 0.27, 0.18)),
                        BorderRadius::all(Val::Px(8.0)),
                        ReturnButton,
                    ))
                    .with_child((
                        Text::new("Return To Editor"),
                        TextFont {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    max_width: Val::Px(340.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Current Station"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                panel.spawn((
                    Text::new("Station data pending"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.86, 0.90, 0.96)),
                    GameplayInspectionText,
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    top: Val::Px(220.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    max_width: Val::Px(340.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Alerts"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                panel.spawn((
                    Text::new("No alerts"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.86, 0.90, 0.96)),
                    GameplayAlertsText,
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    max_width: Val::Px(320.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Controls\nW / A / S / D or Arrows: flight or internal movement\nC: toggle flight/internal control\nE: interact / hold to work\nComputer: cycle ARCH reactor guard\nSpace: fire turrets\nF: collect salvage after encounter\nTab: return to editor",
                    ),
                    TextFont {
                        font: mono_font,
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });

    spawn_test_arena(&mut commands);
    spawn_salvage_wreck(&mut commands);
    spawn_runtime_ship(&mut commands, &asset_server, &editor_ship.ship);
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    query: Query<Entity, With<PlayingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_test_arena(commands: &mut Commands) {
    let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.07, 0.09, 0.13),
            Vec2::new(arena_width, arena_height),
        ),
        Transform::from_xyz(0.0, 0.0, -20.0),
        PlayingCleanup,
    ));

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

    for (translation, color, cooldown, heat_damage, electrical_damage) in [
        (
            Vec3::new(210.0, 130.0, -5.0),
            Color::srgb(0.92, 0.42, 0.24),
            HOSTILE_FIRE_COOLDOWN * 0.8,
            Fx::from_num(4),
            Fx::from_num(1),
        ),
        (
            Vec3::new(-220.0, 20.0, -5.0),
            Color::srgb(0.36, 0.72, 0.96),
            HOSTILE_FIRE_COOLDOWN * 0.7,
            Fx::from_num(1),
            Fx::from_num(4),
        ),
        (
            Vec3::new(160.0, -150.0, -5.0),
            Color::srgb(0.88, 0.34, 0.56),
            HOSTILE_FIRE_COOLDOWN,
            Fx::from_num(3),
            Fx::from_num(3),
        ),
    ] {
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(28.0)),
            Transform::from_translation(translation),
            HostileTarget,
            HostileTurretPlatform,
            SimPosition {
                value: FixedVec2::from_vec2(translation.truncate()),
            },
            HostileWeaponState {
                cooldown_remaining: Fx::from_num(cooldown * 0.5),
                cooldown_duration: Fx::from_num(cooldown),
                heat_damage,
                electrical_damage,
            },
            Integrity { current: 6, max: 6 },
            PlayingCleanup,
        ));
    }
}

fn spawn_salvage_wreck(commands: &mut Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.34, 0.72, 0.62), Vec2::new(26.0, 26.0)),
        Transform::from_xyz(0.0, -220.0, -4.0),
        SimPosition {
            value: FixedVec2::from_num(0, -220),
        },
        SalvageWreck,
        SalvagePickup { scrap_value: 6 },
        PlayingCleanup,
    ));
}
