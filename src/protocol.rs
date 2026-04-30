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

impl From<crate::state::ClientAppState> for SessionAppState {
    fn from(value: crate::state::ClientAppState) -> Self {
        match value {
            crate::state::ClientAppState::Menu => Self::Menu,
            crate::state::ClientAppState::Docked => Self::Docked,
            crate::state::ClientAppState::SectorMap => Self::SectorMap,
            crate::state::ClientAppState::Editing => Self::Editing,
            crate::state::ClientAppState::Encounter => Self::Encounter,
        }
    }
}

impl From<SessionAppState> for crate::state::ClientAppState {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegisterValue {
    Int(i64),
    Bool(bool),
    Symbol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterStateEntry {
    pub key: String,
    pub value: RegisterValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInputFrame {
    pub player_id: u32,
    pub tick: u64,
    #[serde(default)]
    pub writes: Vec<RegisterStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EncounterRegisterState {
    pub entries: Vec<RegisterStateEntry>,
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
    pub progression: crate::state::DemoProgression,
    pub sector: crate::state::SectorState,
    pub last_mission_report: crate::state::LastMissionReport,
    pub encounter_registers: EncounterRegisterState,
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
    UpdateProgression(crate::state::DemoProgression),
    UpdateSector(crate::state::SectorState),
    UpdateMissionReport(crate::state::LastMissionReport),
    SelectSectorNode(Option<u32>),
    SetActiveEncounterNode(Option<u32>),
    ApplyTravelOutcome(crate::state::TravelOutcome),
    RepairShip,
    UpdateEncounterRegisters(EncounterRegisterState),
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
            progression: crate::state::DemoProgression::default(),
            sector: crate::state::SectorState::default(),
            last_mission_report: crate::state::LastMissionReport::default(),
            encounter_registers: EncounterRegisterState::default(),
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
