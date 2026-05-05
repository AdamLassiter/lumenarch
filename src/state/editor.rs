use std::collections::HashMap;

use bevy::prelude::*;

use crate::ship::{
    ModuleKind,
    ModuleVariant,
    ShipDefinition,
    enemy::{EnemyShipEntryValidationStatus, EnemyShipLibrary},
};

#[derive(Resource, Default, Clone)]
pub(crate) struct EditorShip {
    pub(crate) ship: ShipDefinition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EditorMode {
    #[default]
    Player,
    Enemy,
}

#[derive(Resource, Default)]
pub(crate) struct EditorSessionState {
    pub(crate) mode: EditorMode,
}

#[derive(Resource, Clone)]
pub(crate) struct EnemyShipLibraryState {
    pub(crate) library: EnemyShipLibrary,
    pub(crate) entry_statuses: HashMap<String, EnemyShipEntryValidationStatus>,
    pub(crate) selected_index: usize,
}

impl Default for EnemyShipLibraryState {
    fn default() -> Self {
        Self {
            library: EnemyShipLibrary::seeded(),
            entry_statuses: HashMap::default(),
            selected_index: 0,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub(crate) struct EnemyEditorState {
    pub(crate) dirty: bool,
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct EditorViewState {
    pub(crate) center: Vec2,
    pub(crate) zoom: f32,
}

impl Default for EditorViewState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct EditorPanState {
    pub(crate) last_cursor: Option<Vec2>,
}

#[derive(Resource, Clone)]
pub(crate) struct EditorUiState {
    pub(crate) mission_report_expanded: bool,
    pub(crate) toolbox_scroll: f32,
    pub(crate) toolbox_tooltip: EditorToolboxTooltip,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            mission_report_expanded: false,
            toolbox_scroll: 0.0,
            toolbox_tooltip: EditorToolboxTooltip::default(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct EditorToolboxTooltip {
    pub(crate) title: String,
    pub(crate) detail: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EditorToolMode {
    #[default]
    Build,
    Select,
}

#[derive(Resource)]
pub(crate) struct EditorToolState {
    pub(crate) tool_mode: EditorToolMode,
    pub(crate) selected_kind: ModuleKind,
    pub(crate) selected_variant: ModuleVariant,
    pub(crate) selected_rotation: u8,
    pub(crate) selected_channel: u8,
}

impl Default for EditorToolState {
    fn default() -> Self {
        Self {
            tool_mode: EditorToolMode::Build,
            selected_kind: ModuleKind::Hull,
            selected_variant: ModuleVariant::default_for_kind(ModuleKind::Hull),
            selected_rotation: 0,
            selected_channel: 0,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ArchEditorState {
    pub(crate) selected_module_id: Option<u64>,
    pub(crate) selected_line: usize,
    pub(crate) selected_language: ProgrammingLanguageMode,
    pub(crate) panel_open: bool,
}

#[derive(Resource, Clone, Default)]
pub(crate) struct ProgramTextEditorState {
    pub(crate) module_id: Option<u64>,
    pub(crate) language: ProgrammingLanguageMode,
    pub(crate) draft_text: String,
    pub(crate) cursor_index: usize,
    pub(crate) select_all: bool,
    pub(crate) focused: bool,
    pub(crate) dirty: bool,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) status_line: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ProgrammingLanguageMode {
    #[default]
    Arch,
    Lumen,
}

#[derive(Component)]
pub(crate) struct EditorRoot;

#[derive(Component)]
pub(crate) struct LeaveEditorButton;

#[derive(Component)]
pub(crate) struct EnemyPrevButton;

#[derive(Component)]
pub(crate) struct EnemyNextButton;

#[derive(Component)]
pub(crate) struct EnemyNewButton;

#[derive(Component)]
pub(crate) struct EditorStatusText;

#[derive(Component)]
pub(crate) struct EditorMissionReportButton;

#[derive(Component)]
pub(crate) struct EditorMissionReportButtonText;

#[derive(Component)]
pub(crate) struct EditorMissionReportText;

#[derive(Component)]
pub(crate) struct EditorToolboxScrollViewport;

#[derive(Component)]
pub(crate) struct EditorToolboxScrollContent;

#[derive(Component)]
pub(crate) struct ToolboxVariantButton {
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
}

#[derive(Component)]
pub(crate) struct ToolboxVariantButtonText {
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
}

#[derive(Component)]
pub(crate) struct EditorToolModeButton {
    pub(crate) mode: EditorToolMode,
}

#[derive(Component)]
pub(crate) struct EditorToolModeButtonText {
    pub(crate) mode: EditorToolMode,
}

#[derive(Component)]
pub(crate) struct EditorBuildSection;

#[derive(Component)]
pub(crate) struct EditorSelectSection;

#[derive(Component)]
pub(crate) struct EditorSelectionSummaryText;

#[derive(Component)]
pub(crate) struct EditorToolboxTooltipText;

#[derive(Component)]
pub(crate) struct EditorAutoHullButton;

#[derive(Component)]
pub(crate) struct EditorDeleteSelectionButton;

#[derive(Component)]
pub(crate) struct EditorCopySelectionButton;

#[derive(Component)]
pub(crate) struct EditorPasteSelectionButton;

#[derive(Resource, Default, Clone)]
pub(crate) struct EditorSelectionState {
    pub(crate) selected_module_ids: Vec<u64>,
    pub(crate) clipboard: Vec<ShipModuleSnapshot>,
    pub(crate) marquee_origin: Option<(i32, i32)>,
    pub(crate) marquee_current: Option<(i32, i32)>,
}

#[derive(Clone, Debug)]
pub(crate) struct ShipModuleSnapshot {
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) rotation_quadrants: u8,
    pub(crate) channel: u8,
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct EditorPointerState {
    pub(crate) last_build_cell: Option<(i32, i32, MouseButton)>,
}

#[derive(Clone, Copy)]
pub(crate) enum ProgramButtonAction {
    SwitchLanguage(ProgrammingLanguageMode),
    CycleArchTemplate,
    AdjustArchConstant { index: usize, delta: i32 },
    CycleLumenTemplate,
    ToggleLumenEnabled,
}

#[derive(Clone, Copy)]
pub(crate) enum ArchEditorButtonAction {
    SelectModule(u64),
    SelectLine {
        module_id: u64,
        line: usize,
    },
    AddLine(u64),
    InsertLineAfter {
        module_id: u64,
        line: usize,
    },
    RemoveLine {
        module_id: u64,
        line: usize,
    },
    MoveLineUp {
        module_id: u64,
        line: usize,
    },
    MoveLineDown {
        module_id: u64,
        line: usize,
    },
    CycleOpcode {
        module_id: u64,
        line: usize,
    },
    CycleDst {
        module_id: u64,
        line: usize,
    },
    CycleSrcA {
        module_id: u64,
        line: usize,
    },
    CycleSrcB {
        module_id: u64,
        line: usize,
    },
    AdjustImmediateA {
        module_id: u64,
        line: usize,
        delta: i32,
    },
    AdjustImmediateB {
        module_id: u64,
        line: usize,
        delta: i32,
    },
    AdjustJump {
        module_id: u64,
        line: usize,
        delta: i32,
    },
    RenameModuleProgram(u64),
    SelectLumenLine {
        module_id: u64,
        line: usize,
    },
    AddLumenLine(u64),
    InsertLumenLineAfter {
        module_id: u64,
        line: usize,
    },
    RemoveLumenLine {
        module_id: u64,
        line: usize,
    },
    MoveLumenLineUp {
        module_id: u64,
        line: usize,
    },
    MoveLumenLineDown {
        module_id: u64,
        line: usize,
    },
    CycleLumenOp {
        module_id: u64,
        line: usize,
    },
    CycleLumenTarget {
        module_id: u64,
        line: usize,
    },
    CycleLumenAspect {
        module_id: u64,
        line: usize,
    },
    AdjustLumenWeight {
        module_id: u64,
        line: usize,
        delta: i32,
    },
    RenameLumenProgram(u64),
}

#[derive(Component)]
pub(crate) struct ComputerProgramPanel;

#[derive(Component)]
pub(crate) struct ComputerProgramEntry;

#[derive(Component)]
pub(crate) struct ComputerProgramButton {
    pub(crate) module_id: u64,
    pub(crate) action: ProgramButtonAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProgramEditorAction {
    SwitchArch,
    SwitchLumen,
    Check,
    Apply,
    Revert,
    Focus,
}

#[derive(Component)]
pub(crate) struct ProgramEditorActionButton {
    pub(crate) action: ProgramEditorAction,
}

#[derive(Component)]
pub(crate) struct ProgramEditorTextBox;

#[derive(Component)]
pub(crate) struct ProgramEditorDraftText;

#[derive(Component)]
pub(crate) struct ProgramEditorStatusText;

#[derive(Component)]
pub(crate) struct ProgramEditorDiagnosticsText;

#[derive(Component)]
pub(crate) struct ArchEditorButton {
    pub(crate) action: ArchEditorButtonAction,
}

#[derive(Component)]
pub(crate) struct ShipTileSprite;

#[derive(Component)]
pub(crate) struct PreviewTile;

#[derive(Component)]
pub(crate) struct EditingCleanup;
