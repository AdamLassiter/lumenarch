mod bootstrap;
mod input;
mod lobby;
mod presentation;
mod types;

#[cfg(test)]
pub(crate) use bootstrap::load_initial_rollback_state;
pub(crate) use bootstrap::{
    begin_session_attempt,
    finalize_pending_session_bootstrap,
    request_lobby_session_start,
};
pub(crate) use input::{
    apply_host_meta_ops,
    command_for_handle,
    decode_player_inputs,
    local_player_input,
    read_local_inputs,
    rollback_phase_is_encounter,
};
pub(crate) use lobby::{LobbyRuntime, poll_lobby_runtime_events, sync_lobby_profile_changes};
pub(crate) use presentation::{
    advance_rollback_state,
    editor_ui_should_not_be_present,
    frontend_mode_is_debug_enemy_editor,
    frontend_mode_is_lobby,
    frontend_mode_is_not_lobby,
    session_not_presents_docked,
    session_not_presents_encounter,
    session_not_presents_sector_map,
    session_presents_docked,
    session_presents_encounter,
    session_presents_player_editor,
    session_presents_sector_map,
    sync_active_presentation_phase,
    sync_local_player_handle,
    sync_player_editor_mode,
    sync_presentation_from_rollback,
};
pub(crate) use types::{
    ActivePresentationPhase,
    ChecksumHistory,
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
    LobbyPlayerInfo,
    LobbySnapshot,
    LocalPlayerHandle,
    LumenGgrsConfig,
    ObservedLocalPlayer,
    PendingLocalMetaCommand,
    PendingLocalStationCommand,
    PendingMetaCommand,
    PendingStationCommand,
    PlayerGgrsInput,
    PlayerHandleMap,
    RollbackGameState,
    RollbackMetaOp,
    RollbackPhase,
    SessionBootstrapConfig,
    SessionConfig,
    SessionPhase,
    SessionRole,
    SessionStatus,
    StationControlOp,
    is_host_authority,
};
