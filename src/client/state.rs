use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use super::DEFAULT_HOST_ADDR;
use crate::{
    protocol::ShipSnapshot,
    ship::{ModuleKind, ShipDefinition},
};
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub(crate) enum ClientAppState {
    #[default]
    Menu,
    Editing,
    Playing,
}

#[derive(Resource)]
pub(crate) struct ConnectionConfig {
    pub(crate) server_addr: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            server_addr: DEFAULT_HOST_ADDR.to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ConnectionStatus {
    pub(crate) phase: ConnectionPhase,
    pub(crate) active_snapshot: Option<ShipSnapshot>,
}

#[derive(Default)]
pub(crate) enum ConnectionPhase {
    #[default]
    Idle,
    Connecting,
    Connected,
    Failed(String),
}

#[derive(Resource, Clone, Default)]
pub(crate) struct ConnectionMailbox {
    pub(crate) events: Arc<Mutex<Vec<ConnectionEvent>>>,
}

pub(crate) enum ConnectionEvent {
    Connected(ShipSnapshot),
    Failed(String),
}

#[derive(Resource, Default, Clone)]
pub(crate) struct EditorShip {
    pub(crate) ship: ShipDefinition,
}

#[derive(Resource)]
pub(crate) struct DemoProgression {
    pub(crate) scrap: u32,
}

impl Default for DemoProgression {
    fn default() -> Self {
        Self { scrap: 100 }
    }
}

#[derive(Resource, Default)]
pub(crate) struct DebugOverlayState {
    pub(crate) enabled: bool,
}

#[derive(Resource, Default, Clone)]
pub(crate) struct LastMissionReport {
    pub(crate) headline: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) scrap_awarded: u32,
    pub(crate) total_scrap: u32,
    pub(crate) hottest_module: Option<String>,
    pub(crate) first_disabled_module: Option<String>,
    pub(crate) repairs_performed: u32,
    pub(crate) stabilizations_performed: u32,
    pub(crate) automation_used: bool,
    pub(crate) automation_triggers: u32,
    pub(crate) redesign_hints: Vec<String>,
    pub(crate) recovered_raw_salvage: u32,
    pub(crate) processed_repair_charge: u32,
    pub(crate) consumed_repair_charge: u32,
    pub(crate) transfer_count: u32,
    pub(crate) processor_cycles: u32,
    pub(crate) logistics_bottleneck: Option<String>,
    pub(crate) logistics_automation_used: bool,
    pub(crate) arch_primary_program: Option<String>,
    pub(crate) arch_invalid_executions: u32,
    pub(crate) arch_recent_writes: Vec<String>,
}

#[derive(Resource)]
pub(crate) struct EditorToolState {
    pub(crate) selected_kind: ModuleKind,
    pub(crate) selected_rotation: u8,
}

impl Default for EditorToolState {
    fn default() -> Self {
        Self {
            selected_kind: ModuleKind::Hull,
            selected_rotation: 0,
        }
    }
}

#[derive(Component)]
pub(crate) struct MenuRoot;

#[derive(Component)]
pub(crate) struct EditorRoot;

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct JoinButton;

#[derive(Component)]
pub(crate) struct LaunchButton;

#[derive(Component)]
pub(crate) struct ReturnButton;

#[derive(Component)]
pub(crate) struct StatusText;

#[derive(Component)]
pub(crate) struct HostAddressText;

#[derive(Component)]
pub(crate) struct EditorStatusText;

#[derive(Component)]
pub(crate) struct GameplayStatusText;

#[derive(Component)]
pub(crate) struct GameplayInspectionText;

#[derive(Component)]
pub(crate) struct GameplayAlertsText;

#[derive(Component)]
pub(crate) struct GameplayControlsText;

#[derive(Component)]
pub(crate) struct ToolboxButton {
    pub(crate) kind: ModuleKind,
}

#[derive(Clone, Copy)]
pub(crate) enum ProgramButtonAction {
    CycleTemplate,
    AdjustConstant { index: usize, delta: i32 },
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

#[derive(Component)]
pub(crate) struct ShipTileSprite;

#[derive(Component)]
pub(crate) struct PreviewTile;

#[derive(Component)]
pub(crate) struct EditingCleanup;

#[derive(Component)]
pub(crate) struct PlayingCleanup;
