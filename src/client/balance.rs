use std::{fs, path::Path};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_BALANCE_CONFIG_PATH: &str = "saves/balance_config.json";

#[derive(Resource, Clone, Serialize, Deserialize)]
pub(crate) struct BalanceConfig {
    pub(crate) ship: ShipBalanceConfig,
    pub(crate) reactor: ReactorBalanceConfig,
    pub(crate) fields: FieldBalanceConfig,
    pub(crate) combat: CombatBalanceConfig,
    pub(crate) hostile_ai: HostileAiBalanceConfig,
    pub(crate) logistics: LogisticsBalanceConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ShipBalanceConfig {
    pub(crate) thrust_base_acceleration: f32,
    pub(crate) turn_speed_base: f32,
    pub(crate) turn_speed_per_engine: f32,
    pub(crate) max_speed_base: f32,
    pub(crate) max_speed_per_engine: f32,
    pub(crate) linear_damping: f32,
    pub(crate) angular_damping: f32,
    pub(crate) reactor_output_per_reactor: f32,
    pub(crate) battery_capacity_per_battery: f32,
    pub(crate) passive_draw_base: f32,
    pub(crate) passive_draw_per_module: f32,
    pub(crate) engine_draw_per_engine: f32,
    pub(crate) weapon_draw_per_turret: f32,
    pub(crate) reactor_output_floor_per_reactor: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ReactorBalanceConfig {
    pub(crate) warmup_threshold: f32,
    pub(crate) full_power_threshold: f32,
    pub(crate) fuel_burn_rate: f32,
    pub(crate) cold_reaction_decay: f32,
    pub(crate) reaction_power_factor: f32,
    pub(crate) turbine_power_factor: f32,
    pub(crate) max_power_output: f32,
    pub(crate) reaction_heat_factor: f32,
    pub(crate) turbine_cooling_factor: f32,
    pub(crate) control_adjust_rate: f32,
    pub(crate) starting_reaction_rate: f32,
    pub(crate) starting_turbine_load: f32,
    pub(crate) starting_power_output: f32,
    pub(crate) starting_fuel: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct FieldBalanceConfig {
    pub(crate) attention_heat_multiplier: f32,
    pub(crate) attention_grounding_penalty: f32,
    pub(crate) damage_heat_bonus: f32,
    pub(crate) damage_electrical_bonus: f32,
    pub(crate) damage_grounding_loss: f32,
    pub(crate) instability_leak_factor: f32,
    pub(crate) attenuation_radius_tiles: f32,
    pub(crate) reactor_base_heat: f32,
    pub(crate) engine_base_heat: f32,
    pub(crate) turret_base_heat: f32,
    pub(crate) battery_base_heat: f32,
    pub(crate) computer_base_heat: f32,
    pub(crate) generic_base_heat: f32,
    pub(crate) reactor_heat_min_bonus: f32,
    pub(crate) sampled_heat_factor: f32,
    pub(crate) heat_response_rate: f32,
    pub(crate) sampled_electrical_factor: f32,
    pub(crate) damage_instability_factor: f32,
    pub(crate) electrical_decay_rate: f32,
    pub(crate) degraded_heat_threshold: f32,
    pub(crate) degraded_electrical_threshold: f32,
    pub(crate) disabled_heat_threshold: f32,
    pub(crate) disabled_electrical_threshold: f32,
    pub(crate) player_heat_warning_threshold: f32,
    pub(crate) player_electrical_warning_threshold: f32,
    pub(crate) emitter_reactor_heat: f32,
    pub(crate) emitter_reactor_electrical: f32,
    pub(crate) emitter_reactor_grounding: f32,
    pub(crate) emitter_engine_heat: f32,
    pub(crate) emitter_engine_electrical: f32,
    pub(crate) emitter_engine_grounding: f32,
    pub(crate) emitter_turret_heat: f32,
    pub(crate) emitter_turret_electrical: f32,
    pub(crate) emitter_turret_grounding: f32,
    pub(crate) emitter_battery_heat: f32,
    pub(crate) emitter_battery_electrical: f32,
    pub(crate) emitter_battery_grounding: f32,
    pub(crate) emitter_computer_heat: f32,
    pub(crate) emitter_computer_electrical: f32,
    pub(crate) emitter_computer_grounding: f32,
    pub(crate) emitter_processor_heat: f32,
    pub(crate) emitter_processor_electrical: f32,
    pub(crate) emitter_processor_grounding: f32,
    pub(crate) emitter_hull_cooling: f32,
    pub(crate) emitter_hull_grounding: f32,
    pub(crate) emitter_generic_grounding: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CombatBalanceConfig {
    pub(crate) camera_follow_lerp_rate: f32,
    pub(crate) projectile_speed: f32,
    pub(crate) projectile_lifetime: f32,
    pub(crate) projectile_radius: f32,
    pub(crate) hostile_target_radius: f32,
    pub(crate) module_hit_radius: f32,
    pub(crate) hostile_projectile_speed: f32,
    pub(crate) hostile_fire_cooldown: f32,
    pub(crate) player_weapon_cooldown: f32,
    pub(crate) turret_rotation_speed: f32,
    pub(crate) muzzle_offset_tiles: f32,
    pub(crate) hostile_projectile_damage: i32,
    pub(crate) hostile_projectile_heat_damage: f32,
    pub(crate) hostile_projectile_electrical_damage: f32,
    pub(crate) salvage_pickup_radius: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct HostileAiBalanceConfig {
    pub(crate) brawler_preferred_range: f32,
    pub(crate) skirmisher_preferred_range: f32,
    pub(crate) default_preferred_range: f32,
    pub(crate) default_aggression: f32,
    pub(crate) turn_slowdown_angle: f32,
    pub(crate) firing_angle_threshold: f32,
    pub(crate) far_range_multiplier: f32,
    pub(crate) near_range_multiplier: f32,
    pub(crate) close_throttle: f32,
    pub(crate) cruise_throttle: f32,
    pub(crate) turn_slowdown_multiplier: f32,
    pub(crate) salvage_reward_base: u32,
    pub(crate) salvage_reward_per_threat: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct LogisticsBalanceConfig {
    pub(crate) manipulator_transfer_duration: f32,
    pub(crate) manipulator_range_tiles: f32,
    pub(crate) processor_duration: f32,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            ship: ShipBalanceConfig {
                thrust_base_acceleration: 260.0,
                turn_speed_base: 1.2,
                turn_speed_per_engine: 0.35,
                max_speed_base: 110.0,
                max_speed_per_engine: 24.0,
                linear_damping: 0.9,
                angular_damping: 5.0,
                reactor_output_per_reactor: 8.0,
                battery_capacity_per_battery: 24.0,
                passive_draw_base: 1.0,
                passive_draw_per_module: 0.08,
                engine_draw_per_engine: 2.5,
                weapon_draw_per_turret: 2.0,
                reactor_output_floor_per_reactor: 0.8,
            },
            reactor: ReactorBalanceConfig {
                warmup_threshold: 3.0,
                full_power_threshold: 6.0,
                fuel_burn_rate: 0.22,
                cold_reaction_decay: 0.35,
                reaction_power_factor: 3.0,
                turbine_power_factor: 9.0,
                max_power_output: 12.0,
                reaction_heat_factor: 4.8,
                turbine_cooling_factor: 2.2,
                control_adjust_rate: 0.45,
                starting_reaction_rate: 0.5,
                starting_turbine_load: 0.5,
                starting_power_output: 4.0,
                starting_fuel: 100.0,
            },
            fields: FieldBalanceConfig {
                attention_heat_multiplier: 1.5,
                attention_grounding_penalty: 0.2,
                damage_heat_bonus: 3.0,
                damage_electrical_bonus: 0.6,
                damage_grounding_loss: 0.4,
                instability_leak_factor: 1.0 / 24.0,
                attenuation_radius_tiles: 3.5,
                reactor_base_heat: 0.9,
                engine_base_heat: 0.45,
                turret_base_heat: 0.3,
                battery_base_heat: 0.2,
                computer_base_heat: 0.15,
                generic_base_heat: 0.05,
                reactor_heat_min_bonus: -1.5,
                sampled_heat_factor: 0.45,
                heat_response_rate: 1.35,
                sampled_electrical_factor: 0.08,
                damage_instability_factor: 0.45,
                electrical_decay_rate: 0.5,
                degraded_heat_threshold: 9.0,
                degraded_electrical_threshold: 8.0,
                disabled_heat_threshold: 16.0,
                disabled_electrical_threshold: 14.0,
                player_heat_warning_threshold: 8.0,
                player_electrical_warning_threshold: 7.0,
                emitter_reactor_heat: 1.0,
                emitter_reactor_electrical: 0.5,
                emitter_reactor_grounding: 0.2,
                emitter_engine_heat: 1.0,
                emitter_engine_electrical: 0.5,
                emitter_engine_grounding: 0.2,
                emitter_turret_heat: 2.0,
                emitter_turret_electrical: 1.0,
                emitter_turret_grounding: 0.2,
                emitter_battery_heat: 0.5,
                emitter_battery_electrical: 2.0,
                emitter_battery_grounding: 1.4,
                emitter_computer_heat: 0.3,
                emitter_computer_electrical: 0.4,
                emitter_computer_grounding: 1.6,
                emitter_processor_heat: 0.8,
                emitter_processor_electrical: 0.4,
                emitter_processor_grounding: 0.8,
                emitter_hull_cooling: 2.0,
                emitter_hull_grounding: 2.8,
                emitter_generic_grounding: 0.8,
            },
            combat: CombatBalanceConfig {
                camera_follow_lerp_rate: 8.0,
                projectile_speed: 420.0,
                projectile_lifetime: 1.6,
                projectile_radius: 8.0,
                hostile_target_radius: 18.0,
                module_hit_radius: 15.0,
                hostile_projectile_speed: 180.0,
                hostile_fire_cooldown: 1.8,
                player_weapon_cooldown: 0.3,
                turret_rotation_speed: 2.6,
                muzzle_offset_tiles: 0.35,
                hostile_projectile_damage: 3,
                hostile_projectile_heat_damage: 1.2,
                hostile_projectile_electrical_damage: 0.8,
                salvage_pickup_radius: 42.0,
            },
            hostile_ai: HostileAiBalanceConfig {
                brawler_preferred_range: 120.0,
                skirmisher_preferred_range: 220.0,
                default_preferred_range: 180.0,
                default_aggression: 0.85,
                turn_slowdown_angle: 0.45,
                firing_angle_threshold: 0.35,
                far_range_multiplier: 1.15,
                near_range_multiplier: 0.75,
                close_throttle: 0.2,
                cruise_throttle: 0.55,
                turn_slowdown_multiplier: 0.35,
                salvage_reward_base: 4,
                salvage_reward_per_threat: 3,
            },
            logistics: LogisticsBalanceConfig {
                manipulator_transfer_duration: 0.75,
                manipulator_range_tiles: 2.5,
                processor_duration: 2.2,
            },
        }
    }
}

pub(crate) fn load_or_create_default_balance() -> Result<BalanceConfig, String> {
    let path = Path::new(DEFAULT_BALANCE_CONFIG_PATH);
    match load_balance_from_path(path)? {
        Some(config) => Ok(config),
        None => {
            let config = BalanceConfig::default();
            save_balance_to_path(path, &config)?;
            Ok(config)
        }
    }
}

fn load_balance_from_path(path: &Path) -> Result<Option<BalanceConfig>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let encoded = fs::read_to_string(path)
        .map_err(|error| format!("failed to read balance config {}: {error}", path.display()))?;
    let config = serde_json::from_str(&encoded).map_err(|error| {
        format!(
            "failed to decode balance config {}: {error}",
            path.display()
        )
    })?;
    Ok(Some(config))
}

fn save_balance_to_path(path: &Path, config: &BalanceConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create balance config directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let encoded = serde_json::to_string_pretty(config).map_err(|error| {
        format!(
            "failed to encode balance config {}: {error}",
            path.display()
        )
    })?;
    fs::write(path, encoded)
        .map_err(|error| format!("failed to write balance config {}: {error}", path.display()))
}
