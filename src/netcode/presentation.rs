use bevy::{log, prelude::*};
use bevy_ggrs::{LocalPlayers, RollbackFrameCount};

use super::{
    ActivePresentationPhase,
    ChecksumHistory,
    LocalPlayerHandle,
    RollbackGameState,
    RollbackPhase,
    bootstrap::stable_hash,
};
use crate::state::{
    EditorMode,
    EditorSessionState,
    EditorShip,
    FrontendMode,
    LastMissionReport,
    Progression,
    SectorState,
};

pub(crate) fn advance_rollback_state(
    frame: Res<RollbackFrameCount>,
    mut rollback_state: ResMut<RollbackGameState>,
    mut checksum_history: ResMut<ChecksumHistory>,
) {
    if frame.0 < checksum_history.last_frame {
        log::warn!(
            "Rollback detected: frame moved backwards from {} to {}",
            checksum_history.last_frame,
            frame.0
        );
    } else if frame.0 == checksum_history.last_frame {
        log::trace!("Rollback frame repeated at {}", frame.0);
    }
    rollback_state.frame = frame.0;
    checksum_history.last_frame = frame.0;
    checksum_history.last_value = stable_hash(rollback_state.as_ref());
    if frame.0 % 30 == 0 {
        log::debug!(
            "Rollback state checkpoint: frame={}, phase={:?}, checksum={:016x}, scene_generation={}",
            rollback_state.frame,
            rollback_state.phase,
            checksum_history.last_value,
            rollback_state.scene_generation
        );
    }
}

pub(crate) fn sync_presentation_from_rollback(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    mut sector: ResMut<SectorState>,
    mut last_mission_report: ResMut<LastMissionReport>,
) {
    if *frontend_mode.get() == FrontendMode::DebugEnemyEditor {
        return;
    }
    if !rollback_state.is_changed() {
        return;
    }
    log::debug!(
        "Mirroring rollback state into presentation resources: phase={:?}, frame={}, scene_generation={}, current_node={}, selected_node={:?}, active_node={:?}",
        rollback_state.phase,
        rollback_state.frame,
        rollback_state.scene_generation,
        rollback_state.sector.current_node_id,
        rollback_state.sector.selected_node_id,
        rollback_state.sector.active_encounter_node_id
    );
    if stable_hash(&editor_ship.ship) != stable_hash(&rollback_state.editor_ship) {
        editor_ship.ship = rollback_state.editor_ship.clone();
    }
    if stable_hash(progression.as_ref()) != stable_hash(&rollback_state.progression) {
        *progression = rollback_state.progression.clone();
    }
    if stable_hash(sector.as_ref()) != stable_hash(&rollback_state.sector) {
        *sector = rollback_state.sector.clone();
    }
    if stable_hash(last_mission_report.as_ref()) != stable_hash(&rollback_state.last_mission_report)
    {
        *last_mission_report = rollback_state.last_mission_report.clone();
    }
}

pub(crate) fn sync_active_presentation_phase(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
    mut active_phase: ResMut<ActivePresentationPhase>,
) {
    let desired = if *frontend_mode.get() == FrontendMode::Session {
        ActivePresentationPhase {
            phase: Some(rollback_state.phase),
            scene_generation: (rollback_state.phase == RollbackPhase::Encounter)
                .then_some(rollback_state.scene_generation),
        }
    } else {
        ActivePresentationPhase::default()
    };
    if *active_phase != desired {
        log::info!(
            "Presentation phase transition: frontend={:?}, old={:?}/{:?}, new={:?}/{:?}",
            frontend_mode.get(),
            active_phase.phase,
            active_phase.scene_generation,
            desired.phase,
            desired.scene_generation
        );
        *active_phase = desired;
    }
}

pub(crate) fn sync_player_editor_mode(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
    mut editor_session: ResMut<EditorSessionState>,
) {
    if *frontend_mode.get() == FrontendMode::Session
        && rollback_state.phase == RollbackPhase::Editing
        && editor_session.mode != EditorMode::Player
    {
        log::debug!(
            "Synchronizing editor session mode to Player for rollback-driven editing phase"
        );
        editor_session.mode = EditorMode::Player;
    }
}

pub(crate) fn frontend_mode_is_session(frontend_mode: Res<State<FrontendMode>>) -> bool {
    *frontend_mode.get() == FrontendMode::Session
}

pub(crate) fn frontend_mode_is_lobby(frontend_mode: Res<State<FrontendMode>>) -> bool {
    *frontend_mode.get() == FrontendMode::Lobby
}

pub(crate) fn frontend_mode_is_debug_enemy_editor(frontend_mode: Res<State<FrontendMode>>) -> bool {
    *frontend_mode.get() == FrontendMode::DebugEnemyEditor
}

pub(crate) fn frontend_mode_is_not_lobby(frontend_mode: Res<State<FrontendMode>>) -> bool {
    *frontend_mode.get() != FrontendMode::Lobby
}

pub(crate) fn frontend_mode_is_not_debug_enemy_editor(
    frontend_mode: Res<State<FrontendMode>>,
) -> bool {
    *frontend_mode.get() != FrontendMode::DebugEnemyEditor
}

pub(crate) fn session_presents_docked(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    *frontend_mode.get() == FrontendMode::Session && rollback_state.phase == RollbackPhase::Docked
}

pub(crate) fn session_not_presents_docked(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    !session_presents_docked(frontend_mode, rollback_state)
}

pub(crate) fn session_presents_sector_map(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    *frontend_mode.get() == FrontendMode::Session
        && rollback_state.phase == RollbackPhase::SectorMap
}

pub(crate) fn session_not_presents_sector_map(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    !session_presents_sector_map(frontend_mode, rollback_state)
}

pub(crate) fn session_presents_player_editor(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    *frontend_mode.get() == FrontendMode::Session && rollback_state.phase == RollbackPhase::Editing
}

pub(crate) fn session_not_presents_player_editor(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    !session_presents_player_editor(frontend_mode, rollback_state)
}

pub(crate) fn editor_ui_should_not_be_present(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    *frontend_mode.get() != FrontendMode::DebugEnemyEditor
        && !(*frontend_mode.get() == FrontendMode::Session
            && rollback_state.phase == RollbackPhase::Editing)
}

pub(crate) fn session_presents_encounter(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    *frontend_mode.get() == FrontendMode::Session
        && rollback_state.phase == RollbackPhase::Encounter
}

pub(crate) fn session_not_presents_encounter(
    frontend_mode: Res<State<FrontendMode>>,
    rollback_state: Res<RollbackGameState>,
) -> bool {
    !session_presents_encounter(frontend_mode, rollback_state)
}

pub(crate) fn sync_local_player_handle(
    local_players: Res<LocalPlayers>,
    mut local_handle: ResMut<LocalPlayerHandle>,
) {
    let new_handle = local_players.0.first().copied();
    if local_handle.0 != new_handle {
        log::info!(
            "Local player handle changed from {:?} to {:?} (all local handles: {:?})",
            local_handle.0,
            new_handle,
            local_players.0
        );
        local_handle.0 = new_handle;
    }
}
