use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use bevy::{log, prelude::*};
use bevy_ggrs::{
    Session,
    prelude::{PlayerType, SessionBuilder},
};
use ggrs::{PlayerHandle, UdpNonBlockingSocket};
use serde::Serialize;

use super::{
    DecodedPlayerCommands,
    LobbyRuntime,
    lobby::{LobbyControlCommand, shutdown_lobby_runtime, start_lobby_runtime},
    types::{
        LumenGgrsConfig,
        PendingLocalMetaCommand,
        PendingLocalStationCommand,
        RollbackGameState,
        RollbackPhase,
        SessionBootstrapConfig,
        SessionConfig,
        SessionPhase,
        SessionRole,
        SessionStatus,
    },
};
use crate::{
    campaign::{CampaignSave, load_campaign},
    ship::storage::load_default_ship,
    state::{FrontendMode, LocalPlayerProfile},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParsedSessionDescriptor {
    pub(crate) role: SessionRole,
    pub(crate) local_bind_addr: SocketAddr,
    pub(crate) peer_addrs: Vec<SocketAddr>,
    pub(crate) local_handle: PlayerHandle,
}

pub(crate) fn begin_session_attempt(
    config: &SessionConfig,
    local_profile: &LocalPlayerProfile,
    status: &mut SessionStatus,
    bootstrap: &mut SessionBootstrapConfig,
    lobby_runtime: &mut LobbyRuntime,
) {
    log::info!(
        "Beginning session bootstrap for descriptor '{}' (delay={}, check_distance={})",
        config.session_descriptor,
        config.input_delay,
        config.check_distance
    );
    shutdown_lobby_runtime(lobby_runtime);
    status.lobby_snapshot = None;
    match parse_session_descriptor(&config.session_descriptor) {
        Ok(descriptor) => {
            log::debug!(
                "Parsed session descriptor into role={:?}, local_handle={}, bind={}, peers={:?}",
                descriptor.role,
                descriptor.local_handle,
                descriptor.local_bind_addr,
                descriptor.peer_addrs,
            );
            status.phase = SessionPhase::Connecting;
            status.role = Some(descriptor.role);
            bootstrap.pending_start = false;
            bootstrap.role = descriptor.role;
            bootstrap.local_bind_addr = descriptor.local_bind_addr;
            bootstrap.peer_addrs.clear();
            bootstrap.local_handle = if descriptor.role == SessionRole::Host {
                0
            } else {
                descriptor.local_handle
            };
            bootstrap.input_delay = config.input_delay;
            bootstrap.check_distance = config.check_distance;
            bootstrap.initial_state = load_initial_rollback_state();
            status.total_players = 1;
            status.active_ship_snapshot = Some(bootstrap.initial_state.editor_ship.clone());
            match start_lobby_runtime(&descriptor, local_profile) {
                Ok(runtime) => {
                    *lobby_runtime = runtime;
                    log::info!(
                        "Started {:?} lobby channel on {}",
                        descriptor.role,
                        descriptor.local_bind_addr
                    );
                }
                Err(error) => {
                    log::warn!("Failed to start lobby runtime: {}", error);
                    status.phase = SessionPhase::Failed(error);
                    return;
                }
            }
            log::info!(
                "Prepared rollback bootstrap state: phase={:?}, ship='{}', modules={}, current_node={}, selected_node={:?}",
                bootstrap.initial_state.phase,
                bootstrap.initial_state.editor_ship.name,
                bootstrap.initial_state.editor_ship.modules.len(),
                bootstrap.initial_state.sector.current_node_id,
                bootstrap.initial_state.sector.selected_node_id
            );
        }
        Err(error) => {
            log::warn!(
                "Failed to parse session descriptor '{}': {}",
                config.session_descriptor,
                error
            );
            status.phase = SessionPhase::Failed(error);
            bootstrap.pending_start = false;
        }
    }
}

pub(crate) fn request_lobby_session_start(
    status: &mut SessionStatus,
    bootstrap: &SessionBootstrapConfig,
    lobby_runtime: &LobbyRuntime,
) {
    let Some(control_tx) = lobby_runtime.control_tx.as_ref() else {
        log::warn!("Host requested lobby start, but no lobby runtime is active");
        status.phase = SessionPhase::Failed("lobby runtime is not active".to_string());
        return;
    };
    if status.role != Some(SessionRole::Host) {
        log::debug!("Ignoring lobby start request on non-host peer");
        return;
    }
    let player_count = status
        .lobby_snapshot
        .as_ref()
        .map(|snapshot| snapshot.players.len())
        .unwrap_or(1);
    log::info!(
        "Host requested rollback session start from lobby with {} player(s)",
        player_count
    );
    if let Err(error) = control_tx.send(LobbyControlCommand::StartSession {
        initial_state: Box::new(bootstrap.initial_state.clone()),
        input_delay: bootstrap.input_delay,
        check_distance: bootstrap.check_distance,
    }) {
        let message = format!("failed to send start-session command to lobby runtime: {error}");
        log::warn!("{}", message);
        status.phase = SessionPhase::Failed(message);
        return;
    }
    status.phase = SessionPhase::Starting;
}

/// Finalizes a pending lobby start into a live rollback session so the frontend can enter gameplay.
pub(crate) fn finalize_pending_session_bootstrap(
    mut commands: Commands,
    mut bootstrap: ResMut<SessionBootstrapConfig>,
    mut status: ResMut<SessionStatus>,
    mut rollback_state: ResMut<RollbackGameState>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
    mut pending_station: ResMut<PendingLocalStationCommand>,
    mut decoded_commands: ResMut<DecodedPlayerCommands>,
    mut lobby_runtime: ResMut<LobbyRuntime>,
) {
    if !bootstrap.pending_start {
        return;
    }

    log::debug!(
        "Finalizing pending session bootstrap: role={:?}, local_handle={}, bind={}, peers={:?}",
        bootstrap.role,
        bootstrap.local_handle,
        bootstrap.local_bind_addr,
        bootstrap.peer_addrs
    );

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
            pending_meta.0 = None;
            pending_station.0 = None;
            decoded_commands.by_handle.clear();
            bootstrap.pending_start = false;
            shutdown_lobby_runtime(lobby_runtime.as_mut());
            next_mode.set(FrontendMode::Session);
            log::info!(
                "Successfully started P2P session as {:?} with local handle {:?} and peer addresses {:?}",
                bootstrap.role,
                bootstrap.local_handle,
                bootstrap.peer_addrs
            );
            log::info!("Switching to session frontend");
        }
        Err(error) => {
            log::warn!(
                "Failed to start P2P session as {:?} on {} with peers {:?}: {}",
                bootstrap.role,
                bootstrap.local_bind_addr,
                bootstrap.peer_addrs,
                error
            );
            status.phase = SessionPhase::Failed(error);
            bootstrap.pending_start = false;
        }
    }
}

pub(crate) fn stable_hash<T: Serialize>(value: &T) -> u128 {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    encoded.hash(&mut hasher);
    hasher.finish() as u128
}

fn build_p2p_session(
    local_bind_addr: SocketAddr,
    local_handle: PlayerHandle,
    peer_addrs: &[SocketAddr],
    input_delay: usize,
    check_distance: usize,
) -> Result<ggrs::P2PSession<LumenGgrsConfig>, String> {
    let player_count = peer_addrs.len() + 1;
    log::debug!(
        "Building P2P session: local_bind_addr={}, local_handle={}, peers={:?}, player_count={}, input_delay={}, check_distance={}",
        local_bind_addr,
        local_handle,
        peer_addrs,
        player_count,
        input_delay,
        check_distance
    );
    let mut builder = SessionBuilder::<LumenGgrsConfig>::new()
        .with_num_players(player_count)
        .unwrap()
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
        log::trace!("Registered player handle {} in P2P session builder", handle);
    }

    let socket = UdpNonBlockingSocket::bind_to_port(local_bind_addr.port())
        .map_err(|error| format!("failed to bind UDP socket on {}: {error}", local_bind_addr))?;
    log::debug!(
        "Bound UDP socket for GGRS P2P session on {}",
        local_bind_addr
    );
    builder
        .start_p2p_session(socket)
        .map_err(|error| format!("failed to start GGRS P2P session: {error}"))
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

pub(crate) fn load_initial_rollback_state() -> RollbackGameState {
    let mut state = RollbackGameState::default();

    if let Ok(Some(saved_ship)) = load_default_ship() {
        state.editor_ship = saved_ship;
    }

    if let Ok(Some(CampaignSave {
        progression,
        mut sector,
        last_mission_report,
    })) = load_campaign()
    {
        let previous_enemy_ship_ids = sector
            .nodes
            .iter()
            .map(|node| {
                (
                    node.id,
                    node.encounter.enemy_ship_ids.clone(),
                    node.encounter.hostile_count,
                )
            })
            .collect::<Vec<_>>();
        sector.ensure_latest_layout();
        let refreshed_enemy_ship_nodes = sector
            .nodes
            .iter()
            .filter_map(|node| {
                previous_enemy_ship_ids
                    .iter()
                    .find(|(id, _, _)| *id == node.id)
                    .and_then(|(_, previous_ids, previous_hostile_count)| {
                        if *previous_ids != node.encounter.enemy_ship_ids
                            || *previous_hostile_count != node.encounter.hostile_count
                        {
                            Some((
                                node.id,
                                previous_ids.clone(),
                                node.encounter.enemy_ship_ids.clone(),
                                *previous_hostile_count,
                                node.encounter.hostile_count,
                            ))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();
        if !refreshed_enemy_ship_nodes.is_empty() {
            log::warn!(
                "Campaign sector data differed from current sector layout; refreshed {} node(s) from sector_layout.json before session bootstrap",
                refreshed_enemy_ship_nodes.len()
            );
            for (
                node_id,
                previous_ids,
                refreshed_ids,
                previous_hostile_count,
                refreshed_hostile_count,
            ) in &refreshed_enemy_ship_nodes
            {
                log::debug!(
                    "Refreshed campaign sector node {} hostile config: enemy_ship_ids {:?} -> {:?}, hostile_count {} -> {}",
                    node_id,
                    previous_ids,
                    refreshed_ids,
                    previous_hostile_count,
                    refreshed_hostile_count
                );
            }
        }
        state.progression = progression;
        state.sector = sector;
        state.last_mission_report = last_mission_report;
    }

    state.phase = RollbackPhase::Docked;
    log::debug!(
        "Loaded initial rollback state: phase={:?}, ship='{}', modules={}, current_node={}, selected_node={:?}, active_node={:?}",
        state.phase,
        state.editor_ship.name,
        state.editor_ship.modules.len(),
        state.sector.current_node_id,
        state.sector.selected_node_id,
        state.sector.active_encounter_node_id
    );
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netcode::{INPUT_FIRE, INPUT_SPACE_EDGE, INPUT_UP, PlayerGgrsInput};

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
