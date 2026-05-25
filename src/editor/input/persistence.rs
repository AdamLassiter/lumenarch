/// Saves the current editor ship on demand so manual refit changes can be persisted immediately.
pub(crate) fn save_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut stations: ResMut<StationCatalogResource>,
    station_editor_state: Res<station_editor::StationEditorState>,
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => {
            enemy_library_state.library.ensure_seeded();
            let selected_index = enemy_library_state.selected_index;
            let mut selected_entry_id = None;
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first_mut(selected_index)
            {
                entry.ship = editor_ship.ship.clone();
                entry.display_name = editor_ship.ship.name.clone();
                selected_entry_id = Some(entry.id.clone());
            }
            if let Some(entry_id) = selected_entry_id {
                let status = match validate_enemy_ship_definition(&editor_ship.ship) {
                    Ok(()) => EnemyShipEntryValidationStatus::Valid,
                    Err(error) => {
                        log::warn!(
                            "Blocked enemy library save because the selected enemy ship is invalid: {}",
                            error
                        );
                        enemy_library_state
                            .entry_statuses
                            .insert(entry_id, EnemyShipEntryValidationStatus::Invalid);
                        return;
                    }
                };
                enemy_library_state.entry_statuses.insert(entry_id, status);
            }

            if let Some((entry_id, error)) =
                enemy_library_state
                    .library
                    .entries
                    .iter()
                    .find_map(|entry| {
                        validate_enemy_ship_definition(&entry.ship)
                            .err()
                            .map(|error| (entry.id.clone(), error))
                    })
            {
                log::warn!(
                    "Blocked enemy library save because entry '{}' is invalid: {}",
                    entry_id,
                    error
                );
                return;
            }

            save_default_enemy_library(&enemy_library_state.library)
        }
        EditorMode::Station => station_editor::save_station_editor_catalog(
            &mut stations,
            &station_editor_state,
            &editor_ship,
        ),
    };
    if let Err(error) = result {
        eprintln!("editor: failed to save ship data: {error}");
    } else if editor_session.mode == EditorMode::Enemy {
        enemy_editor_state.dirty = false;
    }
}

/// Pans and zooms the editor camera so large ships remain comfortable to inspect and edit.
pub(crate) fn pan_and_zoom_editor_view(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    toolbox_query: Query<
        (
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorToolboxPanel>,
    >,
    ui_blocker_query: Query<
        (
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorPlacementBlocker>,
    >,
    mut editor_ui_state: ResMut<EditorUiState>,
    mut pan_state: ResMut<EditorPanState>,
    mut view_state: ResMut<EditorViewState>,
    camera_query: Single<(&mut Transform, &mut Projection), (With<Camera2d>, With<MainCamera>)>,
) {
    let window = window.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();
    let Projection::Orthographic(projection) = projection.deref_mut() else {
        return;
    };

    for event in mouse_wheel.read() {
        if is_cursor_over_toolbox(window, &toolbox_query) {
            let estimated_content_height: f32 = 980.0;
            let viewport_height: f32 = 470.0;
            let max_scroll = (estimated_content_height - viewport_height).max(0.0_f32);
            editor_ui_state.toolbox_scroll =
                (editor_ui_state.toolbox_scroll - event.y * 28.0).clamp(0.0, max_scroll);
        } else {
            let zoom_step = (1.0 - event.y * 0.08).clamp(0.75, 1.25);
            view_state.zoom = (view_state.zoom * zoom_step).clamp(0.35, 2.75);
        }
    }

    let cursor_position = window.cursor_position();
    if mouse_buttons.pressed(MouseButton::Middle)
        && !is_cursor_over_editor_ui(window, &ui_blocker_query)
    {
        if let Some(cursor) = cursor_position {
            if let Some(previous_cursor) = pan_state.last_cursor {
                let delta = cursor - previous_cursor;
                view_state.center.x -= delta.x * view_state.zoom;
                view_state.center.y += delta.y * view_state.zoom;
            }
            pan_state.last_cursor = Some(cursor);
        } else {
            pan_state.last_cursor = None;
        }
    } else {
        pan_state.last_cursor = None;
    }

    camera_transform.translation.x = view_state.center.x;
    camera_transform.translation.y = view_state.center.y;
    projection.scale = view_state.zoom;
}

/// Reloads the saved editor ship so refit authors can discard or revisit stored layouts quickly.
pub(crate) fn load_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    status: Res<netcode::SessionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut stations: ResMut<StationCatalogResource>,
    mut station_editor_state: ResMut<station_editor::StationEditorState>,
    sector_state: Res<SectorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }

    match editor_session.mode {
        EditorMode::Player => match load_default_ship() {
            Ok(Some(saved_ship)) => {
                editor_ship.ship = saved_ship;
                rollback_state.editor_ship = editor_ship.ship.clone();
            }
            Ok(None) => {
                eprintln!("editor: no saved ship file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load ship: {error}");
            }
        },
        EditorMode::Enemy => match load_validated_default_enemy_library() {
            Ok(Some(validated)) => {
                log::info!(
                    "Reloaded enemy ship library from disk with {} entries",
                    validated.library.entries.len()
                );
                enemy_library_state.library = validated.library;
                enemy_library_state.entry_statuses = validated.statuses;
                crate::editor::ensure_selected_enemy_reference(
                    &sector_state,
                    &mut enemy_library_state,
                );
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                }
                enemy_editor_state.dirty = false;
            }
            Ok(None) => {
                enemy_library_state.entry_statuses.clear();
                eprintln!("editor: no enemy ship library file was found to load");
                enemy_editor_state.dirty = false;
            }
            Err(error) => {
                enemy_library_state.entry_statuses.clear();
                eprintln!("editor: failed to load enemy ship library: {error}");
                enemy_editor_state.dirty = false;
            }
        },
        EditorMode::Station => {
            if let Err(error) = station_editor::reload_station_catalog(
                &mut stations,
                &mut station_editor_state,
                &mut editor_ship,
            ) {
                eprintln!("editor: failed to reload station catalog: {error}");
            }
            station_editor::ensure_selected_station_reference(
                &sector_state,
                &mut stations,
                &mut station_editor_state,
            );
            station_editor::sync_editor_ship_from_station(
                &stations,
                &station_editor_state,
                &mut editor_ship,
            );
        }
    }
}

/// Persists editor ship changes back to the appropriate save target as the authored layout evolves.
pub(crate) fn persist_editor_ship(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut stations: ResMut<StationCatalogResource>,
    station_editor_state: Res<station_editor::StationEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if !editor_ship.is_changed() {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => {
            rollback_state.editor_ship = editor_ship.ship.clone();
            save_default_ship(&editor_ship.ship)
        }
        EditorMode::Enemy => {
            enemy_library_state.library.ensure_seeded();
            let selected_index = enemy_library_state.selected_index;
            let mut selected_entry_id = None;
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first_mut(selected_index)
            {
                entry.ship = editor_ship.ship.clone();
                entry.display_name = editor_ship.ship.name.clone();
                selected_entry_id = Some(entry.id.clone());
            }
            if let Some(entry_id) = selected_entry_id {
                let status = match validate_enemy_ship_definition(&editor_ship.ship) {
                    Ok(()) => EnemyShipEntryValidationStatus::Valid,
                    Err(_) => EnemyShipEntryValidationStatus::Invalid,
                };
                enemy_library_state.entry_statuses.insert(entry_id, status);
            }
            if let Some((entry_id, error)) =
                enemy_library_state
                    .library
                    .entries
                    .iter()
                    .find_map(|entry| {
                        validate_enemy_ship_definition(&entry.ship)
                            .err()
                            .map(|error| (entry.id.clone(), error))
                    })
            {
                log::warn!(
                    "Skipped enemy library autosave because entry '{}' is invalid: {}",
                    entry_id,
                    error
                );
                enemy_editor_state.dirty = true;
                return;
            }
            save_default_enemy_library(&enemy_library_state.library)
        }
        EditorMode::Station => station_editor::save_station_editor_catalog(
            &mut stations,
            &station_editor_state,
            &editor_ship,
        ),
    };

    if let Err(error) = result {
        eprintln!("editor: failed to autosave ship: {error}");
        if editor_session.mode == EditorMode::Enemy {
            enemy_editor_state.dirty = true;
        }
    } else if editor_session.mode == EditorMode::Enemy {
        enemy_editor_state.dirty = false;
    }
}
use std::ops::DerefMut;

use bevy::{
    input::mouse::{MouseButton, MouseWheel},
    log,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    helpers::editor::{is_cursor_over_editor_ui, is_cursor_over_toolbox},
    netcode,
    ship::{
        enemy::{
            EnemyShipEntryValidationStatus,
            load_validated_default_enemy_library,
            save_default_enemy_library,
            validate_enemy_ship_definition,
        },
        storage::{load_default_ship, save_default_ship},
    },
    state::{
        EditorMode,
        EditorPanState,
        EditorPlacementBlocker,
        EditorSessionState,
        EditorShip,
        EditorToolboxPanel,
        EditorUiState,
        EditorViewState,
        EnemyEditorState,
        EnemyShipLibraryState,
        MainCamera,
        SectorState,
    },
    station_editor,
    stations::StationCatalogResource,
};
