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
        ArchEditorState,
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
        GameplayStationPanelButton,
        LeaveEditorButton,
        MainCamera,
        ProgrammingLanguageMode,
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
    if keys.just_pressed(KeyCode::KeyR) {
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

    if keys.just_pressed(KeyCode::KeyC) {
        tool_state.selected_channel = tool_state.selected_channel.wrapping_add(9) % 10;
    }

    if keys.just_pressed(KeyCode::KeyV) {
        tool_state.selected_channel = (tool_state.selected_channel + 1) % 10;
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
    mut arch_editor_state: ResMut<ArchEditorState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window) {
        return;
    }

    if arch_editor_state.panel_open {
        let width = window.width();
        let height = window.height();
        let over_module_panel = if let Some(cursor) = window.cursor_position() {
            cursor.x >= width * 0.2
                && cursor.x <= width * 0.8
                && cursor.y >= 160.0
                && cursor.y <= height - 40.0
        } else {
            false
        };
        if over_module_panel {
            return;
        }
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
                existing.channel = tool_state.selected_channel;
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
            existing.channel = tool_state.selected_channel;
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
                module.channel = tool_state.selected_channel;
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

pub(crate) fn toggle_editor_module_overlay_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_ship: Res<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        arch_editor_state.panel_open = false;
        return;
    }

    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let window = window.into_inner();
    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };
    let Some(module) = editor_ship.ship.module_at(grid_x, grid_y) else {
        return;
    };
    arch_editor_state.selected_module_id = Some(module.id);
    arch_editor_state.panel_open = true;
    tool_state.selected_kind = module.kind;
    tool_state.selected_variant = module.variant;
    tool_state.selected_rotation = module.rotation_quadrants;
    tool_state.selected_channel = module.effective_channel();
}

pub(crate) fn editor_station_panel_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &GameplayStationPanelButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_ship: ResMut<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !arch_editor_state.panel_open || !netcode::is_host_authority(&status) {
        return;
    }
    let Some(module_id) = arch_editor_state.selected_module_id else {
        return;
    };
    let mut changed = false;
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let Some(module) = editor_ship
                    .ship
                    .modules
                    .iter_mut()
                    .find(|module| module.id == module_id)
                else {
                    continue;
                };
                match button.action {
                    crate::state::StationPanelButtonAction::HelmThrottle { .. }
                    | crate::state::StationPanelButtonAction::HelmTurn { .. } => {}
                    crate::state::StationPanelButtonAction::TurretAdjustAim { .. } => {}
                    crate::state::StationPanelButtonAction::TurretFireToggle => {
                        module.defaults.turret_fire_intent = !module.defaults.turret_fire_intent;
                    }
                    crate::state::StationPanelButtonAction::ReactorAdjustRate { delta } => {
                        let current = module.defaults.reaction_rate_milli as i32;
                        module.defaults.reaction_rate_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    crate::state::StationPanelButtonAction::ReactorAdjustTurbine { delta } => {
                        let current = module.defaults.turbine_load_milli as i32;
                        module.defaults.turbine_load_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    crate::state::StationPanelButtonAction::LogisticsToggleStorageIntake => {
                        module.defaults.storage_allow_intake =
                            !module.defaults.storage_allow_intake;
                    }
                    crate::state::StationPanelButtonAction::LogisticsToggleAirlock => {
                        module.defaults.airlock_open = !module.defaults.airlock_open;
                    }
                    crate::state::StationPanelButtonAction::LogisticsToggleManipulator => {
                        module.defaults.manipulator_transfer_enabled =
                            !module.defaults.manipulator_transfer_enabled;
                    }
                    crate::state::StationPanelButtonAction::LogisticsCycleManipulatorTarget {
                        ..
                    } => {
                        module.defaults.manipulator_manual_mode =
                            !module.defaults.manipulator_manual_mode;
                    }
                    crate::state::StationPanelButtonAction::LogisticsCycleResource => {
                        module.defaults.manipulator_resource_kind =
                            module.defaults.manipulator_resource_kind.next();
                    }
                    crate::state::StationPanelButtonAction::LogisticsToggleProcessor => {
                        if module.kind == crate::ship::ModuleKind::Processor {
                            module.defaults.processor_enabled = !module.defaults.processor_enabled;
                        } else {
                            module.defaults.processor_recipe =
                                module.defaults.processor_recipe.next();
                        }
                    }
                    crate::state::StationPanelButtonAction::ComputerToggleEnabled => {
                        module.defaults.computer_enabled = !module.defaults.computer_enabled;
                    }
                    crate::state::StationPanelButtonAction::ComputerCycleTemplate => {
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => {
                                let program = module.arch_program.get_or_insert_with(|| {
                                    crate::ship::arch::ArchProgram::from_template(
                                        crate::ship::arch::ArchProgramTemplate::BalancedOps,
                                    )
                                });
                                *program = crate::ship::arch::ArchProgram::from_template(
                                    program.template.next(),
                                );
                            }
                            ProgrammingLanguageMode::Lumen => {
                                let program = module.lumen_program.get_or_insert_with(|| {
                                    crate::ship::lumen::LumenProgram::from_template(
                                    crate::ship::lumen::LumenProgramTemplate::BalancedSupervision,
                                )
                                });
                                *program = crate::ship::lumen::LumenProgram::from_template(
                                    program.template.next(),
                                );
                            }
                        }
                    }
                }
                changed = true;
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }

    if changed {
        if editor_session.mode == EditorMode::Player {
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
    if editor_session.mode != EditorMode::Player || !keys.just_pressed(KeyCode::KeyT) {
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
