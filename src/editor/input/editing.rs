use std::ops::DerefMut;

use bevy::{
    input::mouse::{MouseButton, MouseWheel},
    log,
    prelude::*,
    window::PrimaryWindow,
};

use super::super::helpers::{
    cursor_grid_position,
    is_cursor_over_editor_ui,
    is_cursor_over_toolbox,
};
use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    TOOLBOX_COMPONENTS,
    netcode,
    ship::{
        ModuleVariant,
        ShipModule,
        enemy::{
            EnemyShipEntryValidationStatus,
            load_validated_default_enemy_library,
            save_default_enemy_library,
            validate_enemy_ship_definition,
        },
        storage::{load_default_ship, save_default_ship},
    },
    state::{
        DemoProgression,
        EditorMissionReportButton,
        EditorMode,
        EditorPanState,
        EditorSessionState,
        EditorShip,
        EditorToolState,
        EditorUiState,
        EditorViewState,
        EnemyEditorState,
        EnemyShipLibraryState,
        FrontendMode,
        LeaveEditorButton,
        MainCamera,
        ToolboxButton,
    },
};

pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (&Interaction, &ToolboxButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    progression: Res<DemoProgression>,
    mut tool_state: ResMut<EditorToolState>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        let available =
            progression.ready_count(button.kind, ModuleVariant::default_for_kind(button.kind)) > 0;
        match *interaction {
            Interaction::Pressed => {
                tool_state.selected_kind = button.kind;
                tool_state.selected_variant = ModuleVariant::default_for_kind(button.kind);
                *background = BackgroundColor(if available {
                    PRESSED_BUTTON
                } else {
                    super::super::SELECTED_UNAFFORDABLE_BUTTON
                });
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if available {
                    HOVERED_BUTTON
                } else {
                    super::super::UNAFFORDABLE_BUTTON
                });
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn leave_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LeaveEditorButton>),
    >,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                match editor_session.mode {
                    EditorMode::Player => {
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LeaveEditor,
                            ..Default::default()
                        });
                    }
                    EditorMode::Enemy => {
                        next_mode.set(FrontendMode::Lobby);
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.56, 0.40, 0.26));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.46, 0.34, 0.22));
            }
        }
    }
}

pub(crate) fn leave_editor_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        match editor_session.mode {
            EditorMode::Player => {
                pending_meta.0 = Some(netcode::PendingMetaCommand {
                    op: netcode::RollbackMetaOp::LeaveEditor,
                    ..Default::default()
                });
            }
            EditorMode::Enemy => {
                next_mode.set(FrontendMode::Lobby);
            }
        }
    }
}

pub(crate) fn mission_report_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<EditorMissionReportButton>,
        ),
    >,
    mut editor_ui_state: ResMut<EditorUiState>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                editor_ui_state.mission_report_expanded = !editor_ui_state.mission_report_expanded;
                *background = BackgroundColor(PRESSED_BUTTON);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(crate::NORMAL_BUTTON);
            }
        }
    }
}

pub(crate) fn rotate_selected_tool(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool_state: ResMut<EditorToolState>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 3) % 4;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 1) % 4;
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, -1);
    }

    if keys.just_pressed(KeyCode::KeyX) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, 1);
    }
}

pub(crate) fn place_or_remove_tile(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<DemoProgression>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        let selected_variant = tool_state
            .selected_variant
            .normalize_for_kind(tool_state.selected_kind);
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            if existing.kind == tool_state.selected_kind && existing.variant == selected_variant {
                existing.rotation_quadrants = tool_state.selected_rotation;
                return;
            }

            if editor_session.mode == EditorMode::Player {
                if !progression
                    .try_consume_ready_component(tool_state.selected_kind, selected_variant)
                {
                    return;
                }
                progression.add_ready_component(existing.kind, existing.variant, 1);
            }
            existing.kind = tool_state.selected_kind;
            existing.variant = selected_variant;
            existing.rotation_quadrants = tool_state.selected_rotation;
        } else {
            if editor_session.mode == EditorMode::Player
                && !progression
                    .try_consume_ready_component(tool_state.selected_kind, selected_variant)
            {
                return;
            }
            let next_id = editor_ship.ship.next_module_id();
            editor_ship.ship.replace_module(ShipModule::new(
                next_id,
                tool_state.selected_kind,
                grid_x,
                grid_y,
                tool_state.selected_rotation,
            ));
            if let Some(module) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
                module.variant = selected_variant;
            }
        }
        if editor_session.mode == EditorMode::Player {
            rollback_state.editor_ship = editor_ship.ship.clone();
            rollback_state.progression = progression.clone();
        } else {
            enemy_editor_state.dirty = true;
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(existing) = editor_ship.ship.module_at(grid_x, grid_y).cloned()
            && editor_session.mode == EditorMode::Player
        {
            progression.add_ready_component(existing.kind, existing.variant, 1);
        }
        editor_ship.ship.remove_module_at(grid_x, grid_y);
        if editor_session.mode == EditorMode::Player {
            rollback_state.progression = progression.clone();
            rollback_state.editor_ship = editor_ship.ship.clone();
        } else {
            enemy_editor_state.dirty = true;
        }
    }
}

pub(crate) fn repair_selected_component_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut progression: ResMut<DemoProgression>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if editor_session.mode != EditorMode::Player || !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    let variant = tool_state
        .selected_variant
        .normalize_for_kind(tool_state.selected_kind);
    if progression.try_repair_component(tool_state.selected_kind, variant) {
        rollback_state.progression = progression.clone();
    }
}

pub(crate) fn save_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
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
    };
    if let Err(error) = result {
        eprintln!("editor: failed to save ship data: {error}");
    } else if editor_session.mode == EditorMode::Enemy {
        enemy_editor_state.dirty = false;
    }
}

pub(crate) fn pan_and_zoom_editor_view(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
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
        if is_cursor_over_toolbox(window) {
            let estimated_content_height = TOOLBOX_COMPONENTS.len() as f32 * 48.0 + 180.0;
            let viewport_height = 430.0;
            let max_scroll = (estimated_content_height - viewport_height).max(0.0);
            editor_ui_state.toolbox_scroll =
                (editor_ui_state.toolbox_scroll - event.y * 28.0).clamp(0.0, max_scroll);
        } else {
            let zoom_step = (1.0 - event.y * 0.08).clamp(0.75, 1.25);
            view_state.zoom = (view_state.zoom * zoom_step).clamp(0.35, 2.75);
        }
    }

    let cursor_position = window.cursor_position();
    if mouse_buttons.pressed(MouseButton::Middle) && !is_cursor_over_editor_ui(window) {
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

pub(crate) fn load_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    status: Res<netcode::SessionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
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
                enemy_library_state.library.ensure_seeded();
                enemy_library_state.selected_index = enemy_library_state
                    .selected_index
                    .min(enemy_library_state.library.entries.len().saturating_sub(1));
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
    }
}

pub(crate) fn persist_editor_ship(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
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
            enemy_editor_state.dirty = true;
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("editor: failed to autosave ship: {error}");
    }
}
