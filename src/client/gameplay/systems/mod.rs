mod control;
mod interactions;
mod shared;
mod simulation;
mod ui;

pub(crate) use control::{
    apply_player_ship_controls,
    camera_follow_player_ship,
    exit_focused_station,
    handle_player_cargo_interaction,
    integrate_player_ship_motion,
    move_shipboard_player,
    return_button_system,
    return_keyboard_shortcut,
    sync_shipboard_player_visual,
    toggle_shipboard_control_mode,
    update_current_station,
    update_player_reference_frame,
    update_station_command_input,
};
pub(crate) use interactions::{
    apply_module_interactions,
    begin_held_interactions,
    complete_held_interactions,
    detect_nearby_interactions,
    run_shipboard_interaction_input,
};
pub(crate) use simulation::{
    advance_projectiles,
    aim_hostile_turrets,
    collect_salvage,
    draw_debug_overlay,
    drive_hostile_ships,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    fire_player_weapons,
    handle_projectile_hits,
    integrate_hostile_ship_motion,
    return_after_mission_resolution,
    run_arch_automation,
    run_logistics_transfers,
    run_processors,
    sample_ship_fields,
    sync_hostile_ship_state,
    sync_runtime_ship_state,
    tick_recent_action_feedback,
    toggle_debug_overlay,
    update_destroyed_module_visuals,
    update_mission_state,
    update_mission_telemetry,
    update_module_runtime_state,
};
pub(crate) use ui::{update_gameplay_status_text, update_inspection_and_alerts_text};
