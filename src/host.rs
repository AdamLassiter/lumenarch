use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::{
    client::{
        campaign::{CampaignSave, load_campaign, save_campaign},
        state::{DemoProgression, LastMissionReport, SectorNodeStatus, SectorState},
    },
    protocol::{
        ClientHello,
        ClientMessage,
        CommittedInputBatch,
        DriftDetectedMessage,
        HashStatusMessage,
        PlayerInputFrame,
        PlayerPresenceSnapshot,
        ServerMessage,
        SessionAppState,
        SessionCommand,
        SessionPeerInfo,
        SessionSnapshot,
        SessionWelcome,
        ShipSnapshot,
    },
    session::snapshot_with_hash,
    ship::storage::{load_default_ship, save_default_ship},
};

const DEFAULT_HOST_ADDR: &str = "127.0.0.1:5000";
const HOST_TICK_MILLIS: u64 = 100;
const SNAPSHOT_BROADCAST_INTERVAL: u64 = 5;

pub fn run_host() -> Result<(), String> {
    let listener = TcpListener::bind(DEFAULT_HOST_ADDR)
        .map_err(|error| format!("failed to bind host on {DEFAULT_HOST_ADDR}: {error}"))?;

    let session = Arc::new(Mutex::new(HostSession::load()));
    spawn_host_tick_thread(session.clone());

    println!("host: listening on {DEFAULT_HOST_ADDR}");
    println!("host: accepting multiplayer session connections");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let peer = stream
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string());
                println!("host: accepted connection from {peer}");
                let session = session.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_client(stream, session) {
                        eprintln!("host: failed to serve client {peer}: {error}");
                    }
                });
            }
            Err(error) => {
                eprintln!("host: failed to accept incoming connection: {error}");
            }
        }
    }

    Ok(())
}

struct ConnectedPeer {
    display_name: String,
    connected: bool,
    sender: mpsc::Sender<ServerMessage>,
    last_input_tick: u64,
    last_hash: u64,
    presence: PlayerPresenceSnapshot,
}

struct HostSession {
    next_player_id: u32,
    tick: u64,
    app_state: SessionAppState,
    ship: ShipSnapshot,
    progression: DemoProgression,
    sector: SectorState,
    last_mission_report: LastMissionReport,
    peers: BTreeMap<u32, ConnectedPeer>,
    pending_inputs: BTreeMap<u64, Vec<PlayerInputFrame>>,
}

impl HostSession {
    fn load() -> Self {
        let ship = match load_default_ship() {
            Ok(Some(ship)) => ship,
            Ok(None) => ShipSnapshot::empty("Untitled Knot"),
            Err(error) => {
                eprintln!("host: failed to load default ship: {error}");
                ShipSnapshot::empty("Untitled Knot")
            }
        };

        let (progression, sector, last_mission_report) = match load_campaign() {
            Ok(Some(save)) => (save.progression, save.sector, save.last_mission_report),
            Ok(None) => (
                DemoProgression::default(),
                SectorState::default(),
                LastMissionReport::default(),
            ),
            Err(error) => {
                eprintln!("host: failed to load campaign state: {error}");
                (
                    DemoProgression::default(),
                    SectorState::default(),
                    LastMissionReport::default(),
                )
            }
        };

        Self {
            next_player_id: 1,
            tick: 0,
            app_state: SessionAppState::Docked,
            ship,
            progression,
            sector,
            last_mission_report,
            peers: BTreeMap::new(),
            pending_inputs: BTreeMap::new(),
        }
    }

    fn snapshot(&self) -> SessionSnapshot {
        let snapshot = SessionSnapshot {
            tick: self.tick,
            app_state: self.app_state,
            ship: self.ship.clone(),
            progression: self.progression.clone(),
            sector: self.sector.clone(),
            last_mission_report: self.last_mission_report.clone(),
            peers: self
                .peers
                .iter()
                .map(|(player_id, peer)| SessionPeerInfo {
                    player_id: *player_id,
                    display_name: peer.display_name.clone(),
                    connected: peer.connected,
                    last_input_tick: peer.last_input_tick,
                    presence: peer.presence.clone(),
                })
                .collect(),
            state_hash: 0,
        };
        snapshot_with_hash(snapshot)
    }

    fn persist(&self) {
        if let Err(error) = save_default_ship(&self.ship) {
            eprintln!("host: failed to persist ship: {error}");
        }
        let save = CampaignSave {
            progression: self.progression.clone(),
            sector: self.sector.clone(),
            last_mission_report: self.last_mission_report.clone(),
        };
        if let Err(error) = save_campaign(&save) {
            eprintln!("host: failed to persist campaign: {error}");
        }
    }
}

fn spawn_host_tick_thread(session: Arc<Mutex<HostSession>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(HOST_TICK_MILLIS));

            let (snapshot, input_batch, recipients) = {
                let Ok(mut session) = session.lock() else {
                    continue;
                };
                session.tick += 1;
                let tick = session.tick;
                let frames = session.pending_inputs.remove(&tick).unwrap_or_default();
                let batch = CommittedInputBatch { tick, frames };
                let snapshot = if session.tick % SNAPSHOT_BROADCAST_INTERVAL == 0 {
                    Some(session.snapshot())
                } else {
                    None
                };
                let recipients = session
                    .peers
                    .iter()
                    .map(|(player_id, peer)| (*player_id, peer.sender.clone()))
                    .collect::<Vec<_>>();
                (snapshot, batch, recipients)
            };

            if !input_batch.frames.is_empty() {
                broadcast_message(&recipients, ServerMessage::CommittedInputs(input_batch));
            }
            if let Some(snapshot) = snapshot {
                broadcast_message(&recipients, ServerMessage::SessionSnapshot(snapshot));
            }
        }
    });
}

fn handle_client(stream: TcpStream, session: Arc<Mutex<HostSession>>) -> Result<(), String> {
    let writer_stream = stream
        .try_clone()
        .map_err(|error| format!("failed to clone client stream: {error}"))?;
    let (sender, receiver) = mpsc::channel::<ServerMessage>();
    thread::spawn(move || {
        let mut writer_stream = writer_stream;
        for message in receiver {
            let Ok(encoded) = serde_json::to_string(&message) else {
                continue;
            };
            if writer_stream
                .write_all(encoded.as_bytes())
                .and_then(|_| writer_stream.write_all(b"\n"))
                .is_err()
            {
                break;
            }
        }
    });

    let mut reader = BufReader::new(stream);
    let first_message = read_client_message(&mut reader)?;
    let hello = match first_message {
        ClientMessage::Hello(hello) => hello,
        _ => return Err("expected ClientMessage::Hello as first packet".to_string()),
    };

    let (player_id, welcome, recipients) = {
        let mut session = session
            .lock()
            .map_err(|_| "failed to lock host session".to_string())?;
        let player_id = session.next_player_id;
        session.next_player_id += 1;
        session.peers.insert(
            player_id,
            ConnectedPeer {
                display_name: hello.player_name.clone(),
                connected: true,
                sender: sender.clone(),
                last_input_tick: 0,
                last_hash: 0,
                presence: PlayerPresenceSnapshot {
                    player_id,
                    ..Default::default()
                },
            },
        );
        let snapshot = session.snapshot();
        let welcome = SessionWelcome {
            local_player_id: player_id,
            snapshot: snapshot.clone(),
        };
        let recipients = session
            .peers
            .iter()
            .map(|(peer_id, peer)| (*peer_id, peer.sender.clone()))
            .collect::<Vec<_>>();
        (player_id, welcome, recipients)
    };

    sender
        .send(ServerMessage::SessionWelcome(welcome))
        .map_err(|error| format!("failed to send session welcome: {error}"))?;
    broadcast_message(
        &recipients,
        ServerMessage::SessionSnapshot(current_snapshot(&session)?),
    );

    loop {
        let message = match read_client_message(&mut reader) {
            Ok(message) => message,
            Err(error) => {
                let mut session = session
                    .lock()
                    .map_err(|_| "failed to lock host session".to_string())?;
                session.peers.remove(&player_id);
                let snapshot = session.snapshot();
                let recipients = session
                    .peers
                    .iter()
                    .map(|(peer_id, peer)| (*peer_id, peer.sender.clone()))
                    .collect::<Vec<_>>();
                drop(session);
                broadcast_message(&recipients, ServerMessage::SessionSnapshot(snapshot));
                return Err(error);
            }
        };

        let mut session = session
            .lock()
            .map_err(|_| "failed to lock host session".to_string())?;
        match message {
            ClientMessage::Hello(_) => {}
            ClientMessage::Command(command) => {
                apply_command(&mut session, command);
                session.persist();
                let snapshot = session.snapshot();
                let recipients = collect_recipients(&session);
                drop(session);
                broadcast_message(&recipients, ServerMessage::SessionSnapshot(snapshot));
            }
            ClientMessage::InputFrame(frame) => {
                session
                    .pending_inputs
                    .entry(frame.tick)
                    .or_default()
                    .push(frame.clone());
                if let Some(peer) = session.peers.get_mut(&player_id) {
                    peer.last_input_tick = frame.tick;
                }
            }
            ClientMessage::Presence(presence) => {
                if let Some(peer) = session.peers.get_mut(&player_id) {
                    peer.presence = presence;
                    peer.presence.player_id = player_id;
                }
            }
            ClientMessage::HashReport(report) => {
                let snapshot = session.snapshot();
                let matched = report.local_hash == snapshot.state_hash;
                let server_message = if matched {
                    ServerMessage::HashStatus(HashStatusMessage {
                        tick: snapshot.tick,
                        player_id,
                        host_hash: snapshot.state_hash,
                        local_hash: report.local_hash,
                        matched: true,
                        category: report.category.clone(),
                    })
                } else {
                    ServerMessage::DriftDetected(DriftDetectedMessage {
                        tick: snapshot.tick,
                        player_id,
                        host_hash: snapshot.state_hash,
                        local_hash: report.local_hash,
                        category: report.category.clone(),
                        message: "authoritative snapshot mismatch, resync required".to_string(),
                    })
                };
                if let Some(peer) = session.peers.get_mut(&player_id) {
                    peer.last_hash = report.local_hash;
                    let _ = peer.sender.send(server_message);
                    if !matched {
                        let _ = peer
                            .sender
                            .send(ServerMessage::SessionSnapshot(snapshot.clone()));
                    }
                }
            }
            ClientMessage::RequestResync { .. } => {
                let snapshot = session.snapshot();
                if let Some(peer) = session.peers.get(&player_id) {
                    let _ = peer.sender.send(ServerMessage::SessionSnapshot(snapshot));
                }
            }
        }
    }
}

fn current_snapshot(session: &Arc<Mutex<HostSession>>) -> Result<SessionSnapshot, String> {
    session
        .lock()
        .map(|session| session.snapshot())
        .map_err(|_| "failed to lock host session".to_string())
}

fn collect_recipients(session: &HostSession) -> Vec<(u32, mpsc::Sender<ServerMessage>)> {
    session
        .peers
        .iter()
        .map(|(player_id, peer)| (*player_id, peer.sender.clone()))
        .collect()
}

fn broadcast_message(recipients: &[(u32, mpsc::Sender<ServerMessage>)], message: ServerMessage) {
    for (_, sender) in recipients {
        let _ = sender.send(message.clone());
    }
}

fn read_client_message(reader: &mut BufReader<TcpStream>) -> Result<ClientMessage, String> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("failed to read client message: {error}"))?;
    if line.trim().is_empty() {
        return Err("client disconnected".to_string());
    }

    serde_json::from_str::<ClientMessage>(&line).or_else(|_| {
        serde_json::from_str::<ClientHello>(&line)
            .map(ClientMessage::Hello)
            .map_err(|error| format!("failed to decode client message: {error}"))
    })
}

fn apply_command(session: &mut HostSession, command: SessionCommand) {
    match command {
        SessionCommand::SetAppState(state) => {
            session.app_state = state;
        }
        SessionCommand::UpdateShip(ship) => {
            session.ship = ship;
        }
        SessionCommand::UpdateProgression(progression) => {
            session.progression = progression;
        }
        SessionCommand::UpdateSector(sector) => {
            session.sector = sector;
        }
        SessionCommand::UpdateMissionReport(report) => {
            session.last_mission_report = report;
        }
        SessionCommand::SelectSectorNode(selected) => {
            session.sector.selected_node_id = selected;
        }
        SessionCommand::SetActiveEncounterNode(active) => {
            session.sector.active_encounter_node_id = active;
        }
        SessionCommand::ApplyTravelOutcome(outcome) => {
            session.progression.scrap += outcome.scrap_awarded;
            session.progression.hull_wear = session
                .progression
                .hull_wear
                .saturating_add(outcome.hull_wear_delta);
            if let Some(node) = session
                .sector
                .nodes
                .iter_mut()
                .find(|node| node.id == outcome.node_id)
            {
                node.status = if outcome.failed {
                    SectorNodeStatus::Failed
                } else if outcome.exhausted {
                    SectorNodeStatus::Exhausted
                } else if outcome.success {
                    SectorNodeStatus::Completed
                } else {
                    node.status
                };
            }
        }
        SessionCommand::RepairShip => {
            let repair_cost = session.progression.hull_wear.saturating_mul(2);
            if repair_cost > 0 && session.progression.scrap >= repair_cost {
                session.progression.scrap -= repair_cost;
                session.progression.hull_wear = 0;
            }
        }
    }
}
