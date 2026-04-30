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
        AirlockCommandState,
        CarriedResource,
        CurrentStation,
        ManipulatorCommandState,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerShip,
        ProcessorCommandState,
        ProcessorRecipe,
        ReactorCommandState,
        ResourceKind,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipControlMode,
        ShipControlState,
        ShipRoot,
        ShipboardControlState,
        StorageCommandState,
        TurretCommandState,
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
        MultiplayerAppliedInputState,
        MultiplayerDiagnosticsState,
        MultiplayerMovementIntentState,
        MultiplayerSessionState,
        MultiplayerSyncGuard,
        MultiplayerTickState,
        NetworkCommandSender,
        SectorState,
    },
};
use crate::{
    host::SNAPSHOT_INTERVAL,
    protocol::{
        ClientHello,
        ClientMessage,
        EncounterRegisterState,
        PlayerInputFrame,
        PlayerPresenceSnapshot,
        PresenceFrame,
        RegisterStateEntry,
        RegisterValue,
        ServerMessage,
        SessionAppState,
        SessionCommand,
        SessionControlMode,
        SessionSnapshot,
        StateHashReport,
    },
    session::canonical_session_hash,
    ship::{ModuleKind, arch::ArchProgram},
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
                multiplayer_session.last_local_committed_input = batch
                    .frames
                    .iter()
                    .find(|frame| Some(frame.player_id) == multiplayer_session.local_player_id)
                    .cloned();
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
        With<crate::gameplay::components::ShipboardPlayer>,
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };
    let Ok((motion, current_station, carried, control_state)) = player_query.get_single() else {
        return;
    };

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
    keys: Res<ButtonInput<KeyCode>>,
    command_sender: Res<NetworkCommandSender>,
    player_query: Query<
        (&ShipboardControlState, &CurrentStation, &CarriedResource),
        With<crate::gameplay::components::ShipboardPlayer>,
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };
    let Ok((station_state, current_station, _carried)) = player_query.get_single() else {
        return;
    };
    let desired_mode = desired_control_mode_from_keys(station_state, current_station, &keys);
    let desired_focus = desired_mode
        .map(|mode| {
            if mode == ShipControlMode::Interior {
                None
            } else {
                Some(current_station.module_id)
            }
        })
        .unwrap_or(station_state.focused_module_id);
    let control_mode = desired_mode.unwrap_or(station_state.mode);
    let release_focus = matches!(desired_mode, Some(ShipControlMode::Interior));

    let (move_x_milli, move_y_milli) = if control_mode == ShipControlMode::Interior {
        movement_axes_milli(&keys)
    } else {
        (0, 0)
    };

    let (throttle_milli, turn_milli) = if control_mode == ShipControlMode::Cockpit {
        cockpit_axes_milli(&keys)
    } else {
        (0, 0)
    };

    let fire_pressed = control_mode == ShipControlMode::Turret
        && (keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyF));
    let desired_turret_angle_milli = if control_mode == ShipControlMode::Turret {
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            1800
        } else if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            -1800
        } else {
            0
        }
    } else {
        0
    };

    let mut writes = Vec::new();
    set_or_push_write(
        &mut writes,
        "player.ctrl.mode",
        RegisterValue::Symbol(control_mode_name(control_mode).to_string()),
    );
    if release_focus {
        set_or_push_write(
            &mut writes,
            "player.ctrl.focus_module_id",
            RegisterValue::Int(0),
        );
    } else if let Some(module_id) = desired_focus {
        set_or_push_write(
            &mut writes,
            "player.ctrl.focus_module_id",
            RegisterValue::Int(module_id as i64),
        );
    }

    if move_x_milli != 0 {
        set_or_push_write(
            &mut writes,
            "player.move.x_milli",
            RegisterValue::Int(move_x_milli as i64),
        );
    }
    if move_y_milli != 0 {
        set_or_push_write(
            &mut writes,
            "player.move.y_milli",
            RegisterValue::Int(move_y_milli as i64),
        );
    }
    if throttle_milli != 0 {
        set_or_push_write(
            &mut writes,
            "ship.cmd.throttle_milli",
            RegisterValue::Int(throttle_milli as i64),
        );
    }
    if turn_milli != 0 {
        set_or_push_write(
            &mut writes,
            "ship.cmd.turn_milli",
            RegisterValue::Int(turn_milli as i64),
        );
    }
    if fire_pressed {
        set_or_push_write(&mut writes, "ship.cmd.fire", RegisterValue::Bool(true));
    }
    if desired_turret_angle_milli != 0
        && let Some(module_id) = desired_focus
    {
        set_or_push_write(
            &mut writes,
            &format!("ship.mod.{module_id}.turret.desired_angle_delta_milli"),
            RegisterValue::Int(desired_turret_angle_milli as i64),
        );
        if fire_pressed {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.turret.fire_intent"),
                RegisterValue::Bool(true),
            );
        }
    }
    let reactor_reaction_delta_milli = reactor_delta_milli(&keys, true, control_mode);
    let reactor_turbine_delta_milli = reactor_delta_milli(&keys, false, control_mode);
    if reactor_reaction_delta_milli != 0
        && let Some(module_id) = desired_focus
    {
        set_or_push_write(
            &mut writes,
            &format!("ship.mod.{module_id}.reactor.reaction_rate_delta_milli"),
            RegisterValue::Int(reactor_reaction_delta_milli as i64),
        );
    }
    if reactor_turbine_delta_milli != 0
        && let Some(module_id) = desired_focus
    {
        set_or_push_write(
            &mut writes,
            &format!("ship.mod.{module_id}.reactor.turbine_load_delta_milli"),
            RegisterValue::Int(reactor_turbine_delta_milli as i64),
        );
    }
    if control_mode == ShipControlMode::Logistics
        && let Some(module_id) = desired_focus
    {
        if current_station.kind == ModuleKind::Cargo && keys.just_pressed(KeyCode::Space) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.storage.toggle_intake"),
                RegisterValue::Bool(true),
            );
        }
        if current_station.kind == ModuleKind::Airlock && keys.just_pressed(KeyCode::Space) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.airlock.toggle_open"),
                RegisterValue::Bool(true),
            );
        }
        if current_station.kind == ModuleKind::Airlock && keys.just_pressed(KeyCode::KeyM) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.manip.toggle_manual"),
                RegisterValue::Bool(true),
            );
        }
        if current_station.kind == ModuleKind::Airlock && keys.just_pressed(KeyCode::KeyR) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.manip.cycle_resource"),
                RegisterValue::Int(1),
            );
        }
        if current_station.kind == ModuleKind::Airlock && keys.just_pressed(KeyCode::BracketLeft) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.manip.cycle_target"),
                RegisterValue::Int(-1),
            );
        }
        if current_station.kind == ModuleKind::Airlock && keys.just_pressed(KeyCode::BracketRight) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.manip.cycle_target"),
                RegisterValue::Int(1),
            );
        }
        if current_station.kind == ModuleKind::Processor && keys.just_pressed(KeyCode::Space) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.processor.toggle_enabled"),
                RegisterValue::Bool(true),
            );
        }
        if current_station.kind == ModuleKind::Processor && keys.just_pressed(KeyCode::KeyR) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.processor.cycle_recipe"),
                RegisterValue::Bool(true),
            );
        }
    }
    if control_mode == ShipControlMode::Computer
        && let Some(module_id) = desired_focus
    {
        if keys.just_pressed(KeyCode::Space) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.computer.toggle_enabled"),
                RegisterValue::Bool(true),
            );
        }
        if keys.just_pressed(KeyCode::KeyT) {
            set_or_push_write(
                &mut writes,
                &format!("ship.mod.{module_id}.computer.cycle_template"),
                RegisterValue::Bool(true),
            );
        }
    }

    let frame = PlayerInputFrame {
        player_id: local_player_id,
        tick: tick_state.current_tick.saturating_add(2),
        writes,
    };
    let _ = send_message(&command_sender, ClientMessage::InputFrame(frame));
}

pub(crate) fn apply_committed_multiplayer_inputs(
    status: Res<ConnectionStatus>,
    session_state: Res<MultiplayerSessionState>,
    mut applied_state: ResMut<MultiplayerAppliedInputState>,
    mut movement_intent: ResMut<MultiplayerMovementIntentState>,
    mut ship_query: Query<&mut ShipControlState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        &mut ShipboardControlState,
        With<crate::gameplay::components::ShipboardPlayer>,
    >,
    mut module_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            Option<&mut AirlockCommandState>,
            Option<&mut StorageCommandState>,
            Option<&mut ManipulatorCommandState>,
            Option<&mut ProcessorCommandState>,
            Option<&mut ReactorCommandState>,
            Option<&mut TurretCommandState>,
            Option<&mut RuntimeArchComputer>,
        ),
        Without<crate::gameplay::components::HostileShipModule>,
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    if session_state.session_tick == 0
        || applied_state.last_applied_tick == session_state.session_tick
    {
        return;
    }
    applied_state.last_applied_tick = session_state.session_tick;

    let Ok(mut ship_control) = ship_query.get_single_mut() else {
        return;
    };
    let Ok(mut local_control_state) = player_query.get_single_mut() else {
        return;
    };

    ship_control.throttle_demand = fx_from_milli(0);
    ship_control.turn_input = fx_from_milli(0);
    ship_control.fire_pressed = false;
    movement_intent.move_x_milli = 0;
    movement_intent.move_y_milli = 0;

    let mut modules: Vec<_> = module_query.iter_mut().collect();
    for (_, _, _, _, _, _, _, turret, _) in &mut modules {
        if let Some(turret) = turret {
            turret.fire_intent = false;
        }
    }

    for frame in &session_state.last_committed_inputs {
        if Some(frame.player_id) == session_state.local_player_id {
            apply_local_committed_player_state(&mut local_control_state, frame, modules.as_slice());
            movement_intent.move_x_milli =
                input_int(frame, "player.move.x_milli").unwrap_or(0) as i32;
            movement_intent.move_y_milli =
                input_int(frame, "player.move.y_milli").unwrap_or(0) as i32;
        }

        apply_shared_committed_player_state(&mut ship_control, frame, modules.as_mut_slice());
    }
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
    if !matches!(status.phase, ConnectionPhase::Connected)
        || !tick_state.current_tick.is_multiple_of(SNAPSHOT_INTERVAL)
    {
        return;
    }
    let Some(local_player_id) = session_state.local_player_id else {
        return;
    };

    let local_hash = canonical_session_hash(
        SessionAppState::from(app_state.get().clone()),
        &editor_ship.ship,
        &progression,
        &sector_state,
        &last_mission_report,
        &session_state.authoritative_registers,
    );
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

pub(crate) fn sync_encounter_registers_to_host(
    status: Res<ConnectionStatus>,
    session_state: Res<MultiplayerSessionState>,
    command_sender: Res<NetworkCommandSender>,
    ship_query: Query<&ShipControlState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Query<&ShipboardControlState, With<crate::gameplay::components::ShipboardPlayer>>,
    module_query: Query<
        (
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&StorageCommandState>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
        ),
        Without<crate::gameplay::components::HostileShipModule>,
    >,
) {
    if !matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let Ok(ship_control) = ship_query.get_single() else {
        return;
    };
    let Ok(shipboard_control) = player_query.get_single() else {
        return;
    };

    let registers = build_encounter_register_state(ship_control, shipboard_control, &module_query);
    if registers == session_state.authoritative_registers {
        return;
    }

    let _ = send_message(
        &command_sender,
        ClientMessage::Command(SessionCommand::UpdateEncounterRegisters(registers)),
    );
}

pub(crate) fn apply_authoritative_encounter_registers(
    session_state: Res<MultiplayerSessionState>,
    mut ship_query: Query<&mut ShipControlState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        &mut ShipboardControlState,
        With<crate::gameplay::components::ShipboardPlayer>,
    >,
    mut module_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            Option<&mut AirlockCommandState>,
            Option<&mut StorageCommandState>,
            Option<&mut ManipulatorCommandState>,
            Option<&mut ProcessorCommandState>,
            Option<&mut ReactorCommandState>,
            Option<&mut TurretCommandState>,
        ),
        Without<crate::gameplay::components::HostileShipModule>,
    >,
) {
    if !session_state.is_changed() {
        return;
    }
    let Ok(mut ship_control) = ship_query.get_single_mut() else {
        return;
    };
    let Ok(mut shipboard_control) = player_query.get_single_mut() else {
        return;
    };

    let mut modules: Vec<_> = module_query.iter_mut().collect();

    apply_register_state(
        &session_state.authoritative_registers,
        &mut ship_control,
        &mut shipboard_control,
        modules.as_mut_slice(),
    );
}

fn build_encounter_register_state(
    ship_control: &ShipControlState,
    _shipboard_control: &ShipboardControlState,
    module_query: &Query<
        (
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&StorageCommandState>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
        ),
        Without<crate::gameplay::components::HostileShipModule>,
    >,
) -> EncounterRegisterState {
    let mut entries = vec![
        RegisterStateEntry {
            key: "ship.cmd.throttle_milli".to_string(),
            value: RegisterValue::Int(
                (ship_control.throttle_demand.to_num::<f32>() * 1000.0) as i64,
            ),
        },
        RegisterStateEntry {
            key: "ship.cmd.turn_milli".to_string(),
            value: RegisterValue::Int((ship_control.turn_input.to_num::<f32>() * 1000.0) as i64),
        },
        RegisterStateEntry {
            key: "ship.cmd.fire".to_string(),
            value: RegisterValue::Bool(ship_control.fire_pressed),
        },
    ];

    for (runtime_module, airlock, storage, manipulator, processor, reactor, turret) in
        module_query.iter()
    {
        let prefix = format!("ship.mod.{}", runtime_module.module_id);
        if let Some(state) = airlock {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.airlock.open"),
                value: RegisterValue::Bool(state.open),
            });
        }
        if let Some(state) = storage {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.storage.allow_intake"),
                value: RegisterValue::Bool(state.allow_intake),
            });
        }
        if let Some(state) = manipulator {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.manip.manual_mode"),
                value: RegisterValue::Bool(state.manual_mode),
            });
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.manip.transfer_enabled"),
                value: RegisterValue::Bool(state.transfer_enabled),
            });
            if let Some(source) = state.source_module_id {
                entries.push(RegisterStateEntry {
                    key: format!("{prefix}.manip.source_module_id"),
                    value: RegisterValue::Int(source as i64),
                });
            }
            if let Some(target) = state.target_module_id {
                entries.push(RegisterStateEntry {
                    key: format!("{prefix}.manip.target_module_id"),
                    value: RegisterValue::Int(target as i64),
                });
            }
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.manip.resource_kind"),
                value: RegisterValue::Symbol(resource_kind_name(state.resource_kind).to_string()),
            });
        }
        if let Some(state) = processor {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.processor.enabled"),
                value: RegisterValue::Bool(state.enabled),
            });
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.processor.recipe"),
                value: RegisterValue::Symbol(
                    processor_recipe_name(state.selected_recipe).to_string(),
                ),
            });
        }
        if let Some(state) = reactor {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.reactor.reaction_rate_milli"),
                value: RegisterValue::Int((state.reaction_rate.to_num::<f32>() * 1000.0) as i64),
            });
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.reactor.turbine_load_milli"),
                value: RegisterValue::Int((state.turbine_load.to_num::<f32>() * 1000.0) as i64),
            });
        }
        if let Some(state) = turret {
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.turret.desired_angle_milli"),
                value: RegisterValue::Int((state.desired_angle.to_num::<f32>() * 1000.0) as i64),
            });
            entries.push(RegisterStateEntry {
                key: format!("{prefix}.turret.fire_intent"),
                value: RegisterValue::Bool(state.fire_intent),
            });
        }
    }

    entries.sort_by(|a, b| a.key.cmp(&b.key));
    EncounterRegisterState { entries }
}

fn apply_register_state(
    registers: &EncounterRegisterState,
    ship_control: &mut ShipControlState,
    _shipboard_control: &mut ShipboardControlState,
    modules: &mut [(
        Entity,
        &RuntimeShipModule,
        Option<Mut<AirlockCommandState>>,
        Option<Mut<StorageCommandState>>,
        Option<Mut<ManipulatorCommandState>>,
        Option<Mut<ProcessorCommandState>>,
        Option<Mut<ReactorCommandState>>,
        Option<Mut<TurretCommandState>>,
    )],
) {
    for entry in &registers.entries {
        match entry.key.as_str() {
            "ship.cmd.throttle_milli" => {
                if let RegisterValue::Int(value) = entry.value {
                    ship_control.throttle_demand =
                        fixed::FixedI32::<fixed::types::extra::U16>::from_num(
                            value as f32 / 1000.0,
                        );
                }
            }
            "ship.cmd.turn_milli" => {
                if let RegisterValue::Int(value) = entry.value {
                    ship_control.turn_input = fixed::FixedI32::<fixed::types::extra::U16>::from_num(
                        value as f32 / 1000.0,
                    );
                }
            }
            "ship.cmd.fire" => {
                if let RegisterValue::Bool(value) = entry.value {
                    ship_control.fire_pressed = value;
                }
            }
            _ => {
                apply_module_register_entry(entry, modules);
            }
        }
    }
}

fn apply_module_register_entry(
    entry: &RegisterStateEntry,
    modules: &mut [(
        Entity,
        &RuntimeShipModule,
        Option<Mut<AirlockCommandState>>,
        Option<Mut<StorageCommandState>>,
        Option<Mut<ManipulatorCommandState>>,
        Option<Mut<ProcessorCommandState>>,
        Option<Mut<ReactorCommandState>>,
        Option<Mut<TurretCommandState>>,
    )],
) {
    let Some(rest) = entry.key.strip_prefix("ship.mod.") else {
        return;
    };
    let Some((module_id_str, suffix)) = rest.split_once('.') else {
        return;
    };
    let Ok(module_id) = module_id_str.parse::<u64>() else {
        return;
    };
    let Some((_, _, airlock, storage, manipulator, processor, reactor, turret)) = modules
        .iter_mut()
        .find(|(_, module, ..)| module.module_id == module_id)
    else {
        return;
    };

    match suffix {
        "airlock.open" => {
            if let (Some(state), RegisterValue::Bool(value)) = (airlock.as_mut(), &entry.value) {
                state.open = *value;
            }
        }
        "storage.allow_intake" => {
            if let (Some(state), RegisterValue::Bool(value)) = (storage.as_mut(), &entry.value) {
                state.allow_intake = *value;
            }
        }
        "manip.manual_mode" => {
            if let (Some(state), RegisterValue::Bool(value)) = (manipulator.as_mut(), &entry.value)
            {
                state.manual_mode = *value;
            }
        }
        "manip.transfer_enabled" => {
            if let (Some(state), RegisterValue::Bool(value)) = (manipulator.as_mut(), &entry.value)
            {
                state.transfer_enabled = *value;
            }
        }
        "manip.source_module_id" => {
            if let (Some(state), RegisterValue::Int(value)) = (manipulator.as_mut(), &entry.value) {
                state.source_module_id = Some(*value as u64);
            }
        }
        "manip.target_module_id" => {
            if let (Some(state), RegisterValue::Int(value)) = (manipulator.as_mut(), &entry.value) {
                state.target_module_id = Some(*value as u64);
            }
        }
        "manip.resource_kind" => {
            if let (Some(state), RegisterValue::Symbol(value)) =
                (manipulator.as_mut(), &entry.value)
                && let Some(kind) = parse_resource_kind(value)
            {
                state.resource_kind = kind;
            }
        }
        "processor.enabled" => {
            if let (Some(state), RegisterValue::Bool(value)) = (processor.as_mut(), &entry.value) {
                state.enabled = *value;
            }
        }
        "processor.recipe" => {
            if let (Some(state), RegisterValue::Symbol(value)) = (processor.as_mut(), &entry.value)
                && let Some(recipe) = parse_processor_recipe(value)
            {
                state.selected_recipe = recipe;
            }
        }
        "reactor.reaction_rate_milli" => {
            if let (Some(state), RegisterValue::Int(value)) = (reactor.as_mut(), &entry.value) {
                state.reaction_rate =
                    fixed::FixedI32::<fixed::types::extra::U16>::from_num(*value as f32 / 1000.0);
            }
        }
        "reactor.turbine_load_milli" => {
            if let (Some(state), RegisterValue::Int(value)) = (reactor.as_mut(), &entry.value) {
                state.turbine_load =
                    fixed::FixedI32::<fixed::types::extra::U16>::from_num(*value as f32 / 1000.0);
            }
        }
        "turret.desired_angle_milli" => {
            if let (Some(state), RegisterValue::Int(value)) = (turret.as_mut(), &entry.value) {
                state.desired_angle =
                    fixed::FixedI32::<fixed::types::extra::U16>::from_num(*value as f32 / 1000.0);
            }
        }
        "turret.fire_intent" => {
            if let (Some(state), RegisterValue::Bool(value)) = (turret.as_mut(), &entry.value) {
                state.fire_intent = *value;
            }
        }
        _ => {}
    }
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::RawSalvage => "raw_salvage",
        ResourceKind::RepairCharge => "repair_charge",
        ResourceKind::Fuel => "fuel",
        ResourceKind::Ammunition => "ammunition",
    }
}

fn parse_resource_kind(value: &str) -> Option<ResourceKind> {
    match value {
        "raw_salvage" => Some(ResourceKind::RawSalvage),
        "repair_charge" => Some(ResourceKind::RepairCharge),
        "fuel" => Some(ResourceKind::Fuel),
        "ammunition" => Some(ResourceKind::Ammunition),
        _ => None,
    }
}

fn processor_recipe_name(recipe: ProcessorRecipe) -> &'static str {
    match recipe {
        ProcessorRecipe::RepairCharge => "repair_charge",
        ProcessorRecipe::Ammunition => "ammunition",
        ProcessorRecipe::Fuel => "fuel",
    }
}

fn parse_processor_recipe(value: &str) -> Option<ProcessorRecipe> {
    match value {
        "repair_charge" => Some(ProcessorRecipe::RepairCharge),
        "ammunition" => Some(ProcessorRecipe::Ammunition),
        "fuel" => Some(ProcessorRecipe::Fuel),
        _ => None,
    }
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
    multiplayer_session.authoritative_registers = snapshot.encounter_registers.clone();
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

fn desired_control_mode_from_keys(
    control_state: &ShipboardControlState,
    current_station: &CurrentStation,
    keys: &ButtonInput<KeyCode>,
) -> Option<ShipControlMode> {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        return Some(ShipControlMode::Interior);
    }
    if !keys.just_pressed(KeyCode::KeyE) || control_state.mode != ShipControlMode::Interior {
        return None;
    }
    match current_station.kind {
        ModuleKind::Cockpit => Some(ShipControlMode::Cockpit),
        ModuleKind::Turret => Some(ShipControlMode::Turret),
        ModuleKind::Reactor => Some(ShipControlMode::Reactor),
        ModuleKind::Cargo | ModuleKind::Processor | ModuleKind::Airlock => {
            Some(ShipControlMode::Logistics)
        }
        ModuleKind::Computer => Some(ShipControlMode::Computer),
        _ => None,
    }
}

fn movement_axes_milli(keys: &ButtonInput<KeyCode>) -> (i32, i32) {
    let mut x = 0;
    let mut y = 0;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        y += 1000;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        y -= 1000;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        x -= 1000;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        x += 1000;
    }
    (x, y)
}

fn cockpit_axes_milli(keys: &ButtonInput<KeyCode>) -> (i32, i32) {
    let mut throttle = 0;
    let mut turn = 0;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        throttle += 1000;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        throttle -= 1000;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        turn += 1000;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        turn -= 1000;
    }
    (throttle.clamp(0, 1000), turn.clamp(-1000, 1000))
}

fn reactor_delta_milli(
    keys: &ButtonInput<KeyCode>,
    reaction_rate: bool,
    mode: ShipControlMode,
) -> i32 {
    if mode != ShipControlMode::Reactor {
        return 0;
    }
    let positive = if reaction_rate {
        keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)
    } else {
        keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)
    };
    let negative = if reaction_rate {
        keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)
    } else {
        keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)
    };
    match (positive, negative) {
        (true, false) => 900,
        (false, true) => -900,
        _ => 0,
    }
}

fn fx_from_milli(value: i32) -> fixed::FixedI32<fixed::types::extra::U16> {
    fixed::FixedI32::<fixed::types::extra::U16>::from_num(value as f32 / 1000.0)
}

fn apply_local_committed_player_state(
    control_state: &mut ShipboardControlState,
    frame: &PlayerInputFrame,
    modules: &[(
        Entity,
        &RuntimeShipModule,
        Option<Mut<AirlockCommandState>>,
        Option<Mut<StorageCommandState>>,
        Option<Mut<ManipulatorCommandState>>,
        Option<Mut<ProcessorCommandState>>,
        Option<Mut<ReactorCommandState>>,
        Option<Mut<TurretCommandState>>,
        Option<Mut<RuntimeArchComputer>>,
    )],
) {
    let mode = input_control_mode(frame).unwrap_or(ShipControlMode::Interior);
    let focused_module_id = input_focus_module_id(frame);
    control_state.mode = mode;
    control_state.focus_mode = if focused_module_id.is_some() && mode != ShipControlMode::Interior {
        crate::gameplay::components::StationFocusMode::Focused
    } else {
        crate::gameplay::components::StationFocusMode::Internal
    };
    control_state.focused_module_id = focused_module_id;
    control_state.focused_entity = None;
    control_state.focused_kind = None;
    control_state.focused_family = None;

    if let Some(module_id) = focused_module_id
        && let Some((entity, runtime_module, ..)) = modules
            .iter()
            .find(|(_, runtime_module, ..)| runtime_module.module_id == module_id)
    {
        control_state.focused_entity = Some(*entity);
        control_state.focused_kind = Some(runtime_module.kind);
        control_state.focused_family = Some(station_family_for_kind(runtime_module.kind));
    }
}

fn apply_shared_committed_player_state(
    ship_control: &mut ShipControlState,
    frame: &PlayerInputFrame,
    modules: &mut [(
        Entity,
        &RuntimeShipModule,
        Option<Mut<AirlockCommandState>>,
        Option<Mut<StorageCommandState>>,
        Option<Mut<ManipulatorCommandState>>,
        Option<Mut<ProcessorCommandState>>,
        Option<Mut<ReactorCommandState>>,
        Option<Mut<TurretCommandState>>,
        Option<Mut<RuntimeArchComputer>>,
    )],
) {
    match input_control_mode(frame).unwrap_or(ShipControlMode::Interior) {
        ShipControlMode::Cockpit => {
            ship_control.throttle_demand =
                fx_from_milli(input_int(frame, "ship.cmd.throttle_milli").unwrap_or(0) as i32);
            ship_control.turn_input =
                fx_from_milli(input_int(frame, "ship.cmd.turn_milli").unwrap_or(0) as i32);
        }
        ShipControlMode::Turret => {
            if let Some(module_id) = input_focus_module_id(frame)
                && let Some((_, _, _, _, _, _, _, Some(turret), _)) = modules
                    .iter_mut()
                    .find(|(_, runtime_module, ..)| runtime_module.module_id == module_id)
            {
                let delta_key = format!("ship.mod.{module_id}.turret.desired_angle_delta_milli");
                turret.desired_angle +=
                    fx_from_milli(input_int(frame, &delta_key).unwrap_or(0) as i32) / 10;
                let fire_key = format!("ship.mod.{module_id}.turret.fire_intent");
                let fire_intent = input_bool(frame, &fire_key).unwrap_or(false)
                    || input_bool(frame, "ship.cmd.fire").unwrap_or(false);
                turret.fire_intent = fire_intent;
                ship_control.fire_pressed |= fire_intent;
            }
        }
        ShipControlMode::Reactor => {
            if let Some(module_id) = input_focus_module_id(frame)
                && let Some((_, _, _, _, _, _, Some(reactor), _, _)) = modules
                    .iter_mut()
                    .find(|(_, runtime_module, ..)| runtime_module.module_id == module_id)
            {
                let reaction_key =
                    format!("ship.mod.{module_id}.reactor.reaction_rate_delta_milli");
                let turbine_key = format!("ship.mod.{module_id}.reactor.turbine_load_delta_milli");
                reactor.reaction_rate = (reactor.reaction_rate
                    + fx_from_milli(input_int(frame, &reaction_key).unwrap_or(0) as i32) / 10)
                    .clamp(fx_from_milli(0), fx_from_milli(1000));
                reactor.turbine_load = (reactor.turbine_load
                    + fx_from_milli(input_int(frame, &turbine_key).unwrap_or(0) as i32) / 10)
                    .clamp(fx_from_milli(0), fx_from_milli(1000));
            }
        }
        ShipControlMode::Logistics => {
            let candidate_target_ids = if let Some(module_id) = input_focus_module_id(frame) {
                candidate_target_module_ids(module_id, modules)
            } else {
                Vec::new()
            };
            if let Some(module_id) = input_focus_module_id(frame)
                && let Some((_, runtime_module, airlock, storage, manipulator, processor, _, _, _)) =
                    modules
                        .iter_mut()
                        .find(|(_, runtime_module, ..)| runtime_module.module_id == module_id)
            {
                if input_bool(frame, &format!("ship.mod.{module_id}.airlock.toggle_open"))
                    .unwrap_or(false)
                    && runtime_module.kind == ModuleKind::Airlock
                    && let Some(airlock) = airlock
                {
                    airlock.open = !airlock.open;
                }
                if input_bool(
                    frame,
                    &format!("ship.mod.{module_id}.storage.toggle_intake"),
                )
                .unwrap_or(false)
                    && runtime_module.kind == ModuleKind::Cargo
                    && let Some(storage) = storage
                {
                    storage.allow_intake = !storage.allow_intake;
                }
                if let Some(manipulator) = manipulator {
                    if input_bool(frame, &format!("ship.mod.{module_id}.manip.toggle_manual"))
                        .unwrap_or(false)
                    {
                        manipulator.manual_mode = !manipulator.manual_mode;
                    }
                    if input_int(frame, &format!("ship.mod.{module_id}.manip.cycle_resource"))
                        .unwrap_or(0)
                        != 0
                    {
                        manipulator.resource_kind = next_resource_kind(manipulator.resource_kind);
                    }
                    let cycle_target =
                        input_int(frame, &format!("ship.mod.{module_id}.manip.cycle_target"))
                            .unwrap_or(0) as i32;
                    if cycle_target != 0 {
                        manipulator.target_module_id = cycle_target_in_ids(
                            manipulator.target_module_id,
                            cycle_target,
                            &candidate_target_ids,
                        );
                        manipulator.source_module_id = Some(runtime_module.module_id);
                    }
                }
                if let Some(processor) = processor {
                    if input_bool(
                        frame,
                        &format!("ship.mod.{module_id}.processor.toggle_enabled"),
                    )
                    .unwrap_or(false)
                    {
                        processor.enabled = !processor.enabled;
                    }
                    if input_bool(
                        frame,
                        &format!("ship.mod.{module_id}.processor.cycle_recipe"),
                    )
                    .unwrap_or(false)
                    {
                        processor.selected_recipe =
                            next_processor_recipe(processor.selected_recipe);
                    }
                }
            }
        }
        ShipControlMode::Computer => {
            if let Some(module_id) = input_focus_module_id(frame)
                && let Some((_, _, _, _, _, _, _, _, Some(arch))) = modules
                    .iter_mut()
                    .find(|(_, runtime_module, ..)| runtime_module.module_id == module_id)
            {
                if input_bool(
                    frame,
                    &format!("ship.mod.{module_id}.computer.toggle_enabled"),
                )
                .unwrap_or(false)
                {
                    arch.enabled = !arch.enabled;
                }
                if input_bool(
                    frame,
                    &format!("ship.mod.{module_id}.computer.cycle_template"),
                )
                .unwrap_or(false)
                {
                    arch.program = ArchProgram::from_template(arch.program.template.next());
                }
            }
        }
        ShipControlMode::Interior => {}
    }
}

fn station_family_for_kind(kind: ModuleKind) -> crate::gameplay::components::StationFamily {
    match kind {
        ModuleKind::Cockpit => crate::gameplay::components::StationFamily::Cockpit,
        ModuleKind::Turret => crate::gameplay::components::StationFamily::Turret,
        ModuleKind::Reactor => crate::gameplay::components::StationFamily::Reactor,
        ModuleKind::Cargo => crate::gameplay::components::StationFamily::Storage,
        ModuleKind::Airlock => crate::gameplay::components::StationFamily::Manipulator,
        ModuleKind::Processor => crate::gameplay::components::StationFamily::Processor,
        ModuleKind::Computer => crate::gameplay::components::StationFamily::Computer,
        _ => crate::gameplay::components::StationFamily::Storage,
    }
}

fn session_to_control_mode(mode: SessionControlMode) -> ShipControlMode {
    match mode {
        SessionControlMode::Interior => ShipControlMode::Interior,
        SessionControlMode::Cockpit => ShipControlMode::Cockpit,
        SessionControlMode::Turret => ShipControlMode::Turret,
        SessionControlMode::Reactor => ShipControlMode::Reactor,
        SessionControlMode::Logistics => ShipControlMode::Logistics,
        SessionControlMode::Computer => ShipControlMode::Computer,
    }
}

fn next_resource_kind(kind: ResourceKind) -> ResourceKind {
    match kind {
        ResourceKind::RawSalvage => ResourceKind::RepairCharge,
        ResourceKind::RepairCharge => ResourceKind::Fuel,
        ResourceKind::Fuel => ResourceKind::Ammunition,
        ResourceKind::Ammunition => ResourceKind::RawSalvage,
    }
}

fn next_processor_recipe(recipe: ProcessorRecipe) -> ProcessorRecipe {
    match recipe {
        ProcessorRecipe::RepairCharge => ProcessorRecipe::Ammunition,
        ProcessorRecipe::Ammunition => ProcessorRecipe::Fuel,
        ProcessorRecipe::Fuel => ProcessorRecipe::RepairCharge,
    }
}

fn candidate_target_module_ids(
    source_module_id: u64,
    modules: &[(
        Entity,
        &RuntimeShipModule,
        Option<Mut<AirlockCommandState>>,
        Option<Mut<StorageCommandState>>,
        Option<Mut<ManipulatorCommandState>>,
        Option<Mut<ProcessorCommandState>>,
        Option<Mut<ReactorCommandState>>,
        Option<Mut<TurretCommandState>>,
        Option<Mut<RuntimeArchComputer>>,
    )],
) -> Vec<u64> {
    let mut ids = modules
        .iter()
        .filter_map(|(_, runtime_module, ..)| {
            if runtime_module.module_id != source_module_id {
                Some(runtime_module.module_id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn cycle_target_in_ids(current_target: Option<u64>, direction: i32, ids: &[u64]) -> Option<u64> {
    if ids.is_empty() {
        return None;
    }
    let current_index = current_target
        .and_then(|module_id| ids.iter().position(|id| *id == module_id))
        .unwrap_or(0);
    let next_index = ((current_index as i32 + direction).rem_euclid(ids.len() as i32)) as usize;
    Some(ids[next_index])
}

fn set_or_push_write(writes: &mut Vec<RegisterStateEntry>, key: &str, value: RegisterValue) {
    if let Some(entry) = writes.iter_mut().find(|entry| entry.key == key) {
        entry.value = value;
    } else {
        writes.push(RegisterStateEntry {
            key: key.to_string(),
            value,
        });
    }
}

fn input_int(frame: &PlayerInputFrame, key: &str) -> Option<i64> {
    frame
        .writes
        .iter()
        .find_map(|entry| match (entry.key.as_str(), &entry.value) {
            (entry_key, RegisterValue::Int(value)) if entry_key == key => Some(*value),
            _ => None,
        })
}

fn input_bool(frame: &PlayerInputFrame, key: &str) -> Option<bool> {
    frame
        .writes
        .iter()
        .find_map(|entry| match (entry.key.as_str(), &entry.value) {
            (entry_key, RegisterValue::Bool(value)) if entry_key == key => Some(*value),
            _ => None,
        })
}

fn input_symbol<'a>(frame: &'a PlayerInputFrame, key: &str) -> Option<&'a str> {
    frame
        .writes
        .iter()
        .find_map(|entry| match (&entry.key[..], &entry.value) {
            (entry_key, RegisterValue::Symbol(value)) if entry_key == key => Some(value.as_str()),
            _ => None,
        })
}

fn input_focus_module_id(frame: &PlayerInputFrame) -> Option<u64> {
    match input_int(frame, "player.ctrl.focus_module_id") {
        Some(0) | None => None,
        Some(value) => Some(value as u64),
    }
}

fn input_control_mode(frame: &PlayerInputFrame) -> Option<ShipControlMode> {
    let value = input_symbol(frame, "player.ctrl.mode")?;
    Some(match value {
        "Interior" => ShipControlMode::Interior,
        "Cockpit" => ShipControlMode::Cockpit,
        "Turret" => ShipControlMode::Turret,
        "Reactor" => ShipControlMode::Reactor,
        "Logistics" => ShipControlMode::Logistics,
        "Computer" => ShipControlMode::Computer,
        _ => return None,
    })
}

fn control_mode_name(mode: ShipControlMode) -> &'static str {
    match mode {
        ShipControlMode::Interior => "Interior",
        ShipControlMode::Cockpit => "Cockpit",
        ShipControlMode::Turret => "Turret",
        ShipControlMode::Reactor => "Reactor",
        ShipControlMode::Logistics => "Logistics",
        ShipControlMode::Computer => "Computer",
    }
}

fn control_mode_to_session(
    mode: crate::gameplay::components::ShipControlMode,
) -> SessionControlMode {
    match mode {
        crate::gameplay::components::ShipControlMode::Interior => SessionControlMode::Interior,
        crate::gameplay::components::ShipControlMode::Cockpit => SessionControlMode::Cockpit,
        crate::gameplay::components::ShipControlMode::Turret => SessionControlMode::Turret,
        crate::gameplay::components::ShipControlMode::Reactor => SessionControlMode::Reactor,
        crate::gameplay::components::ShipControlMode::Logistics => SessionControlMode::Logistics,
        crate::gameplay::components::ShipControlMode::Computer => SessionControlMode::Computer,
    }
}
