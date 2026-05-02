use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Mutex,
    },
    thread,
    time::Duration,
};

use bevy::{log, prelude::*};
use serde::{Deserialize, Serialize};

use super::{
    bootstrap::ParsedSessionDescriptor,
    RollbackGameState,
    SessionBootstrapConfig,
    SessionPhase,
    SessionRole,
    SessionStatus,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LobbySessionStart {
    local_handle: usize,
    peer_addrs: Vec<SocketAddr>,
    input_delay: usize,
    check_distance: usize,
    initial_state: RollbackGameState,
}

#[derive(Serialize, Deserialize, Debug)]
enum LobbyClientMessage {
    Join { bind_addr: SocketAddr },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum LobbyServerMessage {
    Snapshot(super::LobbySnapshot),
    StartSession(LobbySessionStart),
}

pub(crate) enum LobbyRuntimeEvent {
    Snapshot(super::LobbySnapshot),
    StartSession(LobbySessionStart),
    Failed(String),
}

pub(crate) enum LobbyControlCommand {
    Shutdown,
    StartSession {
        initial_state: RollbackGameState,
        input_delay: usize,
        check_distance: usize,
    },
}

#[derive(Resource)]
pub(crate) struct LobbyRuntime {
    pub(crate) control_tx: Option<Sender<LobbyControlCommand>>,
    pub(crate) event_rx: Option<Mutex<Receiver<LobbyRuntimeEvent>>>,
}

impl Default for LobbyRuntime {
    fn default() -> Self {
        Self {
            control_tx: None,
            event_rx: None,
        }
    }
}

struct HostClientConn {
    stream: TcpStream,
    recv_buffer: Vec<u8>,
    bind_addr: Option<SocketAddr>,
    handle: Option<usize>,
}

pub(crate) fn poll_lobby_runtime_events(
    mut lobby_runtime: ResMut<LobbyRuntime>,
    mut status: ResMut<SessionStatus>,
    mut bootstrap: ResMut<SessionBootstrapConfig>,
) {
    let mut disconnected = false;
    loop {
        let next_event = {
            let Some(event_rx) = lobby_runtime.event_rx.as_ref() else {
                break;
            };
            let Ok(event_rx) = event_rx.lock() else {
                status.phase =
                    SessionPhase::Failed("failed to lock lobby runtime event receiver".to_string());
                disconnected = true;
                break;
            };
            event_rx.try_recv()
        };
        match next_event {
            Ok(LobbyRuntimeEvent::Snapshot(snapshot)) => {
                status.total_players = snapshot.players.len();
                status.lobby_snapshot = Some(snapshot.clone());
                if !matches!(status.phase, SessionPhase::Starting | SessionPhase::Connected) {
                    status.phase = SessionPhase::Lobby;
                }
                log::info!(
                    "Lobby snapshot updated: host={}, players={:?}",
                    snapshot.host_addr,
                    snapshot
                        .players
                        .iter()
                        .map(|player| format!("{}@{}", player.handle, player.bind_addr))
                        .collect::<Vec<_>>()
                );
            }
            Ok(LobbyRuntimeEvent::StartSession(start)) => {
                bootstrap.pending_start = true;
                bootstrap.peer_addrs = start.peer_addrs;
                bootstrap.local_handle = start.local_handle;
                bootstrap.input_delay = start.input_delay;
                bootstrap.check_distance = start.check_distance;
                bootstrap.initial_state = start.initial_state;
                status.phase = SessionPhase::Starting;
                status.total_players = bootstrap.peer_addrs.len() + 1;
                log::info!(
                    "Lobby issued start-session event: local_handle={}, peers={:?}, total_players={}",
                    bootstrap.local_handle,
                    bootstrap.peer_addrs,
                    status.total_players
                );
            }
            Ok(LobbyRuntimeEvent::Failed(error)) => {
                log::warn!("Lobby runtime failed: {}", error);
                status.phase = SessionPhase::Failed(error);
                disconnected = true;
                break;
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                disconnected = true;
                break;
            }
        }
    }

    if disconnected {
        shutdown_lobby_runtime(lobby_runtime.as_mut());
    }
}

pub(crate) fn start_lobby_runtime(
    descriptor: &ParsedSessionDescriptor,
) -> Result<LobbyRuntime, String> {
    let (control_tx, control_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    match descriptor.role {
        SessionRole::Host => {
            let bind_addr = descriptor.local_bind_addr;
            thread::spawn(move || run_host_lobby(bind_addr, event_tx, control_rx));
        }
        SessionRole::Client => {
            let host_addr = *descriptor
                .peer_addrs
                .first()
                .ok_or_else(|| "client descriptor must include a host address after '>'".to_string())?;
            let bind_addr = descriptor.local_bind_addr;
            thread::spawn(move || run_client_lobby(bind_addr, host_addr, event_tx, control_rx));
        }
    }
    Ok(LobbyRuntime {
        control_tx: Some(control_tx),
        event_rx: Some(Mutex::new(event_rx)),
    })
}

pub(crate) fn shutdown_lobby_runtime(lobby_runtime: &mut LobbyRuntime) {
    if let Some(control_tx) = lobby_runtime.control_tx.take() {
        let _ = control_tx.send(LobbyControlCommand::Shutdown);
    }
    lobby_runtime.event_rx = None;
}

fn run_host_lobby(
    bind_addr: SocketAddr,
    event_tx: Sender<LobbyRuntimeEvent>,
    control_rx: Receiver<LobbyControlCommand>,
) {
    let listener = match TcpListener::bind(bind_addr) {
        Ok(listener) => listener,
        Err(error) => {
            let _ = event_tx.send(LobbyRuntimeEvent::Failed(format!(
                "failed to bind host lobby TCP listener on {bind_addr}: {error}"
            )));
            return;
        }
    };
    if let Err(error) = listener.set_nonblocking(true) {
        let _ = event_tx.send(LobbyRuntimeEvent::Failed(format!(
            "failed to configure host lobby listener as nonblocking: {error}"
        )));
        return;
    }

    let mut clients: Vec<HostClientConn> = Vec::new();
    let mut snapshot = host_snapshot(bind_addr, &clients);
    let _ = event_tx.send(LobbyRuntimeEvent::Snapshot(snapshot.clone()));
    log::info!("Host lobby listening on {}", bind_addr);

    'outer: loop {
        loop {
            match control_rx.try_recv() {
                Ok(LobbyControlCommand::Shutdown) => {
                    log::info!("Shutting down host lobby runtime");
                    break 'outer;
                }
                Ok(LobbyControlCommand::StartSession {
                    initial_state,
                    input_delay,
                    check_distance,
                }) => {
                    log::info!(
                        "Host lobby starting rollback session with {} player(s)",
                        snapshot.players.len()
                    );
                    let host_start = LobbySessionStart {
                        local_handle: 0,
                        peer_addrs: snapshot
                            .players
                            .iter()
                            .filter(|player| !player.is_host)
                            .map(|player| player.bind_addr)
                            .collect(),
                        input_delay,
                        check_distance,
                        initial_state: initial_state.clone(),
                    };
                    let _ = event_tx.send(LobbyRuntimeEvent::StartSession(host_start));
                    broadcast_start_session(
                        &mut clients,
                        &snapshot.players,
                        input_delay,
                        check_distance,
                        &initial_state,
                    );
                    break 'outer;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'outer,
            }
        }

        loop {
            match listener.accept() {
                Ok((stream, remote_addr)) => {
                    log::info!("Accepted lobby TCP connection from {}", remote_addr);
                    if let Err(error) = stream.set_nonblocking(true) {
                        log::warn!(
                            "Failed to make host lobby connection {} nonblocking: {}",
                            remote_addr,
                            error
                        );
                        continue;
                    }
                    clients.push(HostClientConn {
                        stream,
                        recv_buffer: Vec::new(),
                        bind_addr: None,
                        handle: None,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    let _ = event_tx.send(LobbyRuntimeEvent::Failed(format!(
                        "host lobby accept failed on {bind_addr}: {error}"
                    )));
                    break 'outer;
                }
            }
        }

        let mut snapshot_changed = false;
        let mut remove_indices = Vec::new();
        for (index, client) in clients.iter_mut().enumerate() {
            let mut buffer = [0u8; 1024];
            loop {
                match client.stream.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("Lobby client {:?} disconnected", client.bind_addr);
                        remove_indices.push(index);
                        break;
                    }
                    Ok(bytes_read) => {
                        client.recv_buffer.extend_from_slice(&buffer[..bytes_read]);
                        while let Some(position) =
                            client.recv_buffer.iter().position(|byte| *byte == b'\n')
                        {
                            let line = client.recv_buffer.drain(..=position).collect::<Vec<_>>();
                            let line = String::from_utf8_lossy(&line);
                            if let Ok(LobbyClientMessage::Join { bind_addr }) =
                                serde_json::from_str::<LobbyClientMessage>(line.trim())
                            {
                                let previous_addr = client.bind_addr;
                                client.bind_addr = Some(bind_addr);
                                if previous_addr != Some(bind_addr) {
                                    log::info!("Lobby join registered for {}", bind_addr);
                                    snapshot_changed = true;
                                }
                            } else {
                                log::warn!(
                                    "Failed to parse host lobby message '{}'",
                                    line.trim()
                                );
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(error) => {
                        log::warn!(
                            "Failed reading from host lobby client {:?}: {}",
                            client.bind_addr,
                            error
                        );
                        remove_indices.push(index);
                        break;
                    }
                }
            }
        }

        if !remove_indices.is_empty() {
            remove_indices.sort_unstable();
            remove_indices.dedup();
            for index in remove_indices.into_iter().rev() {
                clients.remove(index);
            }
            snapshot_changed = true;
        }

        if snapshot_changed {
            reassign_host_client_handles(&mut clients);
            snapshot = host_snapshot(bind_addr, &clients);
            let _ = event_tx.send(LobbyRuntimeEvent::Snapshot(snapshot.clone()));
            broadcast_lobby_snapshot(&mut clients, &snapshot);
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn run_client_lobby(
    local_bind_addr: SocketAddr,
    host_addr: SocketAddr,
    event_tx: Sender<LobbyRuntimeEvent>,
    control_rx: Receiver<LobbyControlCommand>,
) {
    log::info!(
        "Client lobby connecting from {} to host {}",
        local_bind_addr,
        host_addr
    );
    loop {
        match control_rx.try_recv() {
            Ok(LobbyControlCommand::Shutdown) | Err(TryRecvError::Disconnected) => {
                log::info!("Shutting down client lobby runtime");
                return;
            }
            Ok(LobbyControlCommand::StartSession { .. }) | Err(TryRecvError::Empty) => {}
        }

        match TcpStream::connect(host_addr) {
            Ok(mut stream) => {
                log::info!("Connected client lobby TCP stream to host {}", host_addr);
                if let Err(error) = stream.set_nonblocking(true) {
                    let _ = event_tx.send(LobbyRuntimeEvent::Failed(format!(
                        "failed to make client lobby stream nonblocking: {error}"
                    )));
                    return;
                }
                if let Err(error) = send_json_line(
                    &mut stream,
                    &LobbyClientMessage::Join {
                        bind_addr: local_bind_addr,
                    },
                ) {
                    let _ = event_tx.send(LobbyRuntimeEvent::Failed(format!(
                        "failed to send lobby join message: {error}"
                    )));
                    return;
                }
                log::debug!(
                    "Sent lobby join message to host {} with local bind {}",
                    host_addr,
                    local_bind_addr
                );
                let mut recv_buffer = Vec::new();
                loop {
                    match control_rx.try_recv() {
                        Ok(LobbyControlCommand::Shutdown)
                        | Err(TryRecvError::Disconnected) => {
                            log::info!("Shutting down client lobby runtime");
                            return;
                        }
                        Ok(LobbyControlCommand::StartSession { .. })
                        | Err(TryRecvError::Empty) => {}
                    }

                    let mut buffer = [0u8; 1024];
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            log::warn!(
                                "Client lobby TCP stream closed by host {}; retrying connection",
                                host_addr
                            );
                            break;
                        }
                        Ok(bytes_read) => {
                            recv_buffer.extend_from_slice(&buffer[..bytes_read]);
                            while let Some(position) =
                                recv_buffer.iter().position(|byte| *byte == b'\n')
                            {
                                let line = recv_buffer.drain(..=position).collect::<Vec<_>>();
                                let line = String::from_utf8_lossy(&line);
                                match serde_json::from_str::<LobbyServerMessage>(line.trim()) {
                                    Ok(LobbyServerMessage::Snapshot(snapshot)) => {
                                        let _ = event_tx.send(LobbyRuntimeEvent::Snapshot(snapshot));
                                    }
                                    Ok(LobbyServerMessage::StartSession(start)) => {
                                        let _ =
                                            event_tx.send(LobbyRuntimeEvent::StartSession(start));
                                        return;
                                    }
                                    Err(error) => {
                                        log::warn!(
                                            "Failed to parse client lobby message '{}': {}",
                                            line.trim(),
                                            error
                                        );
                                    }
                                }
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(error) => {
                            log::warn!(
                                "Client lobby read error from host {}: {}; retrying connection",
                                host_addr,
                                error
                            );
                            break;
                        }
                    }

                    thread::sleep(Duration::from_millis(25));
                }
            }
            Err(error) => {
                log::debug!(
                    "Client lobby connect to host {} failed: {}; retrying",
                    host_addr,
                    error
                );
                thread::sleep(Duration::from_millis(250));
            }
        }
    }
}

fn reassign_host_client_handles(clients: &mut [HostClientConn]) {
    let mut next_handle = 1;
    for client in clients.iter_mut().filter(|client| client.bind_addr.is_some()) {
        client.handle = Some(next_handle);
        next_handle += 1;
    }
}

fn host_snapshot(host_addr: SocketAddr, clients: &[HostClientConn]) -> super::LobbySnapshot {
    let mut players = vec![super::LobbyPlayerInfo {
        handle: 0,
        bind_addr: host_addr,
        is_host: true,
    }];
    for client in clients
        .iter()
        .filter_map(|client| client.bind_addr.zip(client.handle))
    {
        players.push(super::LobbyPlayerInfo {
            handle: client.1,
            bind_addr: client.0,
            is_host: false,
        });
    }
    players.sort_by_key(|player| player.handle);
    super::LobbySnapshot { host_addr, players }
}

fn broadcast_lobby_snapshot(clients: &mut [HostClientConn], snapshot: &super::LobbySnapshot) {
    for client in clients.iter_mut().filter(|client| client.bind_addr.is_some()) {
        if let Err(error) =
            send_json_line(&mut client.stream, &LobbyServerMessage::Snapshot(snapshot.clone()))
        {
            log::warn!(
                "Failed to send lobby snapshot to client {:?}: {}",
                client.bind_addr,
                error
            );
        }
    }
}

fn broadcast_start_session(
    clients: &mut [HostClientConn],
    players: &[super::LobbyPlayerInfo],
    input_delay: usize,
    check_distance: usize,
    initial_state: &RollbackGameState,
) {
    let ordered_players = players
        .iter()
        .map(|player| player.bind_addr)
        .collect::<Vec<_>>();
    for client in clients.iter_mut().filter(|client| client.bind_addr.is_some()) {
        let Some(local_handle) = client.handle else {
            continue;
        };
        let peer_addrs = ordered_players
            .iter()
            .enumerate()
            .filter(|(handle, _)| *handle != local_handle)
            .map(|(_, addr)| *addr)
            .collect::<Vec<_>>();
        let message = LobbyServerMessage::StartSession(LobbySessionStart {
            local_handle,
            peer_addrs,
            input_delay,
            check_distance,
            initial_state: initial_state.clone(),
        });
        if let Err(error) = send_json_line(&mut client.stream, &message) {
            log::warn!(
                "Failed to send start-session message to client {:?}: {}",
                client.bind_addr,
                error
            );
        }
    }
}

fn send_json_line<T: Serialize>(stream: &mut TcpStream, message: &T) -> std::io::Result<()> {
    let mut encoded = serde_json::to_vec(message)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    encoded.push(b'\n');
    stream.write_all(&encoded)
}
