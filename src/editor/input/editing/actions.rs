use super::*;

/// Handles toolbox clicks so the editor's active tool, layer, and selected part follow UI intent.
pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ToolboxVariantButton>,
            Option<&ToolboxFoundationButton>,
            Option<&EditorToolModeButton>,
            Option<&EditorLayerButton>,
            Option<&StationToolboxButton>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    progression: Res<Progression>,
    mut tool_state: ResMut<EditorToolState>,
    mut editor_ui_state: ResMut<EditorUiState>,
    mut station_editor_state: ResMut<station_editor::StationEditorState>,
) {
    for (
        interaction,
        variant_button,
        foundation_button,
        mode_button,
        layer_button,
        station_button,
        mut background,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(button) = mode_button {
                    if button.mode == EditorToolMode::Select {
                        station_editor_state.selected_tool = None;
                    }
                    tool_state.tool_mode = button.mode;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = layer_button {
                    station_editor_state.selected_tool = None;
                    tool_state.active_layer = button.layer;
                    tool_state.tool_mode = EditorToolMode::Build;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = foundation_button {
                    station_editor_state.selected_tool = None;
                    tool_state.active_layer = if is_hull_foundation_kind(button.kind) {
                        EditorLayer::Hull
                    } else {
                        EditorLayer::Logistics
                    };
                    tool_state.tool_mode = EditorToolMode::Build;
                    tool_state.selected_foundation_kind = button.kind;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = variant_button {
                    let available = tool_state.ignore_component_limits
                        || editor_session.mode == EditorMode::Enemy
                        || editor_session.mode == EditorMode::Station
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    if !available {
                        *background = BackgroundColor(SELECTED_UNAFFORDABLE_BUTTON);
                        continue;
                    }
                    tool_state.tool_mode = EditorToolMode::Build;
                    station_editor_state.selected_tool = None;
                    tool_state.active_layer = if module_belongs_to_hull_layer(button.kind) {
                        EditorLayer::Hull
                    } else {
                        EditorLayer::Components
                    };
                    tool_state.selected_kind = button.kind;
                    tool_state.selected_variant = button.variant;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = station_button {
                    tool_state.tool_mode = EditorToolMode::Build;
                    tool_state.active_layer = EditorLayer::Components;
                    station_editor_state.selected_tool = Some(button.tool);
                    station_editor_state.status = format!("Selected {} tool.", button.tool.label());
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
                    let available = tool_state.ignore_component_limits
                        || editor_session.mode == EditorMode::Enemy
                        || editor_session.mode == EditorMode::Station
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    *background = BackgroundColor(if available {
                        HOVERED_BUTTON
                    } else {
                        UNAFFORDABLE_BUTTON
                    });
                } else if let Some(button) = foundation_button {
                    editor_ui_state.toolbox_tooltip.title = format!(
                        "{} / {}",
                        foundation_family_label(button.kind),
                        button.kind.as_str()
                    );
                    editor_ui_state.toolbox_tooltip.detail =
                        "Logistics tile. It can share a cell with hull and component placements."
                            .to_string();
                    *background = BackgroundColor(HOVERED_BUTTON);
                } else if let Some(button) = station_button {
                    editor_ui_state.toolbox_tooltip.title = "Station".to_string();
                    editor_ui_state.toolbox_tooltip.detail =
                        format!("{} placement tool", button.tool.label());
                    *background = BackgroundColor(HOVERED_BUTTON);
                } else {
                    *background = BackgroundColor(HOVERED_BUTTON);
                }
            }
            Interaction::None => {}
        }
    }
}

/// Leaves the current editor flow from the UI so player refit or enemy debug editing can close cleanly.
pub(crate) fn leave_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LeaveEditorButton>),
    >,
    editor_session: Res<EditorSessionState>,
    editor_ship: Res<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut stations: ResMut<StationCatalogResource>,
    station_editor_state: Res<station_editor::StationEditorState>,
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
                        let saved =
                            sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                                && save_enemy_library_if_valid(&enemy_library_state);
                        enemy_editor_state.dirty = !saved;
                        next_mode.set(FrontendMode::Lobby);
                    }
                    EditorMode::Station => {
                        if let Err(error) = station_editor::save_station_editor_catalog(
                            &mut stations,
                            &station_editor_state,
                            &editor_ship,
                        ) {
                            bevy::log::warn!("Station editor exit save failed: {}", error);
                        }
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

/// Provides a keyboard exit path from the editor so navigation matches the UI button behavior.
pub(crate) fn leave_editor_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    editor_ship: Res<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut stations: ResMut<StationCatalogResource>,
    station_editor_state: Res<station_editor::StationEditorState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
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
                let saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                    && save_enemy_library_if_valid(&enemy_library_state);
                enemy_editor_state.dirty = !saved;
                next_mode.set(FrontendMode::Lobby);
            }
            EditorMode::Station => {
                if let Err(error) = station_editor::save_station_editor_catalog(
                    &mut stations,
                    &station_editor_state,
                    &editor_ship,
                ) {
                    bevy::log::warn!("Station editor exit save failed: {}", error);
                }
                next_mode.set(FrontendMode::Lobby);
            }
        }
    }
}

/// Expands or collapses the mission report panel so recent sortie feedback is available on demand.
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
                *background = BackgroundColor(NORMAL_BUTTON);
            }
        }
    }
}
