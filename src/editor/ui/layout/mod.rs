mod spawn;
mod station_panel;
mod status_updates;
mod toolbox;

use bevy::prelude::*;
pub(crate) use station_panel::cleanup_editor_entities;
use station_panel::{enemy_entry_label, station_actions, station_button_default_label};
pub(crate) use status_updates::{
    sync_editor_toolbox_layer_sections,
    update_editor_module_overlay,
    update_editor_status_text,
    update_station_config_references_text,
};
use toolbox::{
    HullToolboxGroup,
    component_toolbox_groups,
    hull_toolbox_groups,
    logistics_toolbox_groups,
    spawn_foundation_button_grid,
    spawn_layer_button,
    spawn_select_action_button,
    spawn_station_tool_button_grid,
    spawn_tool_mode_button,
    spawn_variant_button_grid,
};

use crate::{
    NORMAL_BUTTON,
    TOOLBOX_WIDTH,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    helpers::editor::{
        editor_mission_report_text,
        editor_status_line,
        enemy_config_references_text,
        selection_summary,
        variant_tooltip_text,
    },
    lobby::spawn_textbox,
    state::{
        ControlsHelpPanel,
        EditingCleanup,
        EditorAutoHullButton,
        EditorBuildSection,
        EditorComponentsBuildSection,
        EditorCopySelectionButton,
        EditorDeleteSelectionButton,
        EditorHullBuildSection,
        EditorLayer,
        EditorLogisticsBuildSection,
        EditorMissionReportButton,
        EditorMissionReportButtonText,
        EditorMissionReportText,
        EditorMode,
        EditorPasteSelectionButton,
        EditorPlacementBlocker,
        EditorRoot,
        EditorSelectSection,
        EditorSelectionState,
        EditorSelectionSummaryText,
        EditorSessionState,
        EditorShip,
        EditorStatusText,
        EditorToolMode,
        EditorToolState,
        EditorToolboxPanel,
        EditorToolboxScrollContent,
        EditorToolboxScrollViewport,
        EditorToolboxTooltipText,
        EditorUiState,
        EnemyConfigReferencesText,
        EnemyEditorState,
        EnemyNextButton,
        EnemyPrevButton,
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
        LeaveEditorButton,
        ProgramEditorAction,
        ProgramEditorActionButton,
        ProgramEditorDiagnosticsText,
        ProgramEditorDraftText,
        ProgramEditorStatusText,
        ProgramEditorTextBox,
        Progression,
        SectorState,
        StationConfigReferencesText,
        StationNextButton,
        StationPrevButton,
        TextBoxField,
    },
    station_editor::{self, StationEditorState},
    stations::StationCatalogResource,
};

/// Reports whether the editor UI is absent so spawn chains only run when the workspace is not already open.
pub(crate) fn editor_ui_missing(query: Query<Entity, With<EditorRoot>>) -> bool {
    query.is_empty()
}

/// Reports whether the editor UI is present so cleanup and transition logic can be gated safely.
pub(crate) fn editor_ui_present(query: Query<Entity, With<EditorRoot>>) -> bool {
    !query.is_empty()
}

/// Spawns the full editor UI shell so ship authoring, inspection, and tooling have a persistent workspace.
pub(crate) use spawn::spawn_editor_ui;
