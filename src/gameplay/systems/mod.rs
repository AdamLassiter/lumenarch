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
    sync_crew_name_labels,
    sync_player_reference_frame_parenting,
    sync_shipboard_player_visual,
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
    apply_player_environmental_effects,
    collect_salvage,
    draw_debug_overlay,
    drive_hostile_ships,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    fire_player_weapons,
    handle_projectile_hits,
    handle_ship_collisions,
    integrate_hostile_ship_motion,
    rebuild_infrastructure_networks,
    return_after_mission_resolution,
    run_arch_automation,
    run_drone_logistics,
    run_logistics_transfers,
    run_processors,
    sample_player_atmosphere,
    sample_ship_fields,
    sync_backdrop_parallax,
    sync_drone_station_population,
    sync_engine_flame_visuals,
    sync_eva_thruster_visuals,
    sync_hostile_ship_state,
    sync_module_work_effect_visuals,
    sync_reactor_glow_visuals,
    sync_runtime_ship_state,
    tick_recent_action_feedback,
    update_destroyed_module_visuals,
    update_detector_modules,
    update_mission_state,
    update_mission_telemetry,
    update_module_runtime_state,
    update_routed_ship_power,
    update_ship_atmosphere,
};
pub(crate) use ui::{
    station_panel_button_system,
    toggle_gameplay_info_panel,
    update_gameplay_status_text,
};
