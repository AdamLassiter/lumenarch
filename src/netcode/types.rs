use std::{collections::BTreeMap, net::SocketAddr};

use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_HOST_ADDR,
    ship::ShipDefinition,
    state::{LastMissionReport, LocalPlayerProfile, Progression, SectorState},
};

pub(crate) type LumenGgrsConfig = bevy_ggrs::GgrsConfig<PlayerGgrsInput, SocketAddr, u128>;

pub(crate) const INPUT_UP: u16 = 1 << 0;
pub(crate) const INPUT_DOWN: u16 = 1 << 1;
pub(crate) const INPUT_LEFT: u16 = 1 << 2;
pub(crate) const INPUT_RIGHT: u16 = 1 << 3;
pub(crate) const INPUT_INTERACT: u16 = 1 << 4;
pub(crate) const INPUT_DROP: u16 = 1 << 5;
pub(crate) const INPUT_PICKUP: u16 = 1 << 6;
pub(crate) const INPUT_FIRE: u16 = 1 << 7;
pub(crate) const INPUT_TOGGLE_STATION: u16 = 1 << 8;
pub(crate) const INPUT_EXIT_STATION: u16 = 1 << 9;
pub(crate) const INPUT_SPACE_EDGE: u16 = 1 << 10;
pub(crate) const INPUT_TOGGLE_DEBUG: u16 = 1 << 11;
pub(crate) const INPUT_CYCLE_TEMPLATE: u16 = 1 << 12;
pub(crate) const INPUT_PREV_EDGE: u16 = 1 << 13;
pub(crate) const INPUT_NEXT_EDGE: u16 = 1 << 14;
pub(crate) const INPUT_AUX_EDGE: u16 = 1 << 15;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RollbackMetaOp {
    #[default]
    None = 0,
    OpenEditor = 1,
    OpenSectorMap = 2,
    RepairShip = 3,
    SelectSectorNode = 4,
    LaunchEncounter = 5,
    ReturnToDock = 6,
    LeaveEditor = 7,
    AcceptContract = 8,
    LaunchContract = 9,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StationControlOp {
    #[default]
    None = 0,
    HelmThrottle = 1,
    HelmTurn = 2,
    TurretAdjustAim = 3,
    TurretFireToggle = 4,
    ReactorAdjustRate = 5,
    ReactorAdjustTurbine = 6,
    LogisticsToggleStorageIntake = 7,
    LogisticsToggleAirlock = 8,
    LogisticsToggleManipulator = 9,
    LogisticsCycleManipulatorTarget = 10,
    LogisticsCycleResource = 11,
    LogisticsToggleProcessor = 12,
    ComputerToggleEnabled = 13,
    ComputerCycleTemplate = 14,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlayerGgrsInput {
    pub(crate) buttons: u16,
    pub(crate) throttle_milli: i16,
    pub(crate) turn_milli: i16,
    pub(crate) aim_x_milli: i16,
    pub(crate) aim_y_milli: i16,
    pub(crate) reactor_delta_milli: i16,
    pub(crate) turbine_delta_milli: i16,
    pub(crate) logistics_delta: i8,
    pub(crate) station_op: u8,
    pub(crate) station_arg0: i16,
    pub(crate) meta_op: u8,
    pub(crate) meta_arg0: i16,
    pub(crate) meta_arg1: i16,
    pub(crate) meta_arg2: i16,
}

impl PlayerGgrsInput {
    pub(crate) fn pressed(self, flag: u16) -> bool {
        self.buttons & flag != 0
    }

    pub(crate) fn meta_op(self) -> RollbackMetaOp {
        match self.meta_op {
            1 => RollbackMetaOp::OpenEditor,
            2 => RollbackMetaOp::OpenSectorMap,
            3 => RollbackMetaOp::RepairShip,
            4 => RollbackMetaOp::SelectSectorNode,
            5 => RollbackMetaOp::LaunchEncounter,
            6 => RollbackMetaOp::ReturnToDock,
            7 => RollbackMetaOp::LeaveEditor,
            8 => RollbackMetaOp::AcceptContract,
            9 => RollbackMetaOp::LaunchContract,
            _ => RollbackMetaOp::None,
        }
    }

    pub(crate) fn station_op(self) -> StationControlOp {
        match self.station_op {
            1 => StationControlOp::HelmThrottle,
            2 => StationControlOp::HelmTurn,
            3 => StationControlOp::TurretAdjustAim,
            4 => StationControlOp::TurretFireToggle,
            5 => StationControlOp::ReactorAdjustRate,
            6 => StationControlOp::ReactorAdjustTurbine,
            7 => StationControlOp::LogisticsToggleStorageIntake,
            8 => StationControlOp::LogisticsToggleAirlock,
            9 => StationControlOp::LogisticsToggleManipulator,
            10 => StationControlOp::LogisticsCycleManipulatorTarget,
            11 => StationControlOp::LogisticsCycleResource,
            12 => StationControlOp::LogisticsToggleProcessor,
            13 => StationControlOp::ComputerToggleEnabled,
            14 => StationControlOp::ComputerCycleTemplate,
            _ => StationControlOp::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RollbackPhase {
    #[default]
    Docked,
    SectorMap,
    Editing,
    Encounter,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RollbackGameState {
    pub(crate) phase: RollbackPhase,
    pub(crate) frame: i32,
    pub(crate) seed: u64,
    pub(crate) scene_generation: u32,
    pub(crate) editor_ship: ShipDefinition,
    pub(crate) progression: Progression,
    pub(crate) sector: SectorState,
    pub(crate) last_mission_report: LastMissionReport,
}

impl Default for RollbackGameState {
    fn default() -> Self {
        Self {
            phase: RollbackPhase::Docked,
            frame: 0,
            seed: 0x10_4E6,
            scene_generation: 0,
            editor_ship: ShipDefinition::empty("Untitled Knot"),
            progression: Progression::default(),
            sector: SectorState::default(),
            last_mission_report: LastMissionReport::default(),
        }
    }
}

#[derive(Resource, Clone)]
pub(crate) struct SessionConfig {
    pub(crate) session_descriptor: String,
    pub(crate) input_delay: usize,
    pub(crate) check_distance: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_descriptor: format!("host@{DEFAULT_HOST_ADDR}"),
            input_delay: 2,
            check_distance: 8,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) enum SessionPhase {
    #[default]
    Idle,
    Connecting,
    Lobby,
    Starting,
    Connected,
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionRole {
    Host,
    Client,
}

#[derive(Resource, Clone, Default)]
pub(crate) struct SessionStatus {
    pub(crate) phase: SessionPhase,
    pub(crate) role: Option<SessionRole>,
    pub(crate) total_players: usize,
    pub(crate) local_player_handles: Vec<PlayerHandle>,
    pub(crate) active_ship_snapshot: Option<ShipDefinition>,
    pub(crate) last_checksum: Option<u128>,
    pub(crate) lobby_snapshot: Option<LobbySnapshot>,
}

#[derive(Resource, Clone)]
pub(crate) struct SessionBootstrapConfig {
    pub(crate) pending_start: bool,
    pub(crate) role: SessionRole,
    pub(crate) local_bind_addr: SocketAddr,
    pub(crate) peer_addrs: Vec<SocketAddr>,
    pub(crate) local_handle: PlayerHandle,
    pub(crate) input_delay: usize,
    pub(crate) check_distance: usize,
    pub(crate) initial_state: RollbackGameState,
}

impl Default for SessionBootstrapConfig {
    fn default() -> Self {
        Self {
            pending_start: false,
            role: SessionRole::Host,
            local_bind_addr: DEFAULT_HOST_ADDR.parse().unwrap(),
            peer_addrs: Vec::new(),
            local_handle: 0,
            input_delay: 2,
            check_distance: 8,
            initial_state: RollbackGameState::default(),
        }
    }
}

#[derive(Resource, Default, Clone)]
pub(crate) struct LocalPlayerHandle(pub(crate) Option<PlayerHandle>);

#[derive(Resource, Default, Clone)]
pub(crate) struct PlayerHandleMap {
    pub(crate) entities: BTreeMap<PlayerHandle, Entity>,
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct ObservedLocalPlayer {
    pub(crate) handle: Option<PlayerHandle>,
    pub(crate) entity: Option<Entity>,
}

#[derive(Resource, Default, Clone)]
pub(crate) struct ChecksumHistory {
    pub(crate) last_frame: i32,
    pub(crate) last_value: u128,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivePresentationPhase {
    pub(crate) phase: Option<RollbackPhase>,
    pub(crate) scene_generation: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LobbyPlayerInfo {
    pub(crate) handle: PlayerHandle,
    pub(crate) bind_addr: SocketAddr,
    pub(crate) is_host: bool,
    pub(crate) profile: LocalPlayerProfile,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LobbySnapshot {
    pub(crate) host_addr: SocketAddr,
    pub(crate) players: Vec<LobbyPlayerInfo>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingMetaCommand {
    pub(crate) op: RollbackMetaOp,
    pub(crate) arg0: i16,
    pub(crate) arg1: i16,
    pub(crate) arg2: i16,
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct PendingLocalMetaCommand(pub(crate) Option<PendingMetaCommand>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingStationCommand {
    pub(crate) op: StationControlOp,
    pub(crate) arg0: i16,
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct PendingLocalStationCommand(pub(crate) Option<PendingStationCommand>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DecodedPlayerCommand {
    pub(crate) raw: PlayerGgrsInput,
    pub(crate) move_x: i8,
    pub(crate) move_y: i8,
    pub(crate) throttle_milli: i16,
    pub(crate) turn_milli: i16,
    pub(crate) aim_x_milli: i16,
    pub(crate) aim_y_milli: i16,
    pub(crate) reactor_delta_milli: i16,
    pub(crate) turbine_delta_milli: i16,
    pub(crate) logistics_delta: i8,
    pub(crate) station: PendingStationCommand,
    pub(crate) meta: PendingMetaCommand,
}

#[derive(Resource, Default, Clone)]
pub(crate) struct DecodedPlayerCommands {
    pub(crate) by_handle: BTreeMap<PlayerHandle, DecodedPlayerCommand>,
}

impl MapEntities for PlayerHandleMap {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in self.entities.values_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

pub(crate) fn is_host_authority(status: &SessionStatus) -> bool {
    status.role.is_none_or(|role| role == SessionRole::Host)
}
