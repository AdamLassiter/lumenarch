mod arch;
mod atmosphere;
mod collisions;
mod detectors;
mod drones;
mod fields;
mod hostiles;
mod infrastructure;
mod logistics;
mod mission;
mod player;
mod projectiles;
mod snapshots;
mod transfers;
mod visuals;

pub(crate) use arch::{run_arch_automation, tick_recent_action_feedback};
pub(crate) use atmosphere::{sample_player_atmosphere, update_ship_atmosphere};
pub(crate) use collisions::handle_ship_collisions;
pub(crate) use detectors::update_detector_modules;
pub(crate) use fields::{
    apply_player_environmental_effects,
    sample_ship_fields,
    update_module_runtime_state,
};
pub(crate) use hostiles::{
    aim_hostile_turrets,
    drive_hostile_ships,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    integrate_hostile_ship_motion,
    sync_hostile_ship_state,
};
pub(crate) use infrastructure::{rebuild_infrastructure_networks, update_routed_ship_power};
pub(crate) use logistics::{
    collect_salvage,
    run_drone_logistics,
    run_logistics_transfers,
    run_processors,
    sync_drone_station_population,
};
pub(crate) use mission::{
    return_after_mission_resolution,
    sync_runtime_ship_state,
    update_mission_state,
    update_mission_telemetry,
};
pub(crate) use player::fire_player_weapons;
pub(crate) use projectiles::{advance_projectiles, handle_projectile_hits};
pub(crate) use visuals::{
    draw_debug_overlay,
    spawn_missing_effect_overlays,
    sync_backdrop_parallax,
    sync_battery_pulse_visuals,
    sync_engine_flame_visuals,
    sync_eva_thruster_visuals,
    sync_fabricator_dust_visuals,
    sync_hazard_effect_visuals,
    sync_module_work_effect_visuals,
    sync_reactor_glow_visuals,
    sync_service_link_visuals,
    sync_ship_environment_effect_visuals,
    sync_turret_flash_visuals,
    update_destroyed_module_visuals,
};
