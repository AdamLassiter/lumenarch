use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, window::PrimaryWindow};

use super::{
    super::{
        GRID_COLOR,
        HALF_TILE_SIZE,
        NORMAL_BUTTON,
        SELECTED_BUTTON,
        TILE_SIZE,
        state::{
            DemoProgression,
            EditingCleanup,
            EditorShip,
            EditorToolState,
            MainCamera,
            PreviewTile,
            ShipTileSprite,
            ToolboxButton,
        },
    },
    helpers::{
        cursor_grid_position,
        grid_to_world,
        is_cursor_over_toolbox,
        module_kind_cost,
        sprite_path_for_kind,
    },
};

pub(crate) fn spawn_preview_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tool_state: Res<EditorToolState>,
) {
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.72),
            ..Sprite::from_image(asset_server.load(sprite_path_for_kind(&tool_state.selected_kind)))
        },
        Transform::from_xyz(0.0, 0.0, 5.0),
        Visibility::Hidden,
        PreviewTile,
        EditingCleanup,
    ));
}

pub(crate) fn sync_preview_tile(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    tool_state: Res<EditorToolState>,
    asset_server: Res<AssetServer>,
    preview_query: Single<(&mut Sprite, &mut Transform, &mut Visibility), With<PreviewTile>>,
) {
    let (mut sprite, mut transform, mut visibility) = preview_query.into_inner();
    let window = window.into_inner();

    if is_cursor_over_toolbox(window) {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Visible;
    sprite.image = asset_server.load(sprite_path_for_kind(&tool_state.selected_kind));
    transform.translation = grid_to_world(grid_x, grid_y, 5.0);
    transform.rotation = Quat::from_rotation_z(-(tool_state.selected_rotation as f32) * FRAC_PI_2);
}

pub(crate) fn sync_ship_tile_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    existing_tiles: Query<Entity, With<ShipTileSprite>>,
) {
    if !editor_ship.is_changed() {
        return;
    }

    for entity in &existing_tiles {
        commands.entity(entity).despawn();
    }

    for module in &editor_ship.ship.modules {
        commands.spawn((
            Sprite::from_image(asset_server.load(sprite_path_for_kind(&module.kind))),
            Transform {
                translation: grid_to_world(module.grid_x, module.grid_y, 1.0),
                rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            ShipTileSprite,
            EditingCleanup,
        ));
    }
}

pub(crate) fn sync_toolbox_visuals(
    tool_state: Res<EditorToolState>,
    progression: Res<DemoProgression>,
    mut query: Query<(&ToolboxButton, &mut BackgroundColor)>,
) {
    if !tool_state.is_changed() && !progression.is_changed() {
        return;
    }

    for (button, mut background) in &mut query {
        let affordable = progression.scrap >= module_kind_cost(button.kind);
        if button.kind == tool_state.selected_kind {
            *background = BackgroundColor(if affordable {
                SELECTED_BUTTON
            } else {
                super::SELECTED_UNAFFORDABLE_BUTTON
            });
        } else {
            *background = BackgroundColor(if affordable {
                NORMAL_BUTTON
            } else {
                super::UNAFFORDABLE_BUTTON
            });
        }
    }
}

pub(crate) fn draw_grid_overlay(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Transform, &OrthographicProjection), (With<Camera2d>, With<MainCamera>)>,
    mut gizmos: Gizmos,
) {
    let window = window.into_inner();
    let (camera_transform, projection) = camera_query.into_inner();
    let half_w = (window.width() * projection.scale * 0.5) + TILE_SIZE * 4.0;
    let half_h = (window.height() * projection.scale * 0.5) + TILE_SIZE * 4.0;
    let min_world_x = camera_transform.translation.x - half_w;
    let max_world_x = camera_transform.translation.x + half_w;
    let min_world_y = camera_transform.translation.y - half_h;
    let max_world_y = camera_transform.translation.y + half_h;

    let min_x = ((min_world_x - HALF_TILE_SIZE) / TILE_SIZE).floor() as i32;
    let max_x = ((max_world_x + HALF_TILE_SIZE) / TILE_SIZE).ceil() as i32;
    let min_y = ((min_world_y - HALF_TILE_SIZE) / TILE_SIZE).floor() as i32;
    let max_y = ((max_world_y + HALF_TILE_SIZE) / TILE_SIZE).ceil() as i32;

    for grid_x in min_x..=max_x {
        let x = grid_x as f32 * TILE_SIZE - HALF_TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(x, min_world_y),
            Vec2::new(x, max_world_y),
            GRID_COLOR,
        );
    }

    for grid_y in min_y..=max_y {
        let y = grid_y as f32 * TILE_SIZE - HALF_TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(min_world_x, y),
            Vec2::new(max_world_x, y),
            GRID_COLOR,
        );
    }
}
