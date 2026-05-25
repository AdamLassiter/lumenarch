use bevy::{input::mouse::MouseButton, prelude::*, window::PrimaryWindow};

use super::{
    auto_hull::apply_auto_hull_to_ship,
    enemy::{save_enemy_library_if_valid, sync_selected_enemy_entry},
    selection::{
        delete_selected_group,
        foundation_snapshot,
        module_snapshot,
        move_selected_foundation_group,
        move_selected_group,
        paste_clipboard_group,
        paste_foundation_clipboard_group,
        selected_or_all_modules,
        ship_anchor,
    },
};
use crate::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    editor::{SELECTED_UNAFFORDABLE_BUTTON, UNAFFORDABLE_BUTTON},
    helpers::editor::{
        cursor_grid_position,
        foundation_family_label,
        foundation_supports_module,
        is_cursor_over_editor_ui,
        is_hull_foundation_kind,
        module_belongs_to_components_layer,
        module_belongs_to_hull_layer,
        module_family_label,
        variant_tooltip_text,
    },
    netcode,
    ship::{
        ModuleKind,
        ShipDefinition,
        ShipFoundationTile,
        ShipModule,
        arch::{ArchProgram, ArchProgramTemplate},
        lumen::{LumenProgram, LumenProgramTemplate},
    },
    state::{
        ArchEditorState,
        EditorAutoHullButton,
        EditorCopySelectionButton,
        EditorDeleteSelectionButton,
        EditorLayer,
        EditorLayerButton,
        EditorMissionReportButton,
        EditorMode,
        EditorPasteSelectionButton,
        EditorPlacementBlocker,
        EditorPointerState,
        EditorSelectionState,
        EditorSessionState,
        EditorShip,
        EditorToolMode,
        EditorToolModeButton,
        EditorToolState,
        EditorUiState,
        EnemyEditorState,
        EnemyShipLibraryState,
        FocusedTextBox,
        FrontendMode,
        GameplayStationPanelButton,
        LeaveEditorButton,
        ProgrammingLanguageMode,
        Progression,
        StationPanelButtonAction,
        StationToolboxButton,
        ToolboxFoundationButton,
        ToolboxVariantButton,
    },
    station_editor,
    stations::StationCatalogResource,
};

mod actions;
mod building;
mod selection;
#[cfg(test)]
mod tests;

pub(crate) use actions::{
    leave_editor_button_system,
    leave_editor_keyboard_shortcut,
    mission_report_button_system,
    toolbox_button_system,
};
use building::*;
pub(crate) use building::{place_or_remove_tile, rotate_selected_tool};
use selection::*;
pub(crate) use selection::{
    editor_station_panel_button_system,
    repair_selected_component_shortcut,
    selection_action_button_system,
    selection_shortcuts,
    toggle_editor_module_overlay_shortcuts,
};
