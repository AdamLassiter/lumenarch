mod atmosphere;
mod automation;
mod combat;
mod fields;
mod logistics;
mod mission;
mod visuals;

pub(crate) use atmosphere::{sample_player_atmosphere, update_ship_atmosphere};
pub(crate) use automation::{run_arch_automation, tick_recent_action_feedback};
pub(crate) use combat::{
    advance_projectiles,
    aim_hostile_turrets,
    drive_hostile_ships,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    fire_player_weapons,
    handle_projectile_hits,
    handle_ship_collisions,
    integrate_hostile_ship_motion,
    sync_hostile_ship_state,
};
pub(crate) use fields::{
    apply_player_environmental_effects,
    sample_ship_fields,
    update_module_runtime_state,
};
pub(crate) use logistics::{collect_salvage, run_logistics_transfers, run_processors};
pub(crate) use mission::{
    return_after_mission_resolution,
    sync_runtime_ship_state,
    update_mission_state,
    update_mission_telemetry,
};
pub(crate) use visuals::{draw_debug_overlay, update_destroyed_module_visuals};
