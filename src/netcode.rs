use std::{collections::BTreeMap, net::SocketAddr};

use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use bevy_ggrs::{
    LocalInputs,
    LocalPlayers,
    PlayerInputs,
    RollbackFrameCount,
    Session,
    prelude::{PlayerType, SessionBuilder},
};
use ggrs::{PlayerHandle, UdpNonBlockingSocket};
use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_HOST_ADDR,
    campaign::{CampaignSave, load_campaign},
    ship::{ShipDefinition, storage::load_default_ship},
    state::{ClientAppState, DemoProgression, LastMissionReport, SectorState},
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
            _ => RollbackMetaOp::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RollbackPhase {
    #[default]
    Menu,
    Docked,
    SectorMap,
    Editing,
    Encounter,
}

impl From<ClientAppState> for RollbackPhase {
    fn from(value: ClientAppState) -> Self {
        match value {
            ClientAppState::Menu => Self::Menu,
            ClientAppState::Docked => Self::Docked,
            ClientAppState::SectorMap => Self::SectorMap,
            ClientAppState::Editing => Self::Editing,
            ClientAppState::Encounter => Self::Encounter,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RollbackGameState {
    pub(crate) phase: RollbackPhase,
    pub(crate) frame: i32,
    pub(crate) seed: u64,
    pub(crate) scene_generation: u32,
    pub(crate) editor_ship: ShipDefinition,
    pub(crate) progression: DemoProgression,
    pub(crate) sector: SectorState,
    pub(crate) last_mission_report: LastMissionReport,
}

impl Default for RollbackGameState {
    fn default() -> Self {
        Self {
            phase: RollbackPhase::Menu,
            frame: 0,
            seed: 0x10_4E6,
            scene_generation: 0,
            editor_ship: ShipDefinition::empty("Untitled Knot"),
            progression: DemoProgression::default(),
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
    pub(crate) meta: PendingMetaCommand,
}

#[derive(Resource, Default, Clone)]
pub(crate) struct DecodedPlayerCommands {
    pub(crate) by_handle: BTreeMap<PlayerHandle, DecodedPlayerCommand>,
}

pub(crate) fn begin_session_attempt(
    config: &SessionConfig,
    status: &mut SessionStatus,
    bootstrap: &mut SessionBootstrapConfig,
) {
    match parse_session_descriptor(&config.session_descriptor) {
        Ok(descriptor) => {
            let total_players = descriptor.peer_addrs.len() + 1;
            status.phase = SessionPhase::Connecting;
            status.role = Some(descriptor.role);
            bootstrap.pending_start = true;
            bootstrap.role = descriptor.role;
            bootstrap.local_bind_addr = descriptor.local_bind_addr;
            bootstrap.peer_addrs = descriptor.peer_addrs;
            bootstrap.local_handle = descriptor.local_handle;
            bootstrap.input_delay = config.input_delay;
            bootstrap.check_distance = config.check_distance;
            bootstrap.initial_state = load_initial_rollback_state();
            status.total_players = total_players;
            status.active_ship_snapshot = Some(bootstrap.initial_state.editor_ship.clone());
        }
        Err(error) => {
            status.phase = SessionPhase::Failed(error);
            bootstrap.pending_start = false;
        }
    }
}

pub(crate) fn finalize_pending_session_bootstrap(
    mut commands: Commands,
    mut bootstrap: ResMut<SessionBootstrapConfig>,
    mut status: ResMut<SessionStatus>,
    mut rollback_state: ResMut<RollbackGameState>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    if !bootstrap.pending_start {
        return;
    }

    match build_p2p_session(
        bootstrap.local_bind_addr,
        bootstrap.local_handle,
        &bootstrap.peer_addrs,
        bootstrap.input_delay,
        bootstrap.check_distance,
    ) {
        Ok(session) => {
            commands.insert_resource(Session::P2P(session));
            status.phase = SessionPhase::Connected;
            status.total_players = bootstrap.peer_addrs.len() + 1;
            status.local_player_handles = vec![bootstrap.local_handle];
            status.active_ship_snapshot = Some(bootstrap.initial_state.editor_ship.clone());
            *rollback_state = bootstrap.initial_state.clone();
            bootstrap.pending_start = false;
            next_state.set(ClientAppState::Docked);
        }
        Err(error) => {
            status.phase = SessionPhase::Failed(error);
            bootstrap.pending_start = false;
        }
    }
}

pub(crate) fn advance_rollback_state(
    frame: Res<RollbackFrameCount>,
    mut rollback_state: ResMut<RollbackGameState>,
    mut checksum_history: ResMut<ChecksumHistory>,
) {
    rollback_state.frame = frame.0;
    checksum_history.last_frame = frame.0;
    checksum_history.last_value = stable_hash(rollback_state.as_ref());
}

pub(crate) fn sync_presentation_from_rollback(
    rollback_state: Res<RollbackGameState>,
    mut editor_ship: ResMut<crate::state::EditorShip>,
    mut progression: ResMut<DemoProgression>,
    mut sector: ResMut<SectorState>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    if !rollback_state.is_changed() {
        return;
    }
    editor_ship.ship = rollback_state.editor_ship.clone();
    *progression = rollback_state.progression.clone();
    *sector = rollback_state.sector.clone();
    *last_mission_report = rollback_state.last_mission_report.clone();
    next_state.set(match rollback_state.phase {
        RollbackPhase::Menu => ClientAppState::Menu,
        RollbackPhase::Docked => ClientAppState::Docked,
        RollbackPhase::SectorMap => ClientAppState::SectorMap,
        RollbackPhase::Editing => ClientAppState::Editing,
        RollbackPhase::Encounter => ClientAppState::Encounter,
    });
}

pub(crate) fn read_local_inputs(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
    local_players: Res<LocalPlayers>,
) {
    let mut inputs = bevy::utils::HashMap::default();
    let input = input_from_hardware(
        &keyboard_input,
        &mouse_buttons,
        window_query.iter().next(),
        pending_meta.0.take(),
    );
    for handle in &local_players.0 {
        inputs.insert(*handle, input);
    }
    commands.insert_resource(LocalInputs::<LumenGgrsConfig>(inputs));
}

pub(crate) fn decode_player_inputs(
    player_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    mut decoded: ResMut<DecodedPlayerCommands>,
) {
    decoded.by_handle.clear();
    let Some(player_inputs) = player_inputs else {
        return;
    };
    for (handle, (input, _)) in player_inputs.iter().enumerate() {
        decoded
            .by_handle
            .insert(handle, decode_player_command(*input));
    }
}

pub(crate) fn local_player_input(
    players: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    local_handle: Option<Res<LocalPlayerHandle>>,
) -> PlayerGgrsInput {
    let Some(players) = players else {
        return PlayerGgrsInput::default();
    };
    let Some(local_handle) = local_handle else {
        return PlayerGgrsInput::default();
    };
    let Some(handle) = local_handle.0 else {
        return PlayerGgrsInput::default();
    };
    players
        .get(handle)
        .map(|(input, _)| *input)
        .unwrap_or_default()
}

pub(crate) fn sync_local_player_handle(
    local_players: Res<LocalPlayers>,
    mut local_handle: ResMut<LocalPlayerHandle>,
) {
    local_handle.0 = local_players.0.first().copied();
}

pub(crate) fn is_host_authority(status: &SessionStatus) -> bool {
    status.role.is_none_or(|role| role == SessionRole::Host)
}

impl MapEntities for PlayerHandleMap {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in self.entities.values_mut() {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

pub(crate) fn rollback_phase_is_encounter(rollback_state: Res<RollbackGameState>) -> bool {
    rollback_state.phase == RollbackPhase::Encounter
}

pub(crate) fn apply_host_meta_ops(
    status: Res<SessionStatus>,
    decoded: Res<DecodedPlayerCommands>,
    mut rollback_state: ResMut<RollbackGameState>,
) {
    if !is_host_authority(&status) {
        return;
    }
    let Some(command) = decoded.by_handle.get(&0) else {
        return;
    };

    match command.meta.op {
        RollbackMetaOp::None => {}
        RollbackMetaOp::OpenEditor => {
            rollback_state.phase = RollbackPhase::Editing;
        }
        RollbackMetaOp::OpenSectorMap => {
            rollback_state.phase = RollbackPhase::SectorMap;
        }
        RollbackMetaOp::RepairShip => {
            let repair_cost = rollback_state.progression.hull_wear.saturating_mul(2);
            if repair_cost > 0 && rollback_state.progression.scrap >= repair_cost {
                rollback_state.progression.scrap -= repair_cost;
                rollback_state.progression.hull_wear = 0;
                rollback_state.last_mission_report.detail = Some(format!(
                    "Station service restored your ship for {repair_cost} scrap."
                ));
            }
        }
        RollbackMetaOp::SelectSectorNode => {
            let node_id = command.meta.arg0.max(0) as u32;
            if rollback_state.sector.is_reachable(node_id)
                && rollback_state
                    .sector
                    .node(node_id)
                    .map(|node| !matches!(node.kind, crate::state::SectorNodeKind::HubStation))
                    .unwrap_or(false)
            {
                rollback_state.sector.selected_node_id = Some(node_id);
            }
        }
        RollbackMetaOp::LaunchEncounter => {
            let node_id = command.meta.arg0.max(0) as u32;
            if rollback_state.sector.is_reachable(node_id)
                && rollback_state
                    .sector
                    .node(node_id)
                    .map(|node| !matches!(node.kind, crate::state::SectorNodeKind::HubStation))
                    .unwrap_or(false)
            {
                rollback_state.sector.active_encounter_node_id = Some(node_id);
                rollback_state.progression.jump_count += 1;
                rollback_state.phase = RollbackPhase::Encounter;
                rollback_state.scene_generation += 1;
            }
        }
        RollbackMetaOp::ReturnToDock => {
            rollback_state.phase = RollbackPhase::Docked;
        }
        RollbackMetaOp::LeaveEditor => {
            rollback_state.phase = RollbackPhase::Docked;
        }
    }
}

fn build_p2p_session(
    local_bind_addr: SocketAddr,
    local_handle: PlayerHandle,
    peer_addrs: &[SocketAddr],
    input_delay: usize,
    check_distance: usize,
) -> Result<ggrs::P2PSession<LumenGgrsConfig>, String> {
    let player_count = peer_addrs.len() + 1;
    let mut builder = SessionBuilder::<LumenGgrsConfig>::new()
        .with_num_players(player_count)
        .with_input_delay(input_delay)
        .with_check_distance(check_distance);

    for handle in 0..player_count {
        let player = if handle == local_handle {
            PlayerType::Local
        } else {
            let peer_index = if handle < local_handle {
                handle
            } else {
                handle - 1
            };
            PlayerType::Remote(
                *peer_addrs
                    .get(peer_index)
                    .ok_or_else(|| format!("missing remote peer address for handle {handle}"))?,
            )
        };
        builder = builder
            .add_player(player, handle)
            .map_err(|error| format!("failed to register player {handle}: {error}"))?;
    }

    let socket = UdpNonBlockingSocket::bind_to_port(local_bind_addr.port())
        .map_err(|error| format!("failed to bind UDP socket on {}: {error}", local_bind_addr))?;
    builder
        .start_p2p_session(socket)
        .map_err(|error| format!("failed to start GGRS P2P session: {error}"))
}

fn input_from_hardware(
    keys: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
    window: Option<&Window>,
    pending_meta: Option<PendingMetaCommand>,
) -> PlayerGgrsInput {
    let mut input = PlayerGgrsInput::default();

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        input.buttons |= INPUT_UP;
        input.throttle_milli = 1000;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        input.buttons |= INPUT_DOWN;
        input.throttle_milli = -1000;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        input.buttons |= INPUT_LEFT;
        input.turn_milli = 1000;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        input.buttons |= INPUT_RIGHT;
        input.turn_milli = -1000;
    }
    if keys.pressed(KeyCode::KeyE) {
        input.buttons |= INPUT_INTERACT;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        input.buttons |= INPUT_TOGGLE_STATION;
    }
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape) {
        input.buttons |= INPUT_EXIT_STATION;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        input.buttons |= INPUT_PICKUP;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        input.buttons |= INPUT_DROP;
    }
    if keys.pressed(KeyCode::Space) {
        input.buttons |= INPUT_FIRE;
    }
    if keys.just_pressed(KeyCode::Space) {
        input.buttons |= INPUT_SPACE_EDGE;
    }
    if keys.just_pressed(KeyCode::F3) {
        input.buttons |= INPUT_TOGGLE_DEBUG;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        input.buttons |= INPUT_CYCLE_TEMPLATE;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        input.buttons |= INPUT_PREV_EDGE;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        input.buttons |= INPUT_NEXT_EDGE;
    }
    if keys.just_pressed(KeyCode::KeyM) || keys.just_pressed(KeyCode::Space) {
        input.buttons |= INPUT_AUX_EDGE;
    }

    if keys.pressed(KeyCode::KeyR) {
        input.reactor_delta_milli = 1000;
    }
    if keys.pressed(KeyCode::KeyF) {
        input.turbine_delta_milli = 1000;
    }
    if keys.pressed(KeyCode::BracketLeft) {
        input.logistics_delta = -1;
    } else if keys.pressed(KeyCode::BracketRight) {
        input.logistics_delta = 1;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        input.buttons |= INPUT_FIRE;
    }
    if let Some(cursor) = window.and_then(Window::cursor_position) {
        let x = ((cursor.x / window.unwrap().width()) * 2.0 - 1.0).clamp(-1.0, 1.0);
        let y = ((cursor.y / window.unwrap().height()) * 2.0 - 1.0).clamp(-1.0, 1.0);
        input.aim_x_milli = (x * 1000.0) as i16;
        input.aim_y_milli = (-y * 1000.0) as i16;
    }

    if let Some(meta) = pending_meta {
        input.meta_op = meta.op as u8;
        input.meta_arg0 = meta.arg0;
        input.meta_arg1 = meta.arg1;
        input.meta_arg2 = meta.arg2;
    }

    input
}

fn decode_player_command(input: PlayerGgrsInput) -> DecodedPlayerCommand {
    DecodedPlayerCommand {
        raw: input,
        move_x: i8::from(input.pressed(INPUT_RIGHT)) - i8::from(input.pressed(INPUT_LEFT)),
        move_y: i8::from(input.pressed(INPUT_UP)) - i8::from(input.pressed(INPUT_DOWN)),
        throttle_milli: input.throttle_milli,
        turn_milli: input.turn_milli,
        aim_x_milli: input.aim_x_milli,
        aim_y_milli: input.aim_y_milli,
        reactor_delta_milli: input.reactor_delta_milli,
        turbine_delta_milli: input.turbine_delta_milli,
        logistics_delta: input.logistics_delta,
        meta: PendingMetaCommand {
            op: input.meta_op(),
            arg0: input.meta_arg0,
            arg1: input.meta_arg1,
            arg2: input.meta_arg2,
        },
    }
}

pub(crate) fn command_for_handle(
    decoded: &DecodedPlayerCommands,
    handle: PlayerHandle,
) -> DecodedPlayerCommand {
    decoded.by_handle.get(&handle).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_button_flags_round_trip() {
        let input = PlayerGgrsInput {
            buttons: INPUT_UP | INPUT_FIRE | INPUT_SPACE_EDGE,
            throttle_milli: 1000,
            turn_milli: -1000,
            ..Default::default()
        };

        assert!(input.pressed(INPUT_UP));
        assert!(input.pressed(INPUT_FIRE));
        assert!(input.pressed(INPUT_SPACE_EDGE));
        assert_eq!(input.throttle_milli, 1000);
        assert_eq!(input.turn_milli, -1000);
    }

    #[test]
    fn rollback_game_state_checksum_is_repeatable() {
        let state = RollbackGameState::default();
        assert_eq!(stable_hash(&state), stable_hash(&state));
    }

    #[test]
    fn parses_host_session_descriptor() {
        let descriptor =
            parse_session_descriptor("host@127.0.0.1:5000>127.0.0.1:5001,127.0.0.1:5002").unwrap();
        assert_eq!(descriptor.role, SessionRole::Host);
        assert_eq!(descriptor.local_handle, 0);
        assert_eq!(descriptor.peer_addrs.len(), 2);
    }

    #[test]
    fn parses_client_session_descriptor() {
        let descriptor = parse_session_descriptor("client1@127.0.0.1:5001>127.0.0.1:5000").unwrap();
        assert_eq!(descriptor.role, SessionRole::Client);
        assert_eq!(descriptor.local_handle, 1);
        assert_eq!(descriptor.peer_addrs.len(), 1);
    }
}

fn stable_hash<T: Serialize>(value: &T) -> u128 {
    use std::hash::{Hash, Hasher};

    let encoded = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    encoded.hash(&mut hasher);
    hasher.finish() as u128
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedSessionDescriptor {
    role: SessionRole,
    local_bind_addr: SocketAddr,
    peer_addrs: Vec<SocketAddr>,
    local_handle: PlayerHandle,
}

fn parse_session_descriptor(value: &str) -> Result<ParsedSessionDescriptor, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("session descriptor cannot be empty".to_string());
    }

    let (role_part, rest) = trimmed
        .split_once('@')
        .ok_or_else(|| "expected descriptor format role@bind_addr>peer1,peer2".to_string())?;
    let (bind_part, peers_part) = rest.split_once('>').unwrap_or((rest, ""));
    let local_bind_addr = bind_part
        .parse()
        .map_err(|error| format!("invalid local bind address '{bind_part}': {error}"))?;
    let peer_addrs = if peers_part.trim().is_empty() {
        Vec::new()
    } else {
        peers_part
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .map(|entry| {
                entry
                    .trim()
                    .parse()
                    .map_err(|error| format!("invalid peer address '{}': {error}", entry.trim()))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    let (role, local_handle) = if role_part == "host" {
        (SessionRole::Host, 0)
    } else if let Some(handle) = role_part.strip_prefix("client") {
        let parsed = handle
            .parse::<usize>()
            .map_err(|error| format!("invalid client handle '{handle}': {error}"))?;
        (SessionRole::Client, parsed)
    } else {
        return Err(format!(
            "invalid role '{}', expected 'host' or 'clientN'",
            role_part
        ));
    };

    Ok(ParsedSessionDescriptor {
        role,
        local_bind_addr,
        peer_addrs,
        local_handle,
    })
}

fn load_initial_rollback_state() -> RollbackGameState {
    let mut state = RollbackGameState::default();

    if let Ok(Some(saved_ship)) = load_default_ship() {
        state.editor_ship = saved_ship;
    }

    if let Ok(Some(CampaignSave {
        progression,
        sector,
        last_mission_report,
    })) = load_campaign()
    {
        state.progression = progression;
        state.sector = sector;
        state.last_mission_report = last_mission_report;
    }

    state.phase = RollbackPhase::Docked;
    state
}
