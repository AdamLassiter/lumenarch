mod components;
mod helpers;
mod spawn;
mod systems;

pub(crate) use spawn::{cleanup_runtime_entities, spawn_runtime_scene};
pub(crate) use systems::{
    advance_projectiles, aim_hostile_turrets, apply_player_ship_controls, camera_follow_player_ship,
    collect_salvage,
    fire_hostile_targets, fire_player_weapons, handle_projectile_hits,
    return_after_mission_resolution,
    return_button_system, return_keyboard_shortcut, sync_runtime_ship_state,
    update_mission_state,
    update_destroyed_module_visuals, update_gameplay_status_text,
    integrate_player_ship_motion,
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
