use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::Duration,
};

use bevy::prelude::*;

use super::{
    gameplay::components::{
        CarriedResource,
        CurrentStation,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerShip,
        ShipControlState,
        ShipRoot,
        ShipWeaponState,
        ShipboardControlState,
    },
    state::{
        ClientAppState,
        ConnectionEvent,
        ConnectionMailbox,
        ConnectionPhase,
        ConnectionStatus,
        DemoProgression,
        EditorShip,
        LastMissionReport,
        MultiplayerDiagnosticsState,
        MultiplayerSessionState,
        MultiplayerSyncGuard,
        MultiplayerTickState,
        NetworkCommandSender,
        SectorState,
    },
};
use crate::{
    protocol::{
        ClientHello,
        ClientMessage,
        PlayerInputFrame,
        PlayerPresenceSnapshot,
        PresenceFrame,
        ServerMessage,
        SessionAppState,
        SessionCommand,
        SessionControlMode,
        SessionSnapshot,
        StateHashReport,
    },
    session::stable_hash_json,
};

pub(crate) fn begin_connection_attempt(
    server_addr: &str,
    status: &mut ConnectionStatus,
    mailbox: &ConnectionMailbox,
    command_sender: &NetworkCommandSender,
) {
    if matches!(status.phase, ConnectionPhase::Connecting) {
        return;
    }

    status.phase = ConnectionPhase::Connecting;
    status.active_snapshot = None;
    status.local_player_id = None;
    clear_mailbox(mailbox);
    if let Ok(mut sender) = command_sender.sender.lock() {
        *sender = None;
    }

    let server_addr = server_addr.to_string();
    let mailbox = mailbox.clone();
    let command_sender = command_sender.clone();
    thread::spawn(move || {
        let event = match connect_to_host(&server_addr, &command_sender, &mailbox) {
            Ok(welcome) => ConnectionEvent::Connected(welcome),
            Err(message) => ConnectionEvent::Failed(message),
        };

        if let Ok(mut events) = mailbox.events.lock() {
            events.push(event);
        }
    });
}

pub(crate) fn poll_connection_events(
    mailbox: Res<ConnectionMailbox>,
    mut status: ResMut<ConnectionStatus>,
    mut next_state: ResMut<NextState<ClientAppState>>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<DemoProgression>,
    mut sector_state: ResMut<SectorState>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut multiplayer_session: ResMut<MultiplayerSessionState>,
    mut diagnostics: ResMut<MultiplayerDiagnosticsState>,
    mut sync_guard: ResMut<MultiplayerSyncGuard>,
) {
    let Ok(mut events) = mailbox.events.lock() else {
        return;
    };

    for event in events.drain(..) {
        match event {
            ConnectionEvent::Connected(welcome) => {
                status.phase = ConnectionPhase::Connected;
                status.local_player_id = Some(welcome.local_player_id);
                status.active_snapshot = Some(welcome.snapshot.ship.clone());
                multiplayer_session.connected = true;
                multiplayer_session.local_player_id = Some(welcome.local_player_id);
                sync_guard.suppress_outbound_once = true;
                apply_authoritative_snapshot(
                    &welcome.snapshot,
                    &mut editor_ship,
                    &mut progression,
                    &mut sector_state,
                    &mut last_mission_report,
                    &mut multiplayer_session,
                    &mut diagnostics,
                    &mut next_state,
                );
            }
            ConnectionEvent::Snapshot(snapshot) => {
                sync_guard.suppress_outbound_once = true;
                apply_authoritative_snapshot(
                    &snapshot,
                    &mut editor_ship,
                    &mut progression,
                    &mut sector_state,
                    &mut last_mission_report,
                    &mut multiplayer_session,
                    &mut diagnostics,
                    &mut next_state,
                );
                status.active_snapshot = Some(snapshot.ship.clone());
            }
            ConnectionEvent::CommittedInputs(batch) => {
                multiplayer_session.session_tick = batch.tick;
                multiplayer_session.last_committed_inputs = batch.frames;
            }
            ConnectionEvent::HashStatus(message) => {
                diagnostics.host_hash = message.host_hash;
                diagnostics.local_hash = message.local_hash;
                diagnostics.last_category = Some(message.category.clone());
                diagnostics.last_message = Some(if message.matched {
                    "hash match".to_string()
                } else {
                    "hash mismatch".to_string()
                });
                if message.matched {
                    diagnostics.last_matching_tick = Some(message.tick);
                    diagnostics.waiting_for_resync = false;
                }
            }
            ConnectionEvent::Drift(message) => {
                diagnostics.host_hash = message.host_hash;
                diagnostics.local_hash = message.local_hash;
                diagnostics
                    .first_mismatching_tick
                    .get_or_insert(message.tick);
                diagnostics.last_category = Some(message.category);
                diagnostics.last_message = Some(message.message);
                diagnostics.mismatch_count += 1;
                diagnostics.waiting_for_resync = true;
            }
            ConnectionEvent::Failed(message) => {
                status.phase = ConnectionPhase::Failed(message);
                multiplayer_session.connected = false;
            }
        }
    }
}

pub(crate) fn sync_app_state_to_host(
    app_state: Res<State<ClientAppState>>,
    status: Res<ConnectionStatus>,
    sync_guard: ResMut<MultiplayerSyncGuard>,
    command_sender: Res<NetworkCommandSender>,
) {
    if !app_state.is_changed() || !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    if suppress_once(sync_guard) {
        return;
    }
    let session_state = SessionAppState::from(app_state.get().clone());
    let _ = send_message(
        &command_sender,
        ClientMessage::Command(SessionCommand::SetAppState(session_state)),
    );
}

pub(crate) fn sync_ship_to_host(
    editor_ship: Res<EditorShip>,
    status: Res<ConnectionStatus>,
    sync_guard: ResMut<MultiplayerSyncGuard>,
    command_sender: Res<NetworkCommandSender>,
) {
    if !editor_ship.is_changed() || !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    if suppress_once(sync_guard) {
        return;
    }
    let _ = send_message(
        &command_sender,
        ClientMessage::Command(SessionCommand::UpdateShip(editor_ship.ship.clone())),
    );
}

pub(crate) fn sync_campaign_to_host(
    progression: Res<DemoProgression>,
    sector_state: Res<SectorState>,
    last_mission_report: Res<LastMissionReport>,
    status: Res<ConnectionStatus>,
    sync_guard: ResMut<MultiplayerSyncGuard>,
    command_sender: Res<NetworkCommandSender>,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    if suppress_once(sync_guard) {
        return;
    }
    if progression.is_changed() {
        let _ = send_message(
            &command_sender,
            ClientMessage::Command(SessionCommand::UpdateProgression(progression.clone())),
        );
    }
    if sector_state.is_changed() {
        let _ = send_message(
            &command_sender,
            ClientMessage::Command(SessionCommand::UpdateSector(sector_state.clone())),
        );
    }
    if last_mission_report.is_changed() {
        let _ = send_message(
            &command_sender,
            ClientMessage::Command(SessionCommand::UpdateMissionReport(
                last_mission_report.clone(),
            )),
        );
    }
}

pub(crate) fn advance_multiplayer_tick(
    status: Res<ConnectionStatus>,
    mut tick_state: ResMut<MultiplayerTickState>,
) {
    if matches!(status.phase, ConnectionPhase::Connected) {
        tick_state.current_tick += 1;
    }
}

pub(crate) fn send_local_multiplayer_presence(
    status: Res<ConnectionStatus>,
    session_state: Res<MultiplayerSessionState>,
    command_sender: Res<NetworkCommandSender>,
    player_query: Query<
        (
            &PlayerMotionState,
            &CurrentStation,
            &CarriedResource,
            &ShipboardControlState,
        ),
        With<crate::client::gameplay::components::ShipboardPlayer>,
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };
    let (motion, current_station, carried, control_state) = player_query.single();

    let presence = PlayerPresenceSnapshot {
        player_id: local_player_id,
        frame: match motion.frame {
            PlayerReferenceFrame::World => PresenceFrame::World,
            PlayerReferenceFrame::Ship(_) => PresenceFrame::Ship,
        },
        world_position: [
            motion.world_position.x.to_num::<f32>(),
            motion.world_position.y.to_num::<f32>(),
        ],
        world_velocity: [
            motion.world_velocity.x.to_num::<f32>(),
            motion.world_velocity.y.to_num::<f32>(),
        ],
        local_position: [
            motion.local_position.x.to_num::<f32>(),
            motion.local_position.y.to_num::<f32>(),
        ],
        local_velocity: [
            motion.local_velocity.x.to_num::<f32>(),
            motion.local_velocity.y.to_num::<f32>(),
        ],
        current_station_module_id: Some(current_station.module_id),
        carried_kind: carried.kind.map(|kind| format!("{kind:?}")),
        carried_amount: carried.amount,
        control_mode: control_mode_to_session(control_state.mode),
    };
    let _ = send_message(&command_sender, ClientMessage::Presence(presence));
}

pub(crate) fn send_local_multiplayer_input(
    status: Res<ConnectionStatus>,
    session_state: Res<MultiplayerSessionState>,
    tick_state: Res<MultiplayerTickState>,
    command_sender: Res<NetworkCommandSender>,
    ship_query: Query<
        (&ShipControlState, &ShipWeaponState, &ShipboardControlState),
        (With<PlayerShip>, With<ShipRoot>),
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };
    let (control_state, weapon_state, station_state) = ship_query.single();
    let frame = PlayerInputFrame {
        player_id: local_player_id,
        tick: tick_state.current_tick.saturating_add(2),
        throttle_milli: (control_state.throttle_demand.to_num::<f32>() * 1000.0) as i32,
        turn_milli: (control_state.turn_input.to_num::<f32>() * 1000.0) as i32,
        fire_pressed: control_state.fire_pressed || weapon_state.turret_count > 0,
        focus_module_id: station_state.focused_module_id,
        control_mode: control_mode_to_session(station_state.mode),
        carrying: false,
        interaction_active: station_state.focused_entity.is_some(),
    };
    let _ = send_message(&command_sender, ClientMessage::InputFrame(frame));
}

pub(crate) fn send_multiplayer_hash_report(
    status: Res<ConnectionStatus>,
    session_state: Res<MultiplayerSessionState>,
    tick_state: Res<MultiplayerTickState>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    sector_state: Res<SectorState>,
    last_mission_report: Res<LastMissionReport>,
    app_state: Res<State<ClientAppState>>,
    mut diagnostics: ResMut<MultiplayerDiagnosticsState>,
    command_sender: Res<NetworkCommandSender>,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) || tick_state.current_tick % 10 != 0 {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };

    let snapshot = SessionSnapshot {
        tick: tick_state.current_tick,
        app_state: SessionAppState::from(app_state.get().clone()),
        ship: editor_ship.ship.clone(),
        progression: progression.clone(),
        sector: sector_state.clone(),
        last_mission_report: last_mission_report.clone(),
        peers: session_state
            .remote_players
            .iter()
            .map(|peer| crate::protocol::SessionPeerInfo {
                player_id: peer.player_id,
                display_name: peer.display_name.clone(),
                connected: peer.connected,
                last_input_tick: tick_state.current_tick,
                presence: peer.presence.clone(),
            })
            .collect(),
        state_hash: 0,
    };
    let local_hash = stable_hash_json(&snapshot);
    diagnostics.local_hash = local_hash;
    let _ = send_message(
        &command_sender,
        ClientMessage::HashReport(StateHashReport {
            player_id: local_player_id,
            tick: tick_state.current_tick,
            local_hash,
            category: "session".to_string(),
        }),
    );
}

pub(crate) fn request_resync_when_waiting(
    status: Res<ConnectionStatus>,
    diagnostics: Res<MultiplayerDiagnosticsState>,
    command_sender: Res<NetworkCommandSender>,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) || !diagnostics.waiting_for_resync {
        return;
    }
    let _ = send_message(
        &command_sender,
        ClientMessage::RequestResync {
            category: diagnostics
                .last_category
                .clone()
                .unwrap_or_else(|| "session".to_string()),
        },
    );
}

fn connect_to_host(
    server_addr: &str,
    command_sender: &NetworkCommandSender,
    mailbox: &ConnectionMailbox,
) -> Result<crate::protocol::SessionWelcome, String> {
    let mut stream = TcpStream::connect(server_addr)
        .map_err(|error| format!("failed to connect to {server_addr}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("failed to set read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("failed to set write timeout: {error}"))?;

    let hello = ClientMessage::Hello(ClientHello::new("multiplayer_client", "Operator"));
    let encoded = serde_json::to_string(&hello)
        .map_err(|error| format!("failed to encode client hello: {error}"))?;
    stream
        .write_all(encoded.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|error| format!("failed to send client hello: {error}"))?;

    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|error| format!("failed to clone connection stream: {error}"))?,
    );
    let welcome = read_server_message(&mut reader).and_then(|message| match message {
        ServerMessage::SessionWelcome(welcome) => Ok(welcome),
        ServerMessage::Error { message } => Err(message),
        _ => Err("expected SessionWelcome from host".to_string()),
    })?;

    let write_stream = stream
        .try_clone()
        .map_err(|error| format!("failed to clone write stream: {error}"))?;
    let read_stream = stream;
    let (tx, rx) = mpsc::channel::<ClientMessage>();
    if let Ok(mut sender) = command_sender.sender.lock() {
        *sender = Some(tx.clone());
    }

    let mailbox_reader = mailbox.clone();
    thread::spawn(move || network_reader_loop(read_stream, mailbox_reader));
    thread::spawn(move || network_writer_loop(write_stream, rx));

    Ok(welcome)
}

fn network_reader_loop(stream: TcpStream, mailbox: ConnectionMailbox) {
    let mut reader = BufReader::new(stream);
    while let Ok(message) = read_server_message(&mut reader) {
        let event = match message {
            ServerMessage::SessionWelcome(welcome) => ConnectionEvent::Connected(welcome),
            ServerMessage::SessionSnapshot(snapshot) => ConnectionEvent::Snapshot(snapshot),
            ServerMessage::CommittedInputs(batch) => ConnectionEvent::CommittedInputs(batch),
            ServerMessage::HashStatus(message) => ConnectionEvent::HashStatus(message),
            ServerMessage::DriftDetected(message) => ConnectionEvent::Drift(message),
            ServerMessage::Error { message } => ConnectionEvent::Failed(message),
        };
        if let Ok(mut events) = mailbox.events.lock() {
            events.push(event);
        }
    }

    if let Ok(mut events) = mailbox.events.lock() {
        events.push(ConnectionEvent::Failed(
            "connection to host closed".to_string(),
        ));
    }
}

fn network_writer_loop(mut stream: TcpStream, rx: mpsc::Receiver<ClientMessage>) {
    for message in rx {
        let Ok(encoded) = serde_json::to_string(&message) else {
            continue;
        };
        if stream
            .write_all(encoded.as_bytes())
            .and_then(|_| stream.write_all(b"\n"))
            .is_err()
        {
            break;
        }
    }
}

fn read_server_message(reader: &mut BufReader<TcpStream>) -> Result<ServerMessage, String> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("failed to read host response: {error}"))?;
    if line.trim().is_empty() {
        return Err("host closed the connection".to_string());
    }
    serde_json::from_str(&line).map_err(|error| format!("failed to decode host response: {error}"))
}

fn clear_mailbox(mailbox: &ConnectionMailbox) {
    if let Ok(mut events) = mailbox.events.lock() {
        events.clear();
    }
}

fn apply_authoritative_snapshot(
    snapshot: &SessionSnapshot,
    editor_ship: &mut ResMut<EditorShip>,
    progression: &mut ResMut<DemoProgression>,
    sector_state: &mut ResMut<SectorState>,
    last_mission_report: &mut ResMut<LastMissionReport>,
    multiplayer_session: &mut ResMut<MultiplayerSessionState>,
    diagnostics: &mut ResMut<MultiplayerDiagnosticsState>,
    next_state: &mut ResMut<NextState<ClientAppState>>,
) {
    editor_ship.ship = snapshot.ship.clone();
    **progression = snapshot.progression.clone();
    **sector_state = snapshot.sector.clone();
    **last_mission_report = snapshot.last_mission_report.clone();
    multiplayer_session.session_tick = snapshot.tick;
    multiplayer_session.remote_players = snapshot
        .peers
        .iter()
        .map(|peer| super::state::RemotePlayerState {
            player_id: peer.player_id,
            display_name: peer.display_name.clone(),
            role: if Some(peer.player_id) == multiplayer_session.local_player_id {
                super::state::MultiplayerPlayerRole::Local
            } else {
                super::state::MultiplayerPlayerRole::Remote
            },
            connected: peer.connected,
            presence: peer.presence.clone(),
        })
        .collect();
    diagnostics.host_hash = snapshot.state_hash;
    diagnostics.last_matching_tick = Some(snapshot.tick);
    diagnostics.waiting_for_resync = false;
    diagnostics.last_resync_tick = Some(snapshot.tick);
    next_state.set(snapshot.app_state.into());
}

fn send_message(
    command_sender: &NetworkCommandSender,
    message: ClientMessage,
) -> Result<(), String> {
    let sender = command_sender
        .sender
        .lock()
        .map_err(|_| "failed to lock command sender".to_string())?;
    let Some(sender) = sender.as_ref() else {
        return Err("no active network sender".to_string());
    };
    sender
        .send(message)
        .map_err(|error| format!("failed to send network message: {error}"))
}

fn suppress_once(mut sync_guard: ResMut<MultiplayerSyncGuard>) -> bool {
    if sync_guard.suppress_outbound_once {
        sync_guard.suppress_outbound_once = false;
        true
    } else {
        false
    }
}

fn control_mode_to_session(
    mode: crate::client::gameplay::components::ShipControlMode,
) -> SessionControlMode {
    match mode {
        crate::client::gameplay::components::ShipControlMode::Interior => {
            SessionControlMode::Interior
        }
        crate::client::gameplay::components::ShipControlMode::Cockpit => {
            SessionControlMode::Cockpit
        }
        crate::client::gameplay::components::ShipControlMode::Turret => SessionControlMode::Turret,
        crate::client::gameplay::components::ShipControlMode::Reactor => {
            SessionControlMode::Reactor
        }
        crate::client::gameplay::components::ShipControlMode::Logistics => {
            SessionControlMode::Logistics
        }
        crate::client::gameplay::components::ShipControlMode::Computer => {
            SessionControlMode::Computer
        }
    }
}
