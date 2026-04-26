use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

use super::{
    components::{
        AngularVelocity, EngineModule, HostileTarget, HostileTurretPlatform, HostileWeaponState,
        Integrity, LinearVelocity, MissionState, PlayerShip, PowerConsumer, PowerProducer,
        RuntimeShipModule, SalvagePickup, SalvageWreck, ShipControlState, ShipPowerState, ShipRoot,
        ShipWeaponState,
        WeaponModule,
    },
    helpers::{
        count_modules, gameplay_status_line, module_integrity, module_local_translation,
        ship_movement_model, ship_power_model, sprite_path_for_kind,
    },
    ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, HOSTILE_FIRE_COOLDOWN, RUNTIME_SHIP_ORIGIN,
};
use super::super::{
    state::{EditorShip, GameplayStatusText, PlayingCleanup, ReturnButton},
    TILE_SIZE,
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
                    bottom: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    max_width: Val::Px(300.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Flight Controls\nW / Up: thrust\nA,D / Left,Right: rotate\nSpace: fire turrets\nF: collect salvage after encounter\nTab: return to editor",
                    ),
                    TextFont {
                        font: mono_font.clone(),
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

    for translation in [
        Vec3::new(220.0, 150.0, -5.0),
        Vec3::new(-260.0, 40.0, -5.0),
        Vec3::new(180.0, -170.0, -5.0),
    ] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.76, 0.24, 0.20), Vec2::splat(28.0)),
            Transform::from_translation(translation),
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState {
                cooldown_remaining: HOSTILE_FIRE_COOLDOWN * 0.5,
                cooldown_duration: HOSTILE_FIRE_COOLDOWN,
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
        SalvageWreck,
        SalvagePickup { scrap_value: 6 },
        PlayingCleanup,
    ));
}

fn spawn_runtime_ship(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
) {
    let engine_count = count_modules(ship, ModuleKind::Engine);
    let reactor_count = count_modules(ship, ModuleKind::Reactor);
    let battery_count = count_modules(ship, ModuleKind::Battery);
    let turret_count = count_modules(ship, ModuleKind::Turret);
    let movement_model = ship_movement_model(ship.modules.len(), engine_count);
    let power_model = ship_power_model(
        ship.modules.len(),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
    );

    let root_entity = commands
        .spawn((
            Transform::from_translation(RUNTIME_SHIP_ORIGIN),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            PlayerShip,
            ShipRoot,
            PlayingCleanup,
        ))
        .insert((
            LinearVelocity { value: Vec2::ZERO },
            AngularVelocity {
                radians_per_second: 0.0,
            },
            movement_model,
            ShipPowerState {
                stored_energy: power_model.battery_capacity,
                generation: power_model.reactor_output,
                draw: power_model.passive_draw,
                surplus: power_model.reactor_output - power_model.passive_draw,
                engine_power_ratio: if engine_count > 0 { 1.0 } else { 0.0 },
                weapons_powered: turret_count == 0,
                engines_powered: engine_count > 0,
            },
            power_model,
            ShipControlState::default(),
            ShipWeaponState {
                turret_count,
                cooldown_remaining: 0.0,
                cooldown_duration: if turret_count > 0 { 0.3 } else { 0.0 },
            },
            MissionState {
                failed: false,
                failure_reason: None,
                completed: false,
                completion_reason: None,
                salvage_collected: false,
                salvage_scrap_awarded: 0,
                return_delay_remaining: None,
            },
        ))
        .id();

    let (min_x, max_x, min_y, max_y) = ship.bounds().unwrap_or((0, 0, 0, 0));
    let center_x = (min_x + max_x) as f32 * 0.5;
    let center_y = (min_y + max_y) as f32 * 0.5;

    let child_entities: Vec<_> = ship
        .modules
        .iter()
        .map(|module| spawn_runtime_module(commands, asset_server, module, center_x, center_y))
        .collect();

    commands.entity(root_entity).add_children(&child_entities);
}

fn spawn_runtime_module(
    commands: &mut Commands,
    asset_server: &AssetServer,
    module: &ShipModule,
    center_x: f32,
    center_y: f32,
) -> Entity {
    let mut entity = commands.spawn((
        Sprite::from_image(asset_server.load(sprite_path_for_kind(module.kind))),
        Transform {
            translation: module_local_translation(module, center_x, center_y),
            rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
            ..default()
        },
        RuntimeShipModule {
            module_id: module.id,
            kind: module.kind,
        },
        Integrity {
            current: module_integrity(module.kind),
            max: module_integrity(module.kind),
        },
        PlayingCleanup,
    ));

    match module.kind {
        ModuleKind::Reactor => {
            entity.insert(PowerProducer { output: 10 });
        }
        ModuleKind::Battery => {
            entity.insert(PowerProducer { output: 4 });
        }
        ModuleKind::Engine => {
            entity.insert((PowerConsumer { draw: 3 }, EngineModule));
        }
        ModuleKind::Turret => {
            entity.insert((PowerConsumer { draw: 2 }, WeaponModule));
        }
        _ => {}
    }

    entity.id()
}
