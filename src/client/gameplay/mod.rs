pub(crate) mod components;
mod helpers;
mod spawn;
mod systems;

pub(crate) use spawn::{cleanup_runtime_entities, spawn_runtime_scene};
pub(crate) use systems::{
    advance_projectiles,
    aim_hostile_turrets,
    apply_module_interactions,
    apply_player_ship_controls,
    begin_held_interactions,
    camera_follow_player_ship,
    collect_salvage,
    complete_held_interactions,
    detect_nearby_interactions,
    draw_debug_overlay,
    drive_hostile_ships,
    exit_focused_station,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    fire_player_weapons,
    handle_player_cargo_interaction,
    handle_projectile_hits,
    integrate_hostile_ship_motion,
    integrate_player_ship_motion,
    move_shipboard_player,
    return_after_mission_resolution,
    return_button_system,
    return_keyboard_shortcut,
    run_arch_automation,
    run_logistics_transfers,
    run_processors,
    run_shipboard_interaction_input,
    sample_player_atmosphere,
    sample_ship_fields,
    station_panel_button_system,
    sync_hostile_ship_state,
    sync_player_reference_frame_parenting,
    sync_remote_session_players,
    sync_runtime_ship_state,
    sync_shipboard_player_visual,
    tick_recent_action_feedback,
    toggle_debug_overlay,
    toggle_shipboard_control_mode,
    update_current_station,
    update_destroyed_module_visuals,
    update_gameplay_status_text,
    update_inspection_and_alerts_text,
    update_mission_state,
    update_mission_telemetry,
    update_module_runtime_state,
    update_player_reference_frame,
    update_ship_atmosphere,
    update_station_command_input,
};

const RUNTIME_SHIP_ORIGIN: bevy::prelude::Vec3 = bevy::prelude::Vec3::new(0.0, 0.0, 10.0);
const ARENA_WIDTH_TILES: i32 = 48;
const ARENA_HEIGHT_TILES: i32 = 32;
