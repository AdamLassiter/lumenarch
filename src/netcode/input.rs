use bevy::{log, platform::collections::HashMap, prelude::*, window::PrimaryWindow};
use bevy_ggrs::{LocalInputs, LocalPlayers, PlayerInputs};
use ggrs::PlayerHandle;

use super::{
    DecodedPlayerCommand,
    DecodedPlayerCommands,
    INPUT_AUX_EDGE,
    INPUT_CYCLE_TEMPLATE,
    INPUT_DOWN,
    INPUT_DROP,
    INPUT_EXIT_STATION,
    INPUT_FIRE,
    INPUT_INTERACT,
    INPUT_LEFT,
    INPUT_NEXT_EDGE,
    INPUT_PICKUP,
    INPUT_PREV_EDGE,
    INPUT_RIGHT,
    INPUT_SPACE_EDGE,
    INPUT_TOGGLE_DEBUG,
    INPUT_TOGGLE_STATION,
    INPUT_UP,
    LocalPlayerHandle,
    LumenGgrsConfig,
    PendingLocalMetaCommand,
    PendingLocalStationCommand,
    PendingMetaCommand,
    PendingStationCommand,
    PlayerGgrsInput,
    RollbackGameState,
    RollbackMetaOp,
    RollbackPhase,
    StationControlOp,
};
use crate::{
    state::SectorNodeKind,
    stations::{self, StationCatalogResource},
};

pub(crate) fn read_local_inputs(
    mut commands: Commands,
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
    mut pending_station: ResMut<PendingLocalStationCommand>,
    local_players: Res<LocalPlayers>,
) {
    let mut inputs = HashMap::default();
    let pending_meta_command = pending_meta.0.take();
    let pending_station_command = pending_station.0.take();
    let empty_keys = ButtonInput::<KeyCode>::default();
    let empty_mouse = ButtonInput::<MouseButton>::default();
    let keyboard_input = keyboard_input.as_deref().unwrap_or(&empty_keys);
    let mouse_buttons = mouse_buttons.as_deref().unwrap_or(&empty_mouse);
    let input = input_from_hardware(
        keyboard_input,
        mouse_buttons,
        window_query.iter().next(),
        pending_station_command,
        pending_meta_command,
    );
    if let Some(meta) = pending_meta_command {
        log::debug!(
            "Queued local meta command into input packet: op={:?}, args=({}, {}, {})",
            meta.op,
            meta.arg0,
            meta.arg1,
            meta.arg2
        );
    }
    if let Some(station) = pending_station_command {
        log::debug!(
            "Queued local station command into input packet: op={:?}, arg0={}",
            station.op,
            station.arg0
        );
    }
    for handle in &local_players.0 {
        inputs.insert(*handle, input);
    }
    log::trace!(
        "Publishing local inputs for handles {:?}: buttons=0x{:04x}, throttle={}, turn={}, aim=({}, {}), reactor_delta={}, turbine_delta={}, logistics_delta={}, station={:?}, meta={:?}",
        local_players.0,
        input.buttons,
        input.throttle_milli,
        input.turn_milli,
        input.aim_x_milli,
        input.aim_y_milli,
        input.reactor_delta_milli,
        input.turbine_delta_milli,
        input.logistics_delta,
        input.station_op(),
        input.meta_op()
    );
    commands.insert_resource(LocalInputs::<LumenGgrsConfig>(inputs));
}

pub(crate) fn decode_player_inputs(
    player_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    mut decoded: ResMut<DecodedPlayerCommands>,
) {
    decoded.by_handle.clear();
    let Some(player_inputs) = player_inputs else {
        log::trace!("PlayerInputs resource missing in decode_player_inputs");
        return;
    };
    for (handle, (input, _)) in player_inputs.iter().enumerate() {
        let command = decode_player_command(*input);
        if command.meta.op != RollbackMetaOp::None
            || command.station.op != StationControlOp::None
            || input.buttons != 0
        {
            log::trace!(
                "Decoded player input for handle {}: buttons=0x{:04x}, station={:?}, meta={:?}, move=({}, {}), throttle={}, turn={}, aim=({}, {})",
                handle,
                input.buttons,
                command.station.op,
                command.meta.op,
                command.move_x,
                command.move_y,
                command.throttle_milli,
                command.turn_milli,
                command.aim_x_milli,
                command.aim_y_milli
            );
        }
        decoded.by_handle.insert(handle, command);
    }
    if let Some(host_command) = decoded.by_handle.get(&0)
        && host_command.meta.op != RollbackMetaOp::None
    {
        log::debug!(
            "Host handle 0 decoded meta command: {:?} ({}, {}, {})",
            host_command.meta.op,
            host_command.meta.arg0,
            host_command.meta.arg1,
            host_command.meta.arg2
        );
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

pub(crate) fn rollback_phase_is_encounter(rollback_state: Res<RollbackGameState>) -> bool {
    rollback_state.phase == RollbackPhase::Encounter
}

pub(crate) fn apply_host_meta_ops(
    decoded: Res<DecodedPlayerCommands>,
    mut rollback_state: ResMut<RollbackGameState>,
    stations: Res<StationCatalogResource>,
) {
    let Some(command) = decoded.by_handle.get(&0) else {
        log::trace!("No input from host player handle 0, skipping meta ops");
        return;
    };

    match command.meta.op {
        RollbackMetaOp::None => {}
        RollbackMetaOp::OpenEditor => {
            log::info!("Applying host meta op: OpenEditor");
            rollback_state.phase = RollbackPhase::Editing;
        }
        RollbackMetaOp::OpenSectorMap => {
            log::info!("Applying host meta op: OpenSectorMap");
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
                log::info!(
                    "Applying host meta op: RepairShip for {} scrap; hull wear cleared",
                    repair_cost
                );
            } else {
                log::debug!(
                    "Ignoring RepairShip meta op because repair_cost={} and scrap={}",
                    repair_cost,
                    rollback_state.progression.scrap
                );
            }
        }
        RollbackMetaOp::SelectSectorNode => {
            let node_id = command.meta.arg0.max(0) as u32;
            if rollback_state.sector.is_reachable(node_id)
                && rollback_state
                    .sector
                    .node(node_id)
                    .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
                    .unwrap_or(false)
            {
                rollback_state.sector.selected_node_id = Some(node_id);
                log::info!("Applying host meta op: SelectSectorNode({})", node_id);
            } else {
                log::debug!(
                    "Ignoring SelectSectorNode({}) because the node is not currently reachable or launchable",
                    node_id
                );
            }
        }
        RollbackMetaOp::LaunchEncounter => {
            let node_id = command.meta.arg0.max(0) as u32;
            if rollback_state.sector.is_reachable(node_id)
                && rollback_state
                    .sector
                    .node(node_id)
                    .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
                    .unwrap_or(false)
            {
                rollback_state.sector.selected_node_id = Some(node_id);
                rollback_state.sector.active_encounter_node_id = Some(node_id);
                rollback_state.progression.jump_count += 1;
                rollback_state.phase = RollbackPhase::Encounter;
                rollback_state.scene_generation += 1;
                log::info!(
                    "Applying host meta op: LaunchEncounter(node_id={}) -> scene_generation={}",
                    node_id,
                    rollback_state.scene_generation
                );
            } else {
                log::debug!(
                    "Ignoring LaunchEncounter({}) because the node is not currently reachable or launchable",
                    node_id
                );
            }
        }
        RollbackMetaOp::ReturnToDock => {
            log::info!("Applying host meta op: ReturnToDock");
            rollback_state.phase = RollbackPhase::Docked;
        }
        RollbackMetaOp::LeaveEditor => {
            log::info!("Applying host meta op: LeaveEditor");
            rollback_state.phase = RollbackPhase::Docked;
        }
        RollbackMetaOp::AcceptContract => {
            let Some(station_id) = stations::current_station_id(&rollback_state.sector) else {
                log::debug!("Ignoring AcceptContract because no current station is active");
                return;
            };
            let contract_index = command.meta.arg0.max(0) as usize;
            let Some((station, contract)) =
                stations.0.contract_by_index(station_id, contract_index)
            else {
                log::debug!(
                    "Ignoring AcceptContract because contract index {} is invalid for station {}",
                    contract_index,
                    station_id
                );
                return;
            };
            rollback_state.progression.active_contract_id = Some(contract.id.clone());
            rollback_state
                .progression
                .unlock_station(station.id.clone());
            rollback_state
                .progression
                .unlock_contact(contract.contact_id.clone());
            rollback_state.sector.selected_node_id = Some(contract.target_node_id);
            rollback_state.last_mission_report.headline =
                Some(format!("Contract Accepted: {}", contract.title));
            rollback_state.last_mission_report.detail =
                Some(format!("{}\n{}", contract.briefing, contract.launch_blurb));
            log::info!(
                "Applying host meta op: AcceptContract('{}' -> node {})",
                contract.id,
                contract.target_node_id
            );
        }
        RollbackMetaOp::LaunchContract => {
            let Some(active_contract_id) = rollback_state.progression.active_contract_id.clone()
            else {
                log::debug!("Ignoring LaunchContract because there is no active contract");
                return;
            };
            let Some((station, contract)) = stations.0.contract(&active_contract_id) else {
                log::warn!(
                    "Active contract '{}' could not be resolved in station catalog",
                    active_contract_id
                );
                rollback_state.progression.active_contract_id = None;
                return;
            };
            let node_id = contract.target_node_id;
            if rollback_state.sector.is_reachable(node_id)
                && rollback_state
                    .sector
                    .node(node_id)
                    .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
                    .unwrap_or(false)
            {
                rollback_state
                    .progression
                    .unlock_station(station.id.clone());
                rollback_state
                    .progression
                    .unlock_contact(contract.contact_id.clone());
                rollback_state.sector.selected_node_id = Some(node_id);
                rollback_state.sector.active_encounter_node_id = Some(node_id);
                rollback_state.progression.jump_count += 1;
                rollback_state.phase = RollbackPhase::Encounter;
                rollback_state.scene_generation += 1;
                rollback_state.last_mission_report.headline =
                    Some(format!("Launch: {}", contract.title));
                rollback_state.last_mission_report.detail = Some(contract.launch_blurb.clone());
                log::info!(
                    "Applying host meta op: LaunchContract('{}') -> node {} / scene_generation={}",
                    contract.id,
                    node_id,
                    rollback_state.scene_generation
                );
            } else {
                log::debug!(
                    "Ignoring LaunchContract('{}') because node {} is not reachable/launchable",
                    contract.id,
                    node_id
                );
            }
        }
    }
}

pub(crate) fn command_for_handle(
    decoded: &DecodedPlayerCommands,
    handle: PlayerHandle,
) -> DecodedPlayerCommand {
    decoded.by_handle.get(&handle).copied().unwrap_or_default()
}

fn input_from_hardware(
    keys: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
    window: Option<&Window>,
    pending_station: Option<PendingStationCommand>,
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
    if let Some(window) = window
        && let Some(cursor) = window.cursor_position()
    {
        let x = ((cursor.x / window.width()) * 2.0 - 1.0).clamp(-1.0, 1.0);
        let y = ((cursor.y / window.height()) * 2.0 - 1.0).clamp(-1.0, 1.0);
        input.aim_x_milli = (x * 1000.0) as i16;
        input.aim_y_milli = (-y * 1000.0) as i16;
    }

    if let Some(station) = pending_station {
        input.station_op = station.op as u8;
        input.station_arg0 = station.arg0;
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
        station: PendingStationCommand {
            op: input.station_op(),
            arg0: input.station_arg0,
        },
        meta: PendingMetaCommand {
            op: input.meta_op(),
            arg0: input.meta_arg0,
            arg1: input.meta_arg1,
            arg2: input.meta_arg2,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_station_command_from_ggrs_input() {
        let input = PlayerGgrsInput {
            station_op: StationControlOp::ReactorAdjustRate as u8,
            station_arg0: 100,
            ..Default::default()
        };

        let decoded = decode_player_command(input);

        assert_eq!(decoded.station.op, StationControlOp::ReactorAdjustRate);
        assert_eq!(decoded.station.arg0, 100);
    }
}
