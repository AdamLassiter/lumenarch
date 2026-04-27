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
    fire_hostile_targets,
    fire_player_weapons,
    handle_projectile_hits,
    integrate_player_ship_motion,
    move_shipboard_player,
    return_after_mission_resolution,
    return_button_system,
    return_keyboard_shortcut,
    run_shipboard_interaction_input,
    sample_ship_fields,
    sync_runtime_ship_state,
    sync_shipboard_player_visual,
    toggle_shipboard_control_mode,
    update_destroyed_module_visuals,
    update_gameplay_status_text,
    update_inspection_and_alerts_text,
    update_mission_state,
    update_module_runtime_state,
};

const RUNTIME_SHIP_ORIGIN: bevy::prelude::Vec3 = bevy::prelude::Vec3::new(0.0, 0.0, 10.0);
const ARENA_WIDTH_TILES: i32 = 48;
const ARENA_HEIGHT_TILES: i32 = 32;
const CAMERA_FOLLOW_LERP_RATE: f32 = 8.0;
const PROJECTILE_SPEED: f32 = 420.0;
const PROJECTILE_LIFETIME: f32 = 1.6;
const PROJECTILE_RADIUS: f32 = 8.0;
const HOSTILE_TARGET_RADIUS: f32 = 18.0;
const MODULE_HIT_RADIUS: f32 = 15.0;
const HOSTILE_PROJECTILE_SPEED: f32 = 180.0;
const HOSTILE_FIRE_COOLDOWN: f32 = 1.8;
const SALVAGE_PICKUP_RADIUS: f32 = 42.0;
