use std::time::{Duration, Instant};

use bevy::{
    ecs::system::RunSystemOnce,
    prelude::*,
};
use bevy_ggrs::{
    Session,
    prelude::{PlayerType, SessionBuilder},
};

use crate::{
    AppRuntimeMode,
    build_app,
    gameplay::{
        self,
        components::{ObservedLocalPlayerMarker, ShipControlMode, ShipboardControlState},
    },
    netcode::{
        self,
        DecodedPlayerCommand,
        DecodedPlayerCommands,
        LobbyPlayerInfo,
        LobbySnapshot,
        PendingMetaCommand,
        RollbackMetaOp,
        RollbackPhase,
        SessionPhase,
        SessionRole,
        SessionStatus,
        INPUT_TOGGLE_STATION,
    },
    state::FrontendMode,
};

#[test]
fn headless_host_lobby_editor_sector_and_cockpit_flow() {
    let mut app = build_app(AppRuntimeMode::Headless);

    begin_headless_host_lobby(&mut app);
    assert!(matches!(
        app.world().resource::<SessionStatus>().phase,
        SessionPhase::Lobby
    ));

    start_headless_host_session(&mut app);
    assert!(matches!(
        app.world().resource::<SessionStatus>().phase,
        SessionPhase::Connected
    ));

    assert_eq!(
        app.world().resource::<State<FrontendMode>>().get(),
        &FrontendMode::Session
    );
    assert_eq!(
        app.world().resource::<netcode::RollbackGameState>().phase,
        RollbackPhase::Docked
    );

    apply_host_meta(&mut app, RollbackMetaOp::OpenEditor, 0, 0, 0);
    assert_eq!(
        app.world().resource::<netcode::RollbackGameState>().phase,
        RollbackPhase::Editing
    );

    apply_host_meta(&mut app, RollbackMetaOp::LeaveEditor, 0, 0, 0);
    assert_eq!(
        app.world().resource::<netcode::RollbackGameState>().phase,
        RollbackPhase::Docked
    );

    apply_host_meta(&mut app, RollbackMetaOp::OpenSectorMap, 0, 0, 0);
    assert_eq!(
        app.world().resource::<netcode::RollbackGameState>().phase,
        RollbackPhase::SectorMap
    );

    let calibration_ring_id = {
        let rollback = app.world().resource::<netcode::RollbackGameState>();
        rollback
            .sector
            .nodes
            .iter()
            .find(|node| node.label == "Calibration Ring")
            .map(|node| node.id)
            .expect("expected Calibration Ring node in sector layout")
    };

    apply_host_meta(
        &mut app,
        RollbackMetaOp::SelectSectorNode,
        calibration_ring_id as i16,
        0,
        0,
    );
    {
        let rollback = app.world().resource::<netcode::RollbackGameState>();
        assert_eq!(rollback.sector.selected_node_id, Some(calibration_ring_id));
    }

    apply_host_meta(
        &mut app,
        RollbackMetaOp::LaunchEncounter,
        calibration_ring_id as i16,
        0,
        0,
    );
    wait_until(
        &mut app,
        |app| app.world().resource::<netcode::RollbackGameState>().phase == RollbackPhase::Encounter,
        "encounter phase entered",
    );

    wait_until(
        &mut app,
        |app| {
            let mut query = app
                .world_mut()
                .query_filtered::<Entity, With<crate::state::PlayingCleanup>>();
            query.iter(app.world()).next().is_some()
        },
        "runtime encounter scene spawned",
    );

    drive_local_toggle_station(&mut app);
    {
        let mut query = app
            .world_mut()
            .query_filtered::<&ShipboardControlState, With<ObservedLocalPlayerMarker>>();
        let control = query
            .single(app.world())
            .expect("expected exactly one observed local player control state");
        assert_eq!(control.mode, ShipControlMode::Cockpit);
    }
}

fn begin_headless_host_lobby(app: &mut App) {
    let host_addr = crate::DEFAULT_HOST_ADDR
        .parse()
        .expect("default host addr should parse");
    let session_config = app.world().resource::<netcode::SessionConfig>().clone();
    let local_profile = app.world().resource::<crate::state::LocalPlayerProfile>().clone();
    let initial_state = netcode::load_initial_rollback_state();

    {
        let mut bootstrap = app.world_mut().resource_mut::<netcode::SessionBootstrapConfig>();
        bootstrap.pending_start = false;
        bootstrap.role = SessionRole::Host;
        bootstrap.local_bind_addr = host_addr;
        bootstrap.peer_addrs.clear();
        bootstrap.local_handle = 0;
        bootstrap.input_delay = session_config.input_delay;
        bootstrap.check_distance = session_config.check_distance;
        bootstrap.initial_state = initial_state.clone();
    }

    {
        let mut status = app.world_mut().resource_mut::<SessionStatus>();
        status.phase = SessionPhase::Lobby;
        status.role = Some(SessionRole::Host);
        status.total_players = 1;
        status.local_player_handles = vec![0];
        status.active_ship_snapshot = Some(initial_state.editor_ship.clone());
        status.lobby_snapshot = Some(LobbySnapshot {
            host_addr,
            players: vec![LobbyPlayerInfo {
                handle: 0,
                bind_addr: host_addr,
                is_host: true,
                profile: local_profile,
            }],
        });
    }
}

fn start_headless_host_session(app: &mut App) {
    let bootstrap = app
        .world()
        .resource::<netcode::SessionBootstrapConfig>()
        .clone();
    let sync_test = SessionBuilder::<netcode::LumenGgrsConfig>::new()
        .with_num_players(1)
        .expect("synctest should accept one player")
        .with_max_prediction_window(bootstrap.check_distance + 2)
        .add_player(PlayerType::Local, 0)
        .expect("failed to register local synctest player")
        .with_input_delay(bootstrap.input_delay)
        .with_check_distance(bootstrap.check_distance)
        .start_synctest_session()
        .expect("failed to start synctest session");

    app.world_mut()
        .insert_resource(Session::SyncTest(sync_test));
    app.world_mut()
        .insert_resource(bevy_ggrs::LocalPlayers(vec![0]));

    {
        let mut status = app.world_mut().resource_mut::<SessionStatus>();
        status.phase = SessionPhase::Connected;
        status.role = Some(SessionRole::Host);
        status.total_players = 1;
        status.local_player_handles = vec![0];
        status.active_ship_snapshot = Some(bootstrap.initial_state.editor_ship.clone());
    }
    *app.world_mut().resource_mut::<netcode::RollbackGameState>() = bootstrap.initial_state;
    app.world_mut().resource_mut::<netcode::LocalPlayerHandle>().0 = Some(0);
    app.world_mut().resource_mut::<netcode::ObservedLocalPlayer>().handle = Some(0);
    app.world_mut()
        .resource_mut::<netcode::PendingLocalMetaCommand>()
        .0 = None;
    app.world_mut()
        .resource_mut::<netcode::PendingLocalStationCommand>()
        .0 = None;
    app.world_mut()
        .resource_mut::<DecodedPlayerCommands>()
        .by_handle
        .clear();
    app.world_mut()
        .resource_mut::<NextState<FrontendMode>>()
        .set(FrontendMode::Session);
    app.update();
}

fn apply_host_meta(app: &mut App, op: RollbackMetaOp, arg0: i16, arg1: i16, arg2: i16) {
    {
        let mut decoded = app.world_mut().resource_mut::<DecodedPlayerCommands>();
        decoded.by_handle.clear();
        decoded.by_handle.insert(
            0,
            DecodedPlayerCommand {
                meta: PendingMetaCommand {
                    op,
                    arg0,
                    arg1,
                    arg2,
                },
                ..Default::default()
            },
        );
    }
    app.world_mut()
        .run_system_once(netcode::apply_host_meta_ops)
        .expect("apply_host_meta_ops system should run");
    app.world_mut()
        .resource_mut::<DecodedPlayerCommands>()
        .by_handle
        .clear();
    app.update();
}

fn drive_local_toggle_station(app: &mut App) {
    app.world_mut()
        .run_system_once(gameplay::update_current_station)
        .expect("update_current_station system should run");
    {
        let mut decoded = app.world_mut().resource_mut::<DecodedPlayerCommands>();
        decoded.by_handle.clear();
        let mut raw = netcode::PlayerGgrsInput::default();
        raw.buttons = INPUT_TOGGLE_STATION;
        decoded.by_handle.insert(
            0,
            DecodedPlayerCommand {
                raw,
                ..Default::default()
            },
        );
    }
    app.world_mut()
        .run_system_once(gameplay::toggle_shipboard_control_mode)
        .expect("toggle_shipboard_control_mode system should run");
    app.world_mut()
        .resource_mut::<DecodedPlayerCommands>()
        .by_handle
        .clear();
}

fn pump_once(app: &mut App) {
    app.update();
}

fn wait_until(app: &mut App, predicate: impl Fn(&mut App) -> bool, description: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        pump_once(app);
        if predicate(app) {
            return;
        }
    }
    panic!("timed out waiting for {description}");
}
