use super::*;

pub(super) fn sync_editor_resources(
    ship: &ShipDefinition,
    progression: &Progression,
    mode: EditorMode,
    rollback_state: &mut netcode::RollbackGameState,
    enemy_editor_state: &mut EnemyEditorState,
) {
    match mode {
        EditorMode::Player => {
            rollback_state.editor_ship = ship.clone();
            rollback_state.progression = progression.clone();
        }
        EditorMode::Enemy => {
            enemy_editor_state.dirty = true;
        }
        EditorMode::Station => {}
    }
}

pub(super) fn select_modules_in_rect(
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

/// Opens and closes module inspection panels from the grid so part configuration stays contextual.
pub(crate) fn toggle_editor_module_overlay_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_ship: Res<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
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

/// Routes station-panel clicks in the editor to part-specific configuration state changes.
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
                    StationPanelButtonAction::HelmThrottle { .. }
                    | StationPanelButtonAction::HelmTurn { .. } => {}
                    StationPanelButtonAction::TurretAdjustAim { .. } => {}
                    StationPanelButtonAction::TurretFireToggle => {
                        module.defaults.turret_fire_intent = !module.defaults.turret_fire_intent;
                    }
                    StationPanelButtonAction::ReactorAdjustRate { delta } => {
                        let current = module.defaults.reaction_rate_milli as i32;
                        module.defaults.reaction_rate_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    StationPanelButtonAction::ReactorAdjustTurbine { delta } => {
                        let current = module.defaults.turbine_load_milli as i32;
                        module.defaults.turbine_load_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    StationPanelButtonAction::LogisticsToggleStorageIntake => {
                        module.defaults.storage_allow_intake =
                            !module.defaults.storage_allow_intake;
                    }
                    StationPanelButtonAction::LogisticsToggleAirlock => {
                        module.defaults.airlock_open = !module.defaults.airlock_open;
                    }
                    StationPanelButtonAction::InfrastructureToggleBlocker => {
                        // Runtime-only until blocker defaults are persisted in ship prefab data.
                    }
                    StationPanelButtonAction::LogisticsToggleManipulator => {
                        module.defaults.manipulator_transfer_enabled =
                            !module.defaults.manipulator_transfer_enabled;
                    }
                    StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => {
                        module.defaults.manipulator_manual_mode =
                            !module.defaults.manipulator_manual_mode;
                    }
                    StationPanelButtonAction::LogisticsCycleResource => {
                        module.defaults.manipulator_resource_kind =
                            module.defaults.manipulator_resource_kind.next();
                    }
                    StationPanelButtonAction::LogisticsToggleProcessor => {
                        if module.kind == ModuleKind::Processor {
                            module.defaults.processor_enabled = !module.defaults.processor_enabled;
                        } else {
                            module.defaults.processor_recipe =
                                module.defaults.processor_recipe.next();
                        }
                    }
                    StationPanelButtonAction::ComputerToggleEnabled => {
                        module.defaults.computer_enabled = !module.defaults.computer_enabled;
                    }
                    StationPanelButtonAction::ComputerCycleTemplate => {
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => {
                                let program = module.arch_program.get_or_insert_with(|| {
                                    ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                                });
                                *program = ArchProgram::from_template(program.template.next());
                            }
                            ProgrammingLanguageMode::Lumen => {
                                let program = module.lumen_program.get_or_insert_with(|| {
                                    LumenProgram::from_template(
                                        LumenProgramTemplate::BalancedSupervision,
                                    )
                                });
                                *program = LumenProgram::from_template(program.template.next());
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

/// Repairs the selected player-owned component in refit mode so damaged inventory can be reused.
pub(crate) fn repair_selected_component_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut progression: ResMut<Progression>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
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

/// Handles selection panel buttons for auto-hull, copy, paste, and delete group editing actions.
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
    tool_state: Res<EditorToolState>,
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
                    if !selection_state.selected_foundation_ids.is_empty() {
                        selection_state.foundation_clipboard = editor_ship
                            .ship
                            .foundation_tiles
                            .iter()
                            .chain(editor_ship.ship.hull_tiles.iter())
                            .filter(|tile| {
                                selection_state.selected_foundation_ids.contains(&tile.id)
                            })
                            .cloned()
                            .map(foundation_snapshot)
                            .collect();
                    } else {
                        selection_state.clipboard = selected_or_all_modules(
                            &editor_ship.ship,
                            &selection_state.selected_module_ids,
                        )
                        .into_iter()
                        .map(module_snapshot)
                        .collect();
                    }
                } else if paste.is_some() {
                    let anchor = cursor_grid_position(window, camera_query)
                        .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
                    changed = if !selection_state.foundation_clipboard.is_empty() {
                        paste_foundation_clipboard_group(
                            &mut editor_ship.ship,
                            &mut selection_state,
                            anchor,
                        )
                    } else {
                        paste_clipboard_group(
                            &mut editor_ship.ship,
                            &mut progression,
                            editor_session.mode,
                            tool_state.ignore_component_limits,
                            &mut selection_state,
                            anchor,
                        )
                    };
                } else if delete.is_some() {
                    changed = delete_selected_group(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        tool_state.ignore_component_limits,
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

/// Provides keyboard shortcuts for moving, copying, pasting, and rebuilding editor selections.
pub(crate) fn selection_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
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
    if focused_textbox.field.is_some() {
        return;
    }
    if !netcode::is_host_authority(&status) || tool_state.tool_mode != EditorToolMode::Select {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut changed = false;

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyC) {
        if !selection_state.selected_foundation_ids.is_empty() {
            selection_state.foundation_clipboard = editor_ship
                .ship
                .foundation_tiles
                .iter()
                .chain(editor_ship.ship.hull_tiles.iter())
                .filter(|tile| selection_state.selected_foundation_ids.contains(&tile.id))
                .cloned()
                .map(foundation_snapshot)
                .collect();
        } else {
            selection_state.clipboard =
                selected_or_all_modules(&editor_ship.ship, &selection_state.selected_module_ids)
                    .into_iter()
                    .map(module_snapshot)
                    .collect();
        }
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyV) {
        let anchor = cursor_grid_position(window.into_inner(), *camera_query)
            .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
        changed |= if !selection_state.foundation_clipboard.is_empty() {
            paste_foundation_clipboard_group(&mut editor_ship.ship, &mut selection_state, anchor)
        } else {
            paste_clipboard_group(
                &mut editor_ship.ship,
                &mut progression,
                editor_session.mode,
                tool_state.ignore_component_limits,
                &mut selection_state,
                anchor,
            )
        };
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        changed |= delete_selected_group(
            &mut editor_ship.ship,
            &mut progression,
            editor_session.mode,
            tool_state.ignore_component_limits,
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
            changed |= if !selection_state.selected_foundation_ids.is_empty() {
                move_selected_foundation_group(&mut editor_ship.ship, &selection_state, dx, dy)
            } else {
                move_selected_group(&mut editor_ship.ship, &selection_state, dx, dy)
            };
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
