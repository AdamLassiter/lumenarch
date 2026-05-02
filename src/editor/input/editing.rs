use std::ops::DerefMut;

use bevy::{
    input::mouse::{MouseButton, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use super::super::{
    helpers::{cursor_grid_position, is_cursor_over_editor_ui, is_cursor_over_toolbox, module_kind_cost},
};
use crate::{
    netcode,
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    TOOLBOX_COMPONENTS,
    ship::{enemy::{load_default_enemy_library, save_default_enemy_library}, storage::{load_default_ship, save_default_ship}, ModuleVariant, ShipModule},
    state::{
        DemoProgression, EditorMissionReportButton, EditorMode, EditorPanState,
        EditorSessionState, EditorShip, EditorToolState, EditorUiState, EditorViewState,
        EnemyShipLibraryState, FrontendMode, LeaveEditorButton, MainCamera, ToolboxButton,
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
        let affordable = progression.scrap
            >= module_kind_cost(button.kind, ModuleVariant::default_for_kind(button.kind));
        match *interaction {
            Interaction::Pressed => {
                tool_state.selected_kind = button.kind;
                tool_state.selected_variant = ModuleVariant::default_for_kind(button.kind);
                *background = BackgroundColor(if affordable {
                    PRESSED_BUTTON
                } else {
                    super::super::SELECTED_UNAFFORDABLE_BUTTON
                });
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if affordable {
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
    mut rollback_state: ResMut<netcode::RollbackGameState>,
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
                        rollback_state.phase = netcode::RollbackPhase::Docked;
                        next_mode.set(FrontendMode::Menu);
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
    mut rollback_state: ResMut<netcode::RollbackGameState>,
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
                rollback_state.phase = netcode::RollbackPhase::Docked;
                next_mode.set(FrontendMode::Menu);
            }
        }
    }
}

pub(crate) fn mission_report_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<EditorMissionReportButton>),
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
        let selected_cost = if editor_session.mode == EditorMode::Player {
            module_kind_cost(tool_state.selected_kind, tool_state.selected_variant)
        } else {
            0
        };
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            if existing.kind == tool_state.selected_kind
                && existing.variant == tool_state.selected_variant
            {
                existing.rotation_quadrants = tool_state.selected_rotation;
                return;
            }

            let existing_cost = if editor_session.mode == EditorMode::Player {
                module_kind_cost(existing.kind, existing.variant)
            } else {
                0
            };
            let additional_cost = selected_cost.saturating_sub(existing_cost);
            if progression.scrap < additional_cost {
                return;
            }

            progression.scrap -= additional_cost;
            existing.kind = tool_state.selected_kind;
            existing.variant = tool_state
                .selected_variant
                .normalize_for_kind(tool_state.selected_kind);
            existing.rotation_quadrants = tool_state.selected_rotation;
        } else {
            if progression.scrap < selected_cost {
                return;
            }
            progression.scrap -= selected_cost;
            let next_id = editor_ship.ship.next_module_id();
            editor_ship.ship.replace_module(ShipModule::new(
                next_id,
                tool_state.selected_kind,
                grid_x,
                grid_y,
                tool_state.selected_rotation,
            ));
            if let Some(module) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
                module.variant = tool_state
                    .selected_variant
                    .normalize_for_kind(tool_state.selected_kind);
            }
        }
        rollback_state.editor_ship = editor_ship.ship.clone();
        rollback_state.progression = progression.clone();
    }

    if buttons.just_pressed(MouseButton::Right) {
        editor_ship.ship.remove_module_at(grid_x, grid_y);
        rollback_state.editor_ship = editor_ship.ship.clone();
    }
}

pub(crate) fn save_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => save_default_enemy_library(&enemy_library_state.library),
    };
    if let Err(error) = result {
        eprintln!("editor: failed to save ship data: {error}");
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
        EditorMode::Enemy => match load_default_enemy_library() {
            Ok(Some(library)) => {
                enemy_library_state.library = library;
                enemy_library_state.library.ensure_seeded();
                enemy_library_state.selected_index = enemy_library_state
                    .selected_index
                    .min(enemy_library_state.library.entries.len().saturating_sub(1));
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                    rollback_state.editor_ship = editor_ship.ship.clone();
                }
            }
            Ok(None) => {
                eprintln!("editor: no enemy ship library file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load enemy ship library: {error}");
            }
        },
    }
}

pub(crate) fn persist_editor_ship(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if !editor_ship.is_changed() {
        return;
    }
    rollback_state.editor_ship = editor_ship.ship.clone();

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => {
            enemy_library_state.library.ensure_seeded();
            let selected_index = enemy_library_state.selected_index;
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first_mut(selected_index)
            {
                entry.ship = editor_ship.ship.clone();
                entry.display_name = editor_ship.ship.name.clone();
            }
            save_default_enemy_library(&enemy_library_state.library)
        }
    };

    if let Err(error) = result {
        eprintln!("editor: failed to autosave ship: {error}");
    }
}
