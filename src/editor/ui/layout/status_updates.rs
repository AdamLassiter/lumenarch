use bevy::{ecs::relationship::Relationship, prelude::*};

use super::station_panel::{
    EditorReadoutVisual,
    editor_control_mode,
    editor_station_action_visible,
    editor_station_button_label,
    editor_station_flags,
    editor_station_readouts,
    enemy_entry_label,
    format_program_textbox,
};
use crate::{
    helpers::editor::{
        editor_mission_report_text,
        editor_status_line,
        enemy_config_references_text,
        selection_summary,
        variant_tooltip_text,
    },
    ship::ModuleKind,
    state::{
        ArchEditorState,
        EditorBuildSection,
        EditorComponentsBuildSection,
        EditorHullBuildSection,
        EditorLayer,
        EditorLogisticsBuildSection,
        EditorMissionReportButtonText,
        EditorMissionReportText,
        EditorSelectSection,
        EditorSelectionState,
        EditorSelectionSummaryText,
        EditorSessionState,
        EditorShip,
        EditorStatusText,
        EditorToolMode,
        EditorToolState,
        EditorToolboxTooltipText,
        EditorUiState,
        EnemyConfigReferencesText,
        EnemyEditorState,
        EnemyShipLibraryState,
        GameplayStationPanel,
        GameplayStationPanelButton,
        GameplayStationPanelButtonLabel,
        GameplayStationReadoutBarFill,
        GameplayStationReadoutBarTrack,
        GameplayStationReadoutLabel,
        GameplayStationReadoutLight,
        GameplayStationReadoutSlot,
        GameplayStationReadoutValue,
        GameplayStationTitleText,
        LastMissionReport,
        ProgramEditorAction,
        ProgramEditorActionButton,
        ProgramEditorDiagnosticsText,
        ProgramEditorDraftText,
        ProgramEditorStatusText,
        ProgramEditorTextBox,
        ProgramTextEditorState,
        ProgrammingLanguageMode,
        Progression,
        SectorState,
        StationConfigReferencesText,
    },
    station_editor::{self, StationEditorState},
    stations::StationCatalogResource,
};

/// Refreshes editor HUD text and section visibility so the current tool, ship state, and selection stay legible.
pub(crate) fn update_editor_status_text(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    enemy_editor_state: Res<EnemyEditorState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
    selection_state: Res<EditorSelectionState>,
    tool_state: Res<EditorToolState>,
    progression: Res<Progression>,
    sector_state: Res<SectorState>,
    station_catalog: Res<StationCatalogResource>,
    station_editor_state: Res<StationEditorState>,
    last_mission_report: Res<LastMissionReport>,
    editor_ui_state: Res<EditorUiState>,
    mut ui_queries: ParamSet<(
        Query<&'static mut Text, With<EditorStatusText>>,
        Query<
            (&'static mut Node, &'static mut Text),
            (With<EditorMissionReportText>, Without<EditorStatusText>),
        >,
        Query<
            &'static mut Text,
            (
                With<EditorMissionReportButtonText>,
                Without<EditorStatusText>,
                Without<EditorMissionReportText>,
            ),
        >,
        Query<&'static mut Text, With<EditorSelectionSummaryText>>,
        Query<&'static mut Text, With<EditorToolboxTooltipText>>,
        Query<&'static mut Node, With<EditorBuildSection>>,
        Query<&'static mut Node, With<EditorSelectSection>>,
        Query<&'static mut Text, With<EnemyConfigReferencesText>>,
    )>,
) {
    // SAFETY: Each branch targets a different editor UI marker or explicitly excludes overlapping markers;
    // `ParamSet` sequences mutable access so no text/node entity can be borrowed by two branches at once.
    if !editor_ship.is_changed()
        && !tool_state.is_changed()
        && !selection_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
        && !editor_ui_state.is_changed()
        && !enemy_editor_state.is_changed()
        && !enemy_library_state.is_changed()
        && !station_catalog.is_changed()
        && !station_editor_state.is_changed()
        && !sector_state.is_changed()
    {
        return;
    }

    for mut text in &mut ui_queries.p0() {
        let entry_label = if editor_session.mode == crate::state::EditorMode::Station {
            station_editor::station_entry_label(
                &editor_session,
                &station_catalog,
                &station_editor_state,
            )
        } else {
            enemy_entry_label(&editor_session, &enemy_editor_state, &enemy_library_state)
        };
        **text = editor_status_line(
            editor_session.mode,
            tool_state.tool_mode,
            tool_state.active_layer,
            &entry_label,
            &editor_ship.ship.name,
            &tool_state.selected_kind,
            tool_state.selected_foundation_kind,
            tool_state.selected_variant,
            tool_state.selected_rotation,
            tool_state.selected_channel,
            tool_state.ignore_component_limits,
            editor_ship.ship.modules.len(),
            progression.scrap,
            &progression,
            &editor_ship.ship,
            &selection_state,
        );
    }

    for (mut node, mut text) in &mut ui_queries.p1() {
        node.display = if editor_ui_state.mission_report_expanded {
            Display::Flex
        } else {
            Display::None
        };
        **text = editor_mission_report_text(&last_mission_report);
    }

    for mut text in &mut ui_queries.p2() {
        **text = if editor_ui_state.mission_report_expanded {
            "Hide Last Mission".to_string()
        } else {
            "Show Last Mission".to_string()
        };
    }

    for mut text in &mut ui_queries.p3() {
        **text = format!(
            "Selection\n{}",
            selection_summary(&editor_ship.ship, &selection_state)
        );
    }

    for mut text in &mut ui_queries.p4() {
        **text = if editor_ui_state.toolbox_tooltip.title.is_empty() {
            variant_tooltip_text(
                editor_session.mode,
                &progression,
                tool_state.selected_kind,
                tool_state.selected_variant,
            )
        } else {
            format!(
                "{}\n{}",
                editor_ui_state.toolbox_tooltip.title, editor_ui_state.toolbox_tooltip.detail
            )
        };
    }

    for mut node in &mut ui_queries.p5() {
        node.display = if tool_state.tool_mode == EditorToolMode::Build {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut ui_queries.p6() {
        node.display = if tool_state.tool_mode == EditorToolMode::Select {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut text in &mut ui_queries.p7() {
        **text = enemy_config_references_text(&sector_state, &enemy_library_state);
    }
}

/// Refreshes the station reference list in the station editor catalog panel.
pub(crate) fn update_station_config_references_text(
    sector_state: Res<SectorState>,
    station_catalog: Res<StationCatalogResource>,
    mut query: Query<&mut Text, With<StationConfigReferencesText>>,
) {
    if !sector_state.is_changed() && !station_catalog.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = station_editor::station_config_references_text(&sector_state, &station_catalog);
    }
}

/// Shows only the active build layer section so logistics, hull, and component palettes do not compete visually.
///
/// SAFETY: Each toolbox section has its own marker component, and `ParamSet` ensures the three mutable node
/// queries are accessed one at a time.
pub(crate) fn sync_editor_toolbox_layer_sections(
    tool_state: Res<EditorToolState>,
    mut section_queries: ParamSet<(
        Query<'_, '_, &'static mut Node, With<EditorLogisticsBuildSection>>,
        Query<'_, '_, &'static mut Node, With<EditorHullBuildSection>>,
        Query<'_, '_, &'static mut Node, With<EditorComponentsBuildSection>>,
    )>,
) {
    if !tool_state.is_changed() {
        return;
    }

    for mut node in &mut section_queries.p0() {
        node.display = if tool_state.tool_mode == EditorToolMode::Build
            && tool_state.active_layer == EditorLayer::Logistics
        {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut section_queries.p1() {
        node.display = if tool_state.tool_mode == EditorToolMode::Build
            && tool_state.active_layer == EditorLayer::Hull
        {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut section_queries.p2() {
        node.display = if tool_state.tool_mode == EditorToolMode::Build
            && tool_state.active_layer == EditorLayer::Components
        {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Syncs the editor-side station console overlay so module defaults and program text stay editable in context.
pub(crate) fn update_editor_module_overlay(
    editor_ship: Res<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    program_editor_state: Res<ProgramTextEditorState>,
    mut panel_query: Query<
        &mut Node,
        (
            With<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut title_query: Query<
        &mut Text,
        (
            With<GameplayStationTitleText>,
            Without<ProgramEditorStatusText>,
            Without<ProgramEditorDiagnosticsText>,
            Without<ProgramEditorDraftText>,
            Without<GameplayStationPanelButtonLabel>,
            Without<GameplayStationReadoutLabel>,
            Without<GameplayStationReadoutValue>,
        ),
    >,
    mut button_query: Query<
        (
            &GameplayStationPanelButton,
            &mut Node,
            &Children,
            &mut BackgroundColor,
        ),
        (
            Without<ProgramEditorActionButton>,
            Without<GameplayStationPanel>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
        ),
    >,
    mut slot_query: Query<
        (Entity, &GameplayStationReadoutSlot, &mut Node, &Children),
        (
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutBarFill>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut button_label_query: Query<
        (&GameplayStationPanelButtonLabel, &mut Text),
        (
            Without<GameplayStationTitleText>,
            Without<ProgramEditorDraftText>,
            Without<ProgramEditorStatusText>,
            Without<ProgramEditorDiagnosticsText>,
            Without<GameplayStationReadoutLabel>,
            Without<GameplayStationReadoutValue>,
        ),
    >,
    mut text_query: ParamSet<(
        Query<'_, '_, &'static mut Text, With<ProgramEditorDraftText>>,
        Query<'_, '_, &'static mut Text, With<ProgramEditorStatusText>>,
        Query<'_, '_, &'static mut Text, With<ProgramEditorDiagnosticsText>>,
        Query<
            '_,
            '_,
            (&'static ChildOf, &'static mut Text),
            (
                With<GameplayStationReadoutLabel>,
                Without<GameplayStationReadoutValue>,
                Without<GameplayStationTitleText>,
                Without<GameplayStationPanelButtonLabel>,
                Without<ProgramEditorDraftText>,
                Without<ProgramEditorStatusText>,
                Without<ProgramEditorDiagnosticsText>,
            ),
        >,
        Query<
            '_,
            '_,
            (&'static ChildOf, &'static mut Text),
            (
                With<GameplayStationReadoutValue>,
                Without<GameplayStationReadoutLabel>,
                Without<GameplayStationTitleText>,
                Without<GameplayStationPanelButtonLabel>,
                Without<ProgramEditorDraftText>,
                Without<ProgramEditorStatusText>,
                Without<ProgramEditorDiagnosticsText>,
            ),
        >,
    )>,
    mut bar_track_query: Query<
        (Entity, &ChildOf, &mut Node, &Children),
        (
            With<GameplayStationReadoutBarTrack>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarFill>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut bar_fill_query: Query<
        (&ChildOf, &mut Node, &mut BackgroundColor),
        (
            With<GameplayStationReadoutBarFill>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut light_query: Query<
        (&ChildOf, &mut Node, &mut BackgroundColor),
        (
            With<GameplayStationReadoutLight>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutBarFill>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut program_box_query: Query<
        &mut Node,
        (
            With<ProgramEditorTextBox>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut program_action_query: Query<
        (&ProgramEditorActionButton, &mut Node),
        (
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
        ),
    >,
) {
    // SAFETY: Program editor text, station readout labels, and station readout values use disjoint markers
    // and the `ParamSet` branches are consumed one at a time, preventing double mutable UI access.
    for mut panel_node in &mut panel_query {
        panel_node.display = if arch_editor_state.panel_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !arch_editor_state.panel_open {
        return;
    }

    let Some(module_id) = arch_editor_state.selected_module_id else {
        return;
    };
    let Some(module) = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.id == module_id)
    else {
        return;
    };

    let mode = editor_control_mode(module.kind);
    let flags = editor_station_flags(module.kind);
    let title = if module.kind.supports_channel() {
        format!(
            "{} Console  [CH{}]",
            module.kind.as_str(),
            module.effective_channel()
        )
    } else {
        format!("{} Console", module.kind.as_str())
    };
    for mut text in &mut title_query {
        **text = title.clone();
    }

    let readouts = editor_station_readouts(module);
    let show_program_editor = module.kind == ModuleKind::Computer;

    for mut node in &mut program_box_query {
        node.display = if show_program_editor {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (action, mut node) in &mut program_action_query {
        node.display = if show_program_editor
            || !matches!(
                action.action,
                ProgramEditorAction::SwitchArch
                    | ProgramEditorAction::SwitchLumen
                    | ProgramEditorAction::Check
                    | ProgramEditorAction::Apply
                    | ProgramEditorAction::Revert
            ) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut text in &mut text_query.p0() {
        **text = if show_program_editor {
            format_program_textbox(
                &program_editor_state,
                module_id,
                arch_editor_state.selected_language,
                module,
            )
        } else {
            String::new()
        };
    }
    for mut text in &mut text_query.p1() {
        **text = if show_program_editor {
            if program_editor_state.status_line.is_empty() {
                format!(
                    "{} source",
                    match arch_editor_state.selected_language {
                        ProgrammingLanguageMode::Arch => "ARCH",
                        ProgrammingLanguageMode::Lumen => "LUMEN",
                    }
                )
            } else {
                program_editor_state.status_line.clone()
            }
        } else {
            String::new()
        };
    }
    for mut text in &mut text_query.p2() {
        **text = if show_program_editor && !program_editor_state.diagnostics.is_empty() {
            format!(
                "Diagnostics\n{}",
                program_editor_state.diagnostics.join("\n")
            )
        } else if show_program_editor {
            "Diagnostics\nNo parse diagnostics".to_string()
        } else {
            String::new()
        };
    }

    for (button, mut node, children, mut color) in &mut button_query {
        let visible = editor_station_action_visible(button.action, mode, module.kind, flags);
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *color = BackgroundColor(if visible {
            Color::srgb(0.24, 0.38, 0.58)
        } else {
            Color::srgba(0.24, 0.38, 0.58, 0.18)
        });
        for child in children.iter() {
            if let Ok((label, mut text)) = button_label_query.get_mut(child) {
                **text = editor_station_button_label(
                    label.action,
                    mode,
                    flags,
                    arch_editor_state.selected_language,
                );
            }
        }
    }

    let mut readout_rows = Vec::new();
    for (row_entity, slot, mut row_node, children) in &mut slot_query {
        let readout = readouts.get(slot.index as usize).cloned();
        row_node.display = if readout.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(readout) = readout {
            readout_rows.push((row_entity, children.to_vec(), readout));
        }
    }

    for (row_entity, children, readout) in readout_rows {
        for child in children {
            if let Ok((parent, mut text)) = text_query.p3().get_mut(child)
                && parent.get() == row_entity
            {
                **text = readout.label.clone();
            }
            if let Ok((parent, mut text)) = text_query.p4().get_mut(child)
                && parent.get() == row_entity
            {
                **text = readout.value.clone();
            }
            let fill_update = if let Ok((track_entity, parent, mut node, track_children)) =
                bar_track_query.get_mut(child)
                && parent.get() == row_entity
            {
                let EditorReadoutVisual::Bar {
                    percent,
                    color: fill,
                } = readout.visual
                else {
                    node.display = Display::None;
                    continue;
                };

                node.display = Display::Flex;
                Some((track_entity, track_children.to_vec(), percent, fill))
            } else {
                None
            };

            if let Some((track_entity, track_children, percent, fill)) = fill_update {
                for track_child in track_children {
                    if let Ok((parent, mut node, mut color)) = bar_fill_query.get_mut(track_child)
                        && parent.get() == track_entity
                    {
                        node.width = Val::Percent(percent);
                        *color = BackgroundColor(fill);
                        node.display = Display::Flex;
                    }
                }
            }
            if let Ok((parent, mut node, mut color)) = light_query.get_mut(child)
                && parent.get() == row_entity
            {
                if let EditorReadoutVisual::Light { color: light } = readout.visual {
                    node.display = Display::Flex;
                    *color = BackgroundColor(light);
                } else {
                    node.display = Display::None;
                }
            }
        }
    }
}
