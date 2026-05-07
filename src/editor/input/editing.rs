use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
};

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
    module_family_label,
    variant_tooltip_text,
};
use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    ship::{
        ModuleKind,
        ModuleVariant,
        ShipDefinition,
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
        EditorAutoHullButton,
        EditorCopySelectionButton,
        EditorDeleteSelectionButton,
        EditorMissionReportButton,
        EditorMode,
        EditorPanState,
        EditorPasteSelectionButton,
        EditorPlacementBlocker,
        EditorPointerState,
        EditorSelectionState,
        EditorSessionState,
        EditorShip,
        EditorToolMode,
        EditorToolModeButton,
        EditorToolState,
        EditorToolboxPanel,
        EditorUiState,
        EditorViewState,
        EnemyEditorState,
        EnemyShipLibraryState,
        FrontendMode,
        GameplayStationPanelButton,
        LeaveEditorButton,
        MainCamera,
        ProgrammingLanguageMode,
        Progression,
        ToolboxVariantButton,
    },
};

pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ToolboxVariantButton>,
            Option<&EditorToolModeButton>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    progression: Res<Progression>,
    mut tool_state: ResMut<EditorToolState>,
    mut editor_ui_state: ResMut<EditorUiState>,
) {
    for (interaction, variant_button, mode_button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(button) = mode_button {
                    tool_state.tool_mode = button.mode;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = variant_button {
                    let available = editor_session.mode == EditorMode::Enemy
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    if !available {
                        *background = BackgroundColor(super::super::SELECTED_UNAFFORDABLE_BUTTON);
                        continue;
                    }
                    tool_state.tool_mode = EditorToolMode::Build;
                    tool_state.selected_kind = button.kind;
                    tool_state.selected_variant = button.variant;
                    *background = BackgroundColor(PRESSED_BUTTON);
                }
            }
            Interaction::Hovered => {
                if let Some(button) = variant_button {
                    editor_ui_state.toolbox_tooltip.title = format!(
                        "{} / {}",
                        module_family_label(button.kind),
                        button.kind.as_str()
                    );
                    editor_ui_state.toolbox_tooltip.detail = variant_tooltip_text(
                        editor_session.mode,
                        &progression,
                        button.kind,
                        button.variant,
                    );
                    let available = editor_session.mode == EditorMode::Enemy
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    *background = BackgroundColor(if available {
                        HOVERED_BUTTON
                    } else {
                        super::super::UNAFFORDABLE_BUTTON
                    });
                } else {
                    *background = BackgroundColor(HOVERED_BUTTON);
                }
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
    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyR) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 1) % 4;
    }

    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyZ) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, -1);
    }

    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyX) {
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
    ui_blocker_query: Query<
        (
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorPlacementBlocker>,
    >,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut pointer_state: ResMut<EditorPointerState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window, &ui_blocker_query) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    match tool_state.tool_mode {
        EditorToolMode::Build => {
            selection_state.marquee_origin = None;
            selection_state.marquee_current = None;

            if buttons.just_released(MouseButton::Left) || buttons.just_released(MouseButton::Right)
            {
                pointer_state.last_build_cell = None;
            }

            for (mouse_button, erase) in [(MouseButton::Left, false), (MouseButton::Right, true)] {
                if (buttons.just_pressed(mouse_button) || buttons.pressed(mouse_button))
                    && pointer_state.last_build_cell != Some((grid_x, grid_y, mouse_button))
                    && apply_build_action(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        tool_state.as_ref(),
                        grid_x,
                        grid_y,
                        erase,
                    )
                {
                    pointer_state.last_build_cell = Some((grid_x, grid_y, mouse_button));
                    sync_editor_resources(
                        &editor_ship.ship,
                        &progression,
                        editor_session.mode,
                        &mut rollback_state,
                        &mut enemy_editor_state,
                    );
                }
            }
        }
        EditorToolMode::Select => {
            pointer_state.last_build_cell = None;
            if buttons.just_pressed(MouseButton::Left) {
                selection_state.marquee_origin = Some((grid_x, grid_y));
                selection_state.marquee_current = Some((grid_x, grid_y));
            } else if buttons.pressed(MouseButton::Left) {
                selection_state.marquee_current = Some((grid_x, grid_y));
            } else if buttons.just_released(MouseButton::Left) {
                if let Some(origin) = selection_state.marquee_origin {
                    let current = selection_state.marquee_current.unwrap_or(origin);
                    selection_state.selected_module_ids =
                        select_modules_in_rect(&editor_ship.ship, origin, current);
                }
                selection_state.marquee_origin = None;
                selection_state.marquee_current = None;
            }
        }
    }
}

fn apply_build_action(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    tool_state: &EditorToolState,
    grid_x: i32,
    grid_y: i32,
    erase: bool,
) -> bool {
    if erase {
        if let Some(existing) = ship.module_at(grid_x, grid_y).cloned() {
            if mode == EditorMode::Player {
                progression.add_ready_component(existing.kind, existing.variant, 1);
            }
            ship.remove_module_at(grid_x, grid_y);
            return true;
        }
        return false;
    }

    let selected_variant = tool_state
        .selected_variant
        .normalize_for_kind(tool_state.selected_kind);
    if let Some(existing) = ship.module_at_mut(grid_x, grid_y) {
        if existing.kind == tool_state.selected_kind && existing.variant == selected_variant {
            existing.rotation_quadrants = tool_state.selected_rotation % 4;
            existing.channel = tool_state.selected_channel;
            return true;
        }

        if mode == EditorMode::Player
            && !progression.try_consume_ready_component(tool_state.selected_kind, selected_variant)
        {
            return false;
        }
        if mode == EditorMode::Player {
            progression.add_ready_component(existing.kind, existing.variant, 1);
        }
        existing.kind = tool_state.selected_kind;
        existing.variant = selected_variant;
        existing.rotation_quadrants = tool_state.selected_rotation % 4;
        existing.channel = tool_state.selected_channel;
        return true;
    }

    if mode == EditorMode::Player
        && !progression.try_consume_ready_component(tool_state.selected_kind, selected_variant)
    {
        return false;
    }
    let next_id = ship.next_module_id();
    let mut module = ShipModule::new(
        next_id,
        tool_state.selected_kind,
        grid_x,
        grid_y,
        tool_state.selected_rotation % 4,
    );
    module.variant = selected_variant;
    module.channel = tool_state.selected_channel;
    ship.replace_module(module);
    true
}

fn sync_editor_resources(
    ship: &ShipDefinition,
    progression: &Progression,
    mode: EditorMode,
    rollback_state: &mut netcode::RollbackGameState,
    enemy_editor_state: &mut EnemyEditorState,
) {
    if mode == EditorMode::Player {
        rollback_state.editor_ship = ship.clone();
        rollback_state.progression = progression.clone();
    } else {
        enemy_editor_state.dirty = true;
    }
}

fn select_modules_in_rect(
    ship: &ShipDefinition,
    origin: (i32, i32),
    current: (i32, i32),
) -> Vec<u64> {
    let min_x = origin.0.min(current.0);
    let max_x = origin.0.max(current.0);
    let min_y = origin.1.min(current.1);
    let max_y = origin.1.max(current.1);
    ship.modules
        .iter()
        .filter(|module| {
            module.grid_x >= min_x
                && module.grid_x <= max_x
                && module.grid_y >= min_y
                && module.grid_y <= max_y
        })
        .map(|module| module.id)
        .collect()
}

pub(crate) fn toggle_editor_module_overlay_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_ship: Res<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
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
    selection_state.selected_module_ids = vec![module.id];
    tool_state.tool_mode = EditorToolMode::Select;
    tool_state.selected_kind = module.kind;
    tool_state.selected_variant = module.variant;
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
    mut progression: ResMut<Progression>,
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

pub(crate) fn selection_action_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&EditorAutoHullButton>,
            Option<&EditorCopySelectionButton>,
            Option<&EditorPasteSelectionButton>,
            Option<&EditorDeleteSelectionButton>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();
    let camera_query = *camera_query;

    for (interaction, auto_hull, copy, paste, delete, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut changed = false;
                if auto_hull.is_some() {
                    changed = apply_auto_hull_to_ship(&mut editor_ship.ship);
                } else if copy.is_some() {
                    selection_state.clipboard = selected_or_all_modules(
                        &editor_ship.ship,
                        &selection_state.selected_module_ids,
                    )
                    .into_iter()
                    .map(module_snapshot)
                    .collect();
                } else if paste.is_some() {
                    let anchor = cursor_grid_position(window, camera_query)
                        .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
                    changed = paste_clipboard_group(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        &mut selection_state,
                        anchor,
                    );
                } else if delete.is_some() {
                    changed = delete_selected_group(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        &mut selection_state,
                    );
                }

                if changed {
                    sync_editor_resources(
                        &editor_ship.ship,
                        &progression,
                        editor_session.mode,
                        &mut rollback_state,
                        &mut enemy_editor_state,
                    );
                }
                *background = BackgroundColor(PRESSED_BUTTON);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if auto_hull.is_some() {
                    Color::srgb(0.46, 0.36, 0.18)
                } else if copy.is_some() {
                    Color::srgb(0.26, 0.42, 0.62)
                } else if paste.is_some() {
                    Color::srgb(0.22, 0.52, 0.34)
                } else {
                    Color::srgb(0.58, 0.26, 0.18)
                });
            }
        }
    }
}

pub(crate) fn selection_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    tool_state: Res<EditorToolState>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) || tool_state.tool_mode != EditorToolMode::Select {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut changed = false;

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyC) {
        selection_state.clipboard =
            selected_or_all_modules(&editor_ship.ship, &selection_state.selected_module_ids)
                .into_iter()
                .map(module_snapshot)
                .collect();
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyV) {
        let anchor = cursor_grid_position(window.into_inner(), *camera_query)
            .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
        changed |= paste_clipboard_group(
            &mut editor_ship.ship,
            &mut progression,
            editor_session.mode,
            &mut selection_state,
            anchor,
        );
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        changed |= delete_selected_group(
            &mut editor_ship.ship,
            &mut progression,
            editor_session.mode,
            &mut selection_state,
        );
    }

    if keys.just_pressed(KeyCode::KeyH) {
        changed |= apply_auto_hull_to_ship(&mut editor_ship.ship);
    }

    for (key, dx, dy) in [
        (KeyCode::ArrowLeft, -1, 0),
        (KeyCode::ArrowRight, 1, 0),
        (KeyCode::ArrowUp, 0, -1),
        (KeyCode::ArrowDown, 0, 1),
    ] {
        if keys.just_pressed(key) {
            changed |= move_selected_group(&mut editor_ship.ship, &selection_state, dx, dy);
        }
    }

    if changed {
        sync_editor_resources(
            &editor_ship.ship,
            &progression,
            editor_session.mode,
            &mut rollback_state,
            &mut enemy_editor_state,
        );
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

fn selected_or_all_modules(ship: &ShipDefinition, selected_module_ids: &[u64]) -> Vec<ShipModule> {
    ship.modules
        .iter()
        .filter(|module| selected_module_ids.is_empty() || selected_module_ids.contains(&module.id))
        .cloned()
        .collect()
}

fn module_snapshot(module: ShipModule) -> crate::state::ShipModuleSnapshot {
    crate::state::ShipModuleSnapshot {
        kind: module.kind,
        variant: module.variant,
        grid_x: module.grid_x,
        grid_y: module.grid_y,
        rotation_quadrants: module.rotation_quadrants,
        channel: module.channel,
    }
}

fn ship_anchor(ship: &ShipDefinition) -> (i32, i32) {
    ship.bounds()
        .map(|(min_x, _, min_y, _)| (min_x, min_y))
        .unwrap_or((0, 0))
}

fn move_selected_group(
    ship: &mut ShipDefinition,
    selection_state: &EditorSelectionState,
    dx: i32,
    dy: i32,
) -> bool {
    if selection_state.selected_module_ids.is_empty() {
        return false;
    }

    let selected = ship
        .modules
        .iter()
        .filter(|module| selection_state.selected_module_ids.contains(&module.id))
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return false;
    }

    for module in &selected {
        let next_x = module.grid_x + dx;
        let next_y = module.grid_y + dy;
        if let Some(blocker) = ship.module_at(next_x, next_y)
            && !selection_state.selected_module_ids.contains(&blocker.id)
        {
            return false;
        }
    }

    for module in &selected {
        if let Some(target) = ship.modules.iter_mut().find(|entry| entry.id == module.id) {
            target.grid_x += dx;
            target.grid_y += dy;
        }
    }
    true
}

fn delete_selected_group(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    selection_state: &mut EditorSelectionState,
) -> bool {
    let selected = selected_or_all_modules(ship, &selection_state.selected_module_ids);
    if selected.is_empty() {
        return false;
    }

    for module in &selected {
        if mode == EditorMode::Player {
            progression.add_ready_component(module.kind, module.variant, 1);
        }
        ship.modules.retain(|entry| entry.id != module.id);
    }
    selection_state.selected_module_ids.clear();
    true
}

fn paste_clipboard_group(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    selection_state: &mut EditorSelectionState,
    anchor: (i32, i32),
) -> bool {
    if selection_state.clipboard.is_empty() {
        return false;
    }
    let min_x = selection_state
        .clipboard
        .iter()
        .map(|module| module.grid_x)
        .min()
        .unwrap_or(anchor.0);
    let min_y = selection_state
        .clipboard
        .iter()
        .map(|module| module.grid_y)
        .min()
        .unwrap_or(anchor.1);

    let planned = selection_state
        .clipboard
        .iter()
        .map(|module| {
            (
                module,
                anchor.0 + (module.grid_x - min_x),
                anchor.1 + (module.grid_y - min_y),
            )
        })
        .collect::<Vec<_>>();

    for (_, grid_x, grid_y) in &planned {
        if ship.module_at(*grid_x, *grid_y).is_some() {
            return false;
        }
    }

    if mode == EditorMode::Player {
        let mut needed = HashMap::<(ModuleKind, ModuleVariant), u32>::new();
        for (module, _, _) in &planned {
            *needed.entry((module.kind, module.variant)).or_default() += 1;
        }
        if needed
            .into_iter()
            .any(|((kind, variant), amount)| progression.ready_count(kind, variant) < amount)
        {
            return false;
        }
        for (module, _, _) in &planned {
            progression.try_consume_ready_component(module.kind, module.variant);
        }
    }

    selection_state.selected_module_ids.clear();
    for (module, grid_x, grid_y) in planned {
        let mut next = ShipModule::new(
            ship.next_module_id(),
            module.kind,
            grid_x,
            grid_y,
            module.rotation_quadrants,
        );
        next.variant = module.variant;
        next.channel = module.channel;
        let new_id = next.id;
        ship.replace_module(next);
        selection_state.selected_module_ids.push(new_id);
    }
    true
}

/// Rebuilds the derived hull shell around all non-hull structure modules in the editor ship.
fn apply_auto_hull_to_ship(ship: &mut ShipDefinition) -> bool {
    let base = ship
        .modules
        .iter()
        .filter(|module| !is_generated_hull_kind(module.kind))
        .cloned()
        .collect::<Vec<_>>();
    if base.is_empty() {
        return false;
    }

    let occupied = base
        .iter()
        .filter(|module| !is_manual_hull_kind(module.kind))
        .map(|module| (module.grid_x, module.grid_y))
        .collect::<HashSet<_>>();

    let existing_hull = ship
        .modules
        .iter()
        .filter(|module| is_manual_hull_kind(module.kind))
        .map(|module| {
            (
                (module.grid_x, module.grid_y),
                (module.kind, module.rotation_quadrants),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut candidates = HashSet::new();
    for module in &base {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let target = (module.grid_x + dx, module.grid_y + dy);
                if !occupied.contains(&target) {
                    candidates.insert(target);
                }
            }
        }
    }

    let mut desired_hull = candidates
        .into_iter()
        .filter_map(|target| {
            auto_hull_kind_for_cell(&occupied, target)
                .map(|(kind, rotation)| (target, (kind, rotation)))
        })
        .collect::<Vec<_>>();
    desired_hull.sort_by_key(|((x, y), _)| (*y, *x));

    ship.modules
        .retain(|module| !is_generated_hull_kind(module.kind));

    for ((grid_x, grid_y), (kind, rotation)) in desired_hull {
        if existing_hull.contains_key(&(grid_x, grid_y)) {
            continue;
        }
        let variant = ModuleVariant::default_for_kind(kind);
        let next_id = ship.next_module_id();
        let mut hull = ShipModule::new(next_id, kind, grid_x, grid_y, rotation);
        hull.variant = variant;
        ship.replace_module(hull);
    }

    true
}

fn is_generated_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )
}

fn is_manual_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull
            | ModuleKind::HullInnerCorner
            | ModuleKind::HullOuterCorner
            | ModuleKind::Airlock
            | ModuleKind::Engine
            | ModuleKind::Turret
    )
}

/// Chooses the hull piece and orientation that best wraps one empty shell cell around structure.
fn auto_hull_kind_for_cell(
    occupied: &HashSet<(i32, i32)>,
    target: (i32, i32),
) -> Option<(ModuleKind, u8)> {
    let north = occupied.contains(&(target.0, target.1 - 1));
    let south = occupied.contains(&(target.0, target.1 + 1));
    let west = occupied.contains(&(target.0 - 1, target.1));
    let east = occupied.contains(&(target.0 + 1, target.1));
    let northeast = occupied.contains(&(target.0 + 1, target.1 - 1));
    let northwest = occupied.contains(&(target.0 - 1, target.1 - 1));
    let southeast = occupied.contains(&(target.0 + 1, target.1 + 1));
    let southwest = occupied.contains(&(target.0 - 1, target.1 + 1));

    if (south && north) || (east && west) {
        None
    } else if north && east && northeast {
        Some((ModuleKind::HullInnerCorner, 0))
    } else if south && east && southeast {
        Some((ModuleKind::HullInnerCorner, 1))
    } else if south && west && southwest {
        Some((ModuleKind::HullInnerCorner, 2))
    } else if north && west && northwest {
        Some((ModuleKind::HullInnerCorner, 3))
    } else if south {
        Some((ModuleKind::Hull, 0))
    } else if west {
        Some((ModuleKind::Hull, 1))
    } else if north {
        Some((ModuleKind::Hull, 2))
    } else if east {
        Some((ModuleKind::Hull, 3))
    } else if southeast {
        Some((ModuleKind::HullOuterCorner, 0))
    } else if southwest {
        Some((ModuleKind::HullOuterCorner, 1))
    } else if northwest {
        Some((ModuleKind::HullOuterCorner, 2))
    } else if northeast {
        Some((ModuleKind::HullOuterCorner, 3))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_auto_hull_to_ship, paste_clipboard_group};
    use crate::{
        ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule},
        state::{EditorMode, EditorSelectionState, Progression, ShipModuleSnapshot},
    };

    #[test]
    fn auto_hull_adds_shell_tiles_around_structure() {
        let mut ship = ShipDefinition::empty("Hull Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        assert!(ship.modules.iter().any(|module| {
            matches!(
                module.kind,
                ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
            )
        }));
    }

    #[test]
    fn auto_hull_builds_edges_and_outer_corners_around_single_component() {
        let mut ship = ShipDefinition::empty("Outer Shell Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let shell = ship
            .modules
            .iter()
            .filter(|module| module.kind != ModuleKind::Core)
            .map(|module| {
                (
                    (module.grid_x, module.grid_y),
                    (module.kind, module.rotation_quadrants),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(
            shell.get(&(-1, -1)),
            Some(&(ModuleKind::HullOuterCorner, 0))
        );
        assert_eq!(shell.get(&(0, -1)), Some(&(ModuleKind::Hull, 0)));
        assert_eq!(shell.get(&(1, -1)), Some(&(ModuleKind::HullOuterCorner, 1)));
        assert_eq!(shell.get(&(-1, 0)), Some(&(ModuleKind::Hull, 3)));
        assert_eq!(shell.get(&(1, 0)), Some(&(ModuleKind::Hull, 1)));
        assert_eq!(shell.get(&(-1, 1)), Some(&(ModuleKind::HullOuterCorner, 3)));
        assert_eq!(shell.get(&(0, 1)), Some(&(ModuleKind::Hull, 2)));
        assert_eq!(shell.get(&(1, 1)), Some(&(ModuleKind::HullOuterCorner, 2)));
    }

    #[test]
    fn auto_hull_uses_inner_corner_for_l_shaped_structure() {
        let mut ship = ShipDefinition::empty("Inner Corner Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));
        ship.replace_module(ShipModule::new(3, ModuleKind::Interior, 0, 1, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let corner = ship
            .module_at(1, 1)
            .expect("expected derived inner corner at the L-shape notch");
        assert_eq!(corner.kind, ModuleKind::HullInnerCorner);
        assert_eq!(corner.rotation_quadrants, 2);
    }

    #[test]
    fn pasting_group_consumes_variant_inventory_in_player_mode() {
        let mut ship = ShipDefinition::empty("Paste Test");
        let mut progression = Progression::default();
        progression.add_ready_component(ModuleKind::Turret, ModuleVariant::BallisticTurret, 1);
        let mut selection_state = EditorSelectionState {
            clipboard: vec![ShipModuleSnapshot {
                kind: ModuleKind::Turret,
                variant: ModuleVariant::BallisticTurret,
                grid_x: 0,
                grid_y: 0,
                rotation_quadrants: 0,
                channel: 2,
            }],
            ..Default::default()
        };

        let pasted = paste_clipboard_group(
            &mut ship,
            &mut progression,
            EditorMode::Player,
            &mut selection_state,
            (4, 5),
        );

        assert!(pasted);
        assert_eq!(
            progression.ready_count(ModuleKind::Turret, ModuleVariant::BallisticTurret),
            0
        );
        assert!(ship.module_at(4, 5).is_some());
    }
}
