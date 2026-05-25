use std::time::{Duration, Instant};

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_ggrs::{
    Session,
    prelude::{PlayerType, SessionBuilder},
};

use crate::{
    AppRuntimeMode,
    DEFAULT_HOST_ADDR,
    build_app,
    gameplay::{
        self,
        components::{
            CurrentStation,
            ObservedLocalPlayerMarker,
            PlayerFocusedTile,
            PlayerMotionState,
            ShipControlMode,
            ShipboardControlState,
            StationFamily,
            StationFocusMode,
        },
        helpers::FixedVec2,
    },
    netcode::{
        self,
        DecodedPlayerCommand,
        DecodedPlayerCommands,
        LobbyPlayerInfo,
        LobbySnapshot,
        LocalPlayerHandle,
        LumenGgrsConfig,
        ObservedLocalPlayer,
        PendingLocalMetaCommand,
        PendingLocalStationCommand,
        PendingMetaCommand,
        RollbackGameState,
        RollbackMetaOp,
        RollbackPhase,
        SessionBootstrapConfig,
        SessionConfig,
        SessionPhase,
        SessionRole,
        SessionStatus,
    },
    ship::{ModuleKind, ShipDefinition, ShipFoundationKind, ShipFoundationTile, ShipModule},
    state::{FrontendMode, LocalPlayerProfile, PlayingCleanup, SectorNodeKind},
};

#[test]
fn headless_host_lobby_editor_sector_and_cockpit_flow() {
    let mut app = build_app(AppRuntimeMode::Headless);
    seed_test_player_ship(&mut app);

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
        let rollback = app.world().resource::<RollbackGameState>();
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
        let rollback = app.world().resource::<RollbackGameState>();
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
        |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Encounter,
        "encounter phase entered",
    );

    wait_until(
        &mut app,
        |app| {
            let mut query = app
                .world_mut()
                .query_filtered::<Entity, With<PlayingCleanup>>();
            query.iter(app.world()).next().is_some()
        },
        "runtime encounter scene spawned",
    );

    enter_cockpit_for_flow_test(&mut app);
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

#[test]
fn entering_station_focus_does_not_reposition_player_body() {
    let mut app = build_app(AppRuntimeMode::Headless);
    seed_test_player_ship(&mut app);

    begin_headless_host_lobby(&mut app);
    start_headless_host_session(&mut app);
    apply_host_meta(&mut app, RollbackMetaOp::OpenSectorMap, 0, 0, 0);

    let calibration_ring_id = {
        let rollback = app.world().resource::<RollbackGameState>();
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
    apply_host_meta(
        &mut app,
        RollbackMetaOp::LaunchEncounter,
        calibration_ring_id as i16,
        0,
        0,
    );
    wait_until(
        &mut app,
        |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Encounter,
        "encounter phase entered",
    );
    wait_until(
        &mut app,
        |app| count_cleanup_entities(app) > 0,
        "runtime encounter scene spawned",
    );

    let before = observed_player_local_position(&mut app);
    enter_cockpit_for_flow_test(&mut app);
    pump_once(&mut app);
    let after = observed_player_local_position(&mut app);

    assert_eq!(after.x, before.x);
    assert_eq!(after.y, before.y);
    let mut control_query = app
        .world_mut()
        .query_filtered::<&ShipboardControlState, With<ObservedLocalPlayerMarker>>();
    let control = control_query
        .single(app.world())
        .expect("expected exactly one observed local player control state");
    assert_eq!(control.mode, ShipControlMode::Cockpit);
}

#[derive(Debug, PartialEq, Eq)]
struct StationFocusSnapshot {
    phase: RollbackPhase,
    control_mode: ShipControlMode,
    focus_mode: StationFocusMode,
    focused_kind: Option<ModuleKind>,
    current_station_kind: ModuleKind,
    focused_tile: Option<(i32, i32)>,
    local_tile: (i32, i32),
}

#[test]
fn repeated_station_focus_flow_produces_same_state_snapshot() {
    let first = station_focus_regression_snapshot();
    let second = station_focus_regression_snapshot();

    assert_eq!(first, second);
}

fn station_focus_regression_snapshot() -> StationFocusSnapshot {
    let mut app = build_app(AppRuntimeMode::Headless);
    seed_test_player_ship(&mut app);

    begin_headless_host_lobby(&mut app);
    start_headless_host_session(&mut app);
    apply_host_meta(&mut app, RollbackMetaOp::OpenSectorMap, 0, 0, 0);

    let calibration_ring_id = {
        let rollback = app.world().resource::<RollbackGameState>();
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
    apply_host_meta(
        &mut app,
        RollbackMetaOp::LaunchEncounter,
        calibration_ring_id as i16,
        0,
        0,
    );
    wait_until(
        &mut app,
        |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Encounter,
        "encounter phase entered",
    );
    wait_until(
        &mut app,
        |app| count_cleanup_entities(app) > 0,
        "runtime encounter scene spawned",
    );

    enter_cockpit_for_flow_test(&mut app);
    pump_once(&mut app);

    let phase = app.world().resource::<RollbackGameState>().phase;
    let mut query = app.world_mut().query_filtered::<(
        &ShipboardControlState,
        &CurrentStation,
        &PlayerFocusedTile,
        &crate::gameplay::components::InternalPosition,
    ), With<ObservedLocalPlayerMarker>>();
    let (control, current_station, focused_tile, internal_position) = query
        .single(app.world())
        .expect("expected exactly one observed local player state snapshot");

    StationFocusSnapshot {
        phase,
        control_mode: control.mode,
        focus_mode: control.focus_mode,
        focused_kind: control.focused_kind,
        current_station_kind: current_station.kind,
        focused_tile: focused_tile
            .ship
            .map(|_| (focused_tile.grid_x, focused_tile.grid_y)),
        local_tile: (internal_position.grid_x, internal_position.grid_y),
    }
}

fn seed_test_player_ship(app: &mut App) {
    let mut ship = ShipDefinition::empty("Headless Test Ship");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    ship.replace_logistics_tile(ShipFoundationTile::new(
        2,
        ShipFoundationKind::Floor,
        1,
        0,
        0,
    ));
    ship.replace_logistics_tile(ShipFoundationTile::new(
        3,
        ShipFoundationKind::Floor,
        -1,
        0,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
    ship.replace_module(ShipModule::new(2, ModuleKind::Cockpit, 1, 0, 0));
    ship.replace_module(ShipModule::new(3, ModuleKind::Airlock, -1, 0, 0));

    app.world_mut()
        .resource_mut::<RollbackGameState>()
        .editor_ship = ship.clone();
    app.world_mut()
        .resource_mut::<crate::state::EditorShip>()
        .ship = ship.clone();
    app.world_mut()
        .resource_mut::<SessionStatus>()
        .active_ship_snapshot = Some(ship);
}

#[test]
fn launching_after_reselect_uses_the_latest_sector_node() {
    let mut app = build_app(AppRuntimeMode::Headless);

    begin_headless_host_lobby(&mut app);
    start_headless_host_session(&mut app);
    apply_host_meta(&mut app, RollbackMetaOp::OpenSectorMap, 0, 0, 0);

    let (first_node_id, second_node_id, second_node_label) = {
        let rollback = app.world().resource::<RollbackGameState>();
        let launchable: Vec<_> = rollback
            .sector
            .nodes
            .iter()
            .filter(|node| rollback.sector.is_reachable(node.id))
            .filter(|node| !matches!(node.kind, SectorNodeKind::HubStation))
            .collect();
        let first = launchable
            .first()
            .expect("expected at least one launchable sector node");
        let second = launchable
            .get(1)
            .expect("expected at least two launchable sector nodes");
        (first.id, second.id, second.label.clone())
    };

    apply_host_meta(
        &mut app,
        RollbackMetaOp::SelectSectorNode,
        first_node_id as i16,
        0,
        0,
    );
    apply_host_meta(
        &mut app,
        RollbackMetaOp::SelectSectorNode,
        second_node_id as i16,
        0,
        0,
    );
    apply_host_meta(
        &mut app,
        RollbackMetaOp::LaunchEncounter,
        second_node_id as i16,
        0,
        0,
    );

    wait_until(
        &mut app,
        |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Encounter,
        "encounter phase entered after reselection",
    );

    let mut mission_query = app
        .world_mut()
        .query::<&gameplay::components::MissionState>();
    let mission = mission_query
        .single(app.world())
        .expect("expected one active mission state");
    assert_eq!(mission.node_id, second_node_id);
    assert_eq!(mission.node_name, second_node_label);
}

#[test]
fn repeated_encounter_cycles_cleanup_runtime_entities() {
    let mut app = build_app(AppRuntimeMode::Headless);

    begin_headless_host_lobby(&mut app);
    start_headless_host_session(&mut app);
    apply_host_meta(&mut app, RollbackMetaOp::OpenSectorMap, 0, 0, 0);

    let node_id = {
        let rollback = app.world().resource::<RollbackGameState>();
        rollback
            .sector
            .nodes
            .iter()
            .find(|node| rollback.sector.is_reachable(node.id))
            .filter(|node| !matches!(node.kind, SectorNodeKind::HubStation))
            .map(|node| node.id)
            .expect("expected at least one launchable sector node")
    };

    for cycle in 0..2 {
        apply_host_meta(
            &mut app,
            RollbackMetaOp::SelectSectorNode,
            node_id as i16,
            0,
            0,
        );
        apply_host_meta(
            &mut app,
            RollbackMetaOp::LaunchEncounter,
            node_id as i16,
            0,
            0,
        );

        wait_until(
            &mut app,
            |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Encounter,
            "encounter phase entered",
        );
        wait_until(
            &mut app,
            |app| count_cleanup_entities(app) > 0,
            "runtime encounter entities spawned",
        );

        let cleanup_count = count_cleanup_entities(&mut app);
        assert!(
            cleanup_count > 0,
            "expected runtime cleanup entities during encounter cycle {cycle}"
        );

        apply_host_meta(&mut app, RollbackMetaOp::ReturnToDock, 0, 0, 0);
        wait_until(
            &mut app,
            |app| app.world().resource::<RollbackGameState>().phase == RollbackPhase::Docked,
            "return to docked phase",
        );
        wait_until(
            &mut app,
            |app| count_cleanup_entities(app) == 0,
            "runtime encounter entities cleaned up",
        );
    }
}

/// Boots the host-side lobby state without opening sockets so tests can drive rollback locally.
fn begin_headless_host_lobby(app: &mut App) {
    let host_addr = DEFAULT_HOST_ADDR
        .parse()
        .expect("default host addr should parse");
    let session_config = app.world().resource::<SessionConfig>().clone();
    let local_profile = app.world().resource::<LocalPlayerProfile>().clone();
    let initial_state = netcode::load_initial_rollback_state();

    {
        let mut bootstrap = app.world_mut().resource_mut::<SessionBootstrapConfig>();
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

/// Starts a single-player synctest session that exercises the normal rollback presentation flow.
fn start_headless_host_session(app: &mut App) {
    let bootstrap = app.world().resource::<SessionBootstrapConfig>().clone();
    let sync_test = SessionBuilder::<LumenGgrsConfig>::new()
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
    *app.world_mut().resource_mut::<RollbackGameState>() = bootstrap.initial_state;
    app.world_mut().resource_mut::<LocalPlayerHandle>().0 = Some(0);
    app.world_mut().resource_mut::<ObservedLocalPlayer>().handle = Some(0);
    app.world_mut().resource_mut::<PendingLocalMetaCommand>().0 = None;
    app.world_mut()
        .resource_mut::<PendingLocalStationCommand>()
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

/// Applies one host-authored rollback meta command and advances the app through its effects.
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

fn enter_cockpit_for_flow_test(app: &mut App) {
    let mut query = app
        .world_mut()
        .query_filtered::<&mut ShipboardControlState, With<ObservedLocalPlayerMarker>>();
    if let Ok(mut control) = query.single_mut(app.world_mut()) {
        control.mode = ShipControlMode::Cockpit;
        control.focus_mode = StationFocusMode::Focused;
        control.focused_entity = None;
        control.focused_module_id = None;
        control.focused_kind = Some(ModuleKind::Cockpit);
        control.focused_family = Some(StationFamily::Cockpit);
    }
}

fn pump_once(app: &mut App) {
    app.update();
}

fn count_cleanup_entities(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<PlayingCleanup>>();
    query.iter(app.world()).count()
}

fn observed_player_local_position(app: &mut App) -> FixedVec2 {
    let mut query = app
        .world_mut()
        .query_filtered::<&PlayerMotionState, With<ObservedLocalPlayerMarker>>();
    query
        .single(app.world())
        .expect("expected exactly one observed local player motion state")
        .local_position
}

/// Advances the headless app until a gameplay condition becomes true or the test times out.
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
