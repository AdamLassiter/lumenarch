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
            EditingCleanup,
            EditorLayerButton,
            EditorMode,
            EditorPlacementBlocker,
            EditorSelectionState,
            EditorSessionState,
            EditorShip,
            EditorToolModeButton,
            EditorToolState,
            EditorToolboxScrollContent,
            EditorUiState,
            MainCamera,
            PreviewTile,
            Progression,
            ShipFoundationSprite,
            ShipTileSprite,
            ToolboxFoundationButton,
            ToolboxFoundationButtonText,
            ToolboxVariantButton,
            ToolboxVariantButtonText,
        },
    },
    helpers::{
        cursor_grid_position,
        grid_to_world,
        is_cursor_over_editor_ui,
        is_hull_module_kind,
        module_belongs_to_hull_layer,
        sprite_path_for_foundation,
        sprite_path_for_foundation_connections,
        sprite_path_for_kind,
        variant_inventory_label,
    },
};
use crate::state::{EditorLayer, EditorToolMode};

/// Spawns the translucent build preview sprite so placement intent is visible under the cursor.
pub(crate) fn spawn_preview_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tool_state: Res<EditorToolState>,
) {
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.72),
            ..Sprite::from_image(asset_server.load(
                if tool_state.active_layer != EditorLayer::Components
                    && !module_belongs_to_hull_layer(tool_state.selected_kind)
                {
                    sprite_path_for_foundation(tool_state.selected_foundation_kind)
                } else {
                    sprite_path_for_kind(&tool_state.selected_kind, tool_state.selected_variant)
                },
            ))
        },
        Transform::from_xyz(0.0, 0.0, 5.0),
        Visibility::Hidden,
        PreviewTile,
        EditingCleanup,
    ));
}

/// Moves and retints the preview sprite so build feedback matches the active tool and hovered cell.
pub(crate) fn sync_preview_tile(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    tool_state: Res<EditorToolState>,
    asset_server: Res<AssetServer>,
    ui_blocker_query: Query<
        (
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorPlacementBlocker>,
    >,
    preview_query: Single<(&mut Sprite, &mut Transform, &mut Visibility), With<PreviewTile>>,
) {
    let (mut sprite, mut transform, mut visibility) = preview_query.into_inner();
    let window = window.into_inner();

    if tool_state.tool_mode != EditorToolMode::Build
        || is_cursor_over_editor_ui(window, &ui_blocker_query)
    {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Visible;
    sprite.image = asset_server.load(
        if tool_state.active_layer != EditorLayer::Components
            && !module_belongs_to_hull_layer(tool_state.selected_kind)
        {
            sprite_path_for_foundation(tool_state.selected_foundation_kind)
        } else {
            sprite_path_for_kind(&tool_state.selected_kind, tool_state.selected_variant)
        },
    );
    transform.translation = grid_to_world(
        grid_x,
        grid_y,
        match tool_state.active_layer {
            EditorLayer::Logistics => 0.2,
            EditorLayer::Hull => 0.6,
            EditorLayer::Components => 5.0,
        },
    );
    transform.rotation = Quat::from_rotation_z(-(tool_state.selected_rotation as f32) * FRAC_PI_2);
}

/// Rebuilds editor ship sprites whenever the authored layout changes so all three layers stay readable.
pub(crate) fn sync_ship_tile_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    tool_state: Res<EditorToolState>,
    existing_tiles: Query<Entity, With<ShipTileSprite>>,
    existing_foundations: Query<Entity, With<ShipFoundationSprite>>,
) {
    if !editor_ship.is_changed() && !tool_state.is_changed() {
        return;
    }

    for entity in &existing_tiles {
        commands.entity(entity).despawn();
    }
    for entity in &existing_foundations {
        commands.entity(entity).despawn();
    }

    for tile in &editor_ship.ship.foundation_tiles {
        let same_kind = |x, y| {
            editor_ship
                .ship
                .logistics_at(x, y)
                .is_some_and(|other| other.kind == tile.kind)
        };
        let (path, connection_rotation) = sprite_path_for_foundation_connections(
            tile.kind,
            same_kind(tile.grid_x, tile.grid_y - 1),
            same_kind(tile.grid_x + 1, tile.grid_y),
            same_kind(tile.grid_x, tile.grid_y + 1),
            same_kind(tile.grid_x - 1, tile.grid_y),
        );
        let mut sprite = Sprite::from_image(asset_server.load(path));
        sprite.color = Color::srgba(
            1.0,
            1.0,
            1.0,
            if tool_state.active_layer == EditorLayer::Logistics {
                1.0
            } else {
                0.32
            },
        );
        commands.spawn((
            sprite,
            Transform {
                translation: grid_to_world(tile.grid_x, tile.grid_y, 0.25),
                rotation: Quat::from_rotation_z(
                    -((tile.rotation_quadrants + connection_rotation) as f32) * FRAC_PI_2,
                ),
                ..default()
            },
            ShipFoundationSprite,
            EditingCleanup,
        ));
    }

    for tile in &editor_ship.ship.hull_tiles {
        let mut sprite =
            Sprite::from_image(asset_server.load(sprite_path_for_foundation(tile.kind)));
        sprite.color = Color::srgba(
            1.0,
            1.0,
            1.0,
            if tool_state.active_layer == EditorLayer::Hull {
                1.0
            } else {
                0.36
            },
        );
        commands.spawn((
            sprite,
            Transform {
                translation: grid_to_world(tile.grid_x, tile.grid_y, 0.5),
                rotation: Quat::from_rotation_z(-(tile.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            ShipFoundationSprite,
            EditingCleanup,
        ));
    }

    for module in editor_ship
        .ship
        .modules
        .iter()
        .filter(|module| !is_hull_module_kind(module.kind))
    {
        let mut sprite = Sprite::from_image(
            asset_server.load(sprite_path_for_kind(&module.kind, module.variant)),
        );
        let module_layer = if module_belongs_to_hull_layer(module.kind) {
            EditorLayer::Hull
        } else {
            EditorLayer::Components
        };
        sprite.color = Color::srgba(
            1.0,
            1.0,
            1.0,
            if module_layer == tool_state.active_layer {
                1.0
            } else {
                0.34
            },
        );
        commands.spawn((
            sprite,
            Transform {
                translation: grid_to_world(
                    module.grid_x,
                    module.grid_y,
                    if module_belongs_to_hull_layer(module.kind) {
                        0.75
                    } else {
                        1.0
                    },
                ),
                rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            ShipTileSprite,
            EditingCleanup,
        ));
    }
}

/// Refreshes toolbox button selection and inventory visuals so the editor reflects current tool state.
pub(crate) fn sync_toolbox_visuals(
    tool_state: Res<EditorToolState>,
    progression: Res<Progression>,
    editor_session: Res<EditorSessionState>,
    mut visuals: ParamSet<(
        Query<'_, '_, (&'static ToolboxVariantButton, &'static mut BackgroundColor)>,
        Query<
            '_,
            '_,
            (
                &'static ToolboxFoundationButton,
                &'static mut BackgroundColor,
            ),
        >,
        Query<'_, '_, (&'static ToolboxVariantButtonText, &'static mut Text)>,
        Query<'_, '_, (&'static EditorToolModeButton, &'static mut BackgroundColor)>,
        Query<'_, '_, (&'static EditorLayerButton, &'static mut BackgroundColor)>,
        Query<'_, '_, (&'static ToolboxFoundationButtonText, &'static mut Text)>,
    )>,
) {
    // SAFETY: Variant, foundation, mode, layer, and label widgets are spawned with distinct marker components;
    // each `ParamSet` branch is accessed in sequence, so no entity is mutably borrowed through two queries at once.
    if !tool_state.is_changed() && !progression.is_changed() && !editor_session.is_changed() {
        return;
    }

    for (button, mut background) in &mut visuals.p0() {
        let affordable = tool_state.ignore_component_limits
            || editor_session.mode == EditorMode::Enemy
            || progression.ready_count(button.kind, button.variant) > 0
            || progression.damaged_count(button.kind, button.variant) > 0;
        if button.kind == tool_state.selected_kind && button.variant == tool_state.selected_variant
        {
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

    for (button, mut background) in &mut visuals.p1() {
        *background = BackgroundColor(if button.kind == tool_state.selected_foundation_kind {
            SELECTED_BUTTON
        } else {
            NORMAL_BUTTON
        });
    }

    for (button, mut text) in &mut visuals.p2() {
        **text = format!(
            "{}\n{}",
            button.variant.display_name(),
            if tool_state.ignore_component_limits {
                "limits ignored".to_string()
            } else {
                variant_inventory_label(
                    editor_session.mode,
                    &progression,
                    button.kind,
                    button.variant,
                )
            },
        );
    }

    for (button, mut background) in &mut visuals.p3() {
        *background = BackgroundColor(if button.mode == tool_state.tool_mode {
            SELECTED_BUTTON
        } else {
            NORMAL_BUTTON
        });
    }

    for (button, mut background) in &mut visuals.p4() {
        *background = BackgroundColor(if button.layer == tool_state.active_layer {
            SELECTED_BUTTON
        } else {
            NORMAL_BUTTON
        });
    }

    for (button, mut text) in &mut visuals.p5() {
        **text = button.kind.display_name().to_string();
    }
}

/// Scrolls the toolbox content node so large part lists remain usable in the editor.
pub(crate) fn sync_toolbox_scroll(
    editor_ui_state: Res<EditorUiState>,
    mut query: Query<&mut Node, With<EditorToolboxScrollContent>>,
) {
    if !editor_ui_state.is_changed() {
        return;
    }

    for mut node in &mut query {
        node.top = Val::Px(-editor_ui_state.toolbox_scroll);
    }
}

/// Draws the editor grid so ship layout remains legible while panning and zooming.
pub(crate) fn draw_grid_overlay(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Transform, &Projection), (With<Camera2d>, With<MainCamera>)>,
    mut gizmos: Gizmos,
) {
    let window = window.into_inner();
    let (camera_transform, projection) = camera_query.into_inner();
    let Projection::Orthographic(projection) = projection else {
        return;
    };
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

/// Highlights selection rectangles and selected items so group edits are spatially obvious.
pub(crate) fn draw_editor_selection_overlay(
    editor_ship: Res<EditorShip>,
    selection_state: Res<EditorSelectionState>,
    mut gizmos: Gizmos,
) {
    for tile in &editor_ship.ship.foundation_tiles {
        if !selection_state.selected_foundation_ids.contains(&tile.id) {
            continue;
        }
        let center = Vec2::new(
            tile.grid_x as f32 * TILE_SIZE,
            -(tile.grid_y as f32) * TILE_SIZE,
        );
        gizmos.rect_2d(
            center,
            Vec2::splat(TILE_SIZE + 4.0),
            Color::srgb(0.25, 0.78, 0.95),
        );
    }
    for tile in &editor_ship.ship.hull_tiles {
        if !selection_state.selected_foundation_ids.contains(&tile.id) {
            continue;
        }
        let center = Vec2::new(
            tile.grid_x as f32 * TILE_SIZE,
            -(tile.grid_y as f32) * TILE_SIZE,
        );
        gizmos.rect_2d(
            center,
            Vec2::splat(TILE_SIZE + 4.0),
            Color::srgb(0.58, 0.86, 0.66),
        );
    }

    for module in &editor_ship.ship.modules {
        if !selection_state.selected_module_ids.contains(&module.id) {
            continue;
        }
        let center = Vec2::new(
            module.grid_x as f32 * TILE_SIZE,
            -(module.grid_y as f32) * TILE_SIZE,
        );
        gizmos.rect_2d(
            center,
            Vec2::splat(TILE_SIZE + 6.0),
            Color::srgb(0.95, 0.74, 0.28),
        );
    }

    if let (Some(origin), Some(current)) = (
        selection_state.marquee_origin,
        selection_state.marquee_current,
    ) {
        let min_x = origin.0.min(current.0) as f32 * TILE_SIZE - HALF_TILE_SIZE;
        let max_x = origin.0.max(current.0) as f32 * TILE_SIZE + HALF_TILE_SIZE;
        let min_y = -(origin.1.max(current.1) as f32 * TILE_SIZE) - HALF_TILE_SIZE;
        let max_y = -(origin.1.min(current.1) as f32 * TILE_SIZE) + HALF_TILE_SIZE;
        let center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
        let size = Vec2::new(max_x - min_x, max_y - min_y);
        gizmos.rect_2d(center, size, Color::srgba(0.62, 0.82, 1.0, 0.9));
    }
}
