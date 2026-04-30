use serde::{Deserialize, Serialize};

pub use crate::ship::ShipDefinition as ShipSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    pub protocol_version: u32,
    pub client_role: String,
    pub player_name: String,
}

impl ClientHello {
    pub fn new(client_role: impl Into<String>, player_name: impl Into<String>) -> Self {
        Self {
            protocol_version: 2,
            client_role: client_role.into(),
            player_name: player_name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SessionAppState {
    #[default]
    Menu,
    Docked,
    SectorMap,
    Editing,
    Encounter,
}

impl From<crate::client::state::ClientAppState> for SessionAppState {
    fn from(value: crate::client::state::ClientAppState) -> Self {
        match value {
            crate::client::state::ClientAppState::Menu => Self::Menu,
            crate::client::state::ClientAppState::Docked => Self::Docked,
            crate::client::state::ClientAppState::SectorMap => Self::SectorMap,
            crate::client::state::ClientAppState::Editing => Self::Editing,
            crate::client::state::ClientAppState::Encounter => Self::Encounter,
        }
    }
}

impl From<SessionAppState> for crate::client::state::ClientAppState {
    fn from(value: SessionAppState) -> Self {
        match value {
            SessionAppState::Menu => Self::Menu,
            SessionAppState::Docked => Self::Docked,
            SessionAppState::SectorMap => Self::SectorMap,
            SessionAppState::Editing => Self::Editing,
            SessionAppState::Encounter => Self::Encounter,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PresenceFrame {
    #[default]
    World,
    Ship,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SessionControlMode {
    #[default]
    Interior,
    Cockpit,
    Turret,
    Reactor,
    Logistics,
    Computer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPresenceSnapshot {
    pub player_id: u32,
    pub frame: PresenceFrame,
    pub world_position: [f32; 2],
    pub world_velocity: [f32; 2],
    pub local_position: [f32; 2],
    pub local_velocity: [f32; 2],
    pub current_station_module_id: Option<u64>,
    pub carried_kind: Option<String>,
    pub carried_amount: u32,
    pub control_mode: SessionControlMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInputFrame {
    pub player_id: u32,
    pub tick: u64,
    pub throttle_milli: i32,
    pub turn_milli: i32,
    pub fire_pressed: bool,
    pub focus_module_id: Option<u64>,
    pub control_mode: SessionControlMode,
    pub carrying: bool,
    pub interaction_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionPeerInfo {
    pub player_id: u32,
    pub display_name: String,
    pub connected: bool,
    pub last_input_tick: u64,
    pub presence: PlayerPresenceSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub tick: u64,
    pub app_state: SessionAppState,
    pub ship: ShipSnapshot,
    pub progression: crate::client::state::DemoProgression,
    pub sector: crate::client::state::SectorState,
    pub last_mission_report: crate::client::state::LastMissionReport,
    pub peers: Vec<SessionPeerInfo>,
    pub state_hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWelcome {
    pub local_player_id: u32,
    pub snapshot: SessionSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCommand {
    SetAppState(SessionAppState),
    UpdateShip(ShipSnapshot),
    UpdateProgression(crate::client::state::DemoProgression),
    UpdateSector(crate::client::state::SectorState),
    UpdateMissionReport(crate::client::state::LastMissionReport),
    SelectSectorNode(Option<u32>),
    SetActiveEncounterNode(Option<u32>),
    ApplyTravelOutcome(crate::client::state::TravelOutcome),
    RepairShip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHashReport {
    pub player_id: u32,
    pub tick: u64,
    pub local_hash: u64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashStatusMessage {
    pub tick: u64,
    pub player_id: u32,
    pub host_hash: u64,
    pub local_hash: u64,
    pub matched: bool,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectedMessage {
    pub tick: u64,
    pub player_id: u32,
    pub host_hash: u64,
    pub local_hash: u64,
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommittedInputBatch {
    pub tick: u64,
    pub frames: Vec<PlayerInputFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello(ClientHello),
    Command(SessionCommand),
    InputFrame(PlayerInputFrame),
    Presence(PlayerPresenceSnapshot),
    HashReport(StateHashReport),
    RequestResync { category: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    SessionWelcome(SessionWelcome),
    SessionSnapshot(SessionSnapshot),
    CommittedInputs(CommittedInputBatch),
    HashStatus(HashStatusMessage),
    DriftDetected(DriftDetectedMessage),
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_snapshot_round_trip_json() {
        let snapshot = SessionSnapshot {
            tick: 7,
            app_state: SessionAppState::Docked,
            ship: ShipSnapshot::empty("Test"),
            progression: crate::client::state::DemoProgression::default(),
            sector: crate::client::state::SectorState::default(),
            last_mission_report: crate::client::state::LastMissionReport::default(),
            peers: vec![SessionPeerInfo {
                player_id: 1,
                display_name: "Host".to_string(),
                connected: true,
                last_input_tick: 7,
                presence: PlayerPresenceSnapshot::default(),
            }],
            state_hash: 42,
        };

        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: SessionSnapshot = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.tick, 7);
        assert_eq!(decoded.state_hash, 42);
        assert_eq!(decoded.peers.len(), 1);
    }
}
