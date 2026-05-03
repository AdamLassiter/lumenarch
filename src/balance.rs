use std::{fs, path::Path};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_BALANCE_CONFIG_PATH: &str = "saves/balance_config.json";

#[derive(Resource, Clone, Serialize, Deserialize)]
pub(crate) struct BalanceConfig {
    pub(crate) ship: ShipBalanceConfig,
    pub(crate) reactor: ReactorBalanceConfig,
    pub(crate) fields: FieldBalanceConfig,
    pub(crate) atmosphere: AtmosphereBalanceConfig,
    pub(crate) combat: CombatBalanceConfig,
    pub(crate) hostile_ai: HostileAiBalanceConfig,
    pub(crate) logistics: LogisticsBalanceConfig,
    #[serde(default)]
    pub(crate) player: PlayerBalanceConfig,
    #[serde(default)]
    pub(crate) interaction: InteractionBalanceConfig,
    #[serde(default)]
    pub(crate) mission: MissionBalanceConfig,
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
    #[serde(default = "default_ship_throttle_activation_threshold")]
    pub(crate) throttle_activation_threshold: f32,
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
    #[serde(default = "default_player_heat_buildup_rate")]
    pub(crate) player_heat_buildup_rate: f32,
    #[serde(default = "default_player_heat_decay_rate")]
    pub(crate) player_heat_decay_rate: f32,
    #[serde(default = "default_player_heat_damage_threshold")]
    pub(crate) player_heat_damage_threshold: f32,
    #[serde(default = "default_player_heat_damage_rate")]
    pub(crate) player_heat_damage_rate: f32,
    #[serde(default = "default_player_heat_buildup_cap")]
    pub(crate) player_heat_buildup_cap: f32,
    #[serde(default = "default_player_electrical_buildup_rate")]
    pub(crate) player_electrical_buildup_rate: f32,
    #[serde(default = "default_player_electrical_decay_rate")]
    pub(crate) player_electrical_decay_rate: f32,
    #[serde(default = "default_player_electrical_stun_threshold")]
    pub(crate) player_electrical_stun_threshold: f32,
    #[serde(default = "default_player_electrical_stun_duration")]
    pub(crate) player_electrical_stun_duration: f32,
    #[serde(default = "default_player_electrical_stun_damage")]
    pub(crate) player_electrical_stun_damage: i32,
    #[serde(default = "default_player_oxygen_blackout_zero_rate")]
    pub(crate) player_oxygen_blackout_zero_rate: f32,
    #[serde(default = "default_player_oxygen_blackout_critical_rate")]
    pub(crate) player_oxygen_blackout_critical_rate: f32,
    #[serde(default = "default_player_oxygen_blackout_warning_rate")]
    pub(crate) player_oxygen_blackout_warning_rate: f32,
    #[serde(default = "default_player_oxygen_blackout_recovery_rate")]
    pub(crate) player_oxygen_blackout_recovery_rate: f32,
    #[serde(default = "default_player_oxygen_blackout_damage_threshold")]
    pub(crate) player_oxygen_blackout_damage_threshold: f32,
    #[serde(default = "default_player_oxygen_blackout_damage_rate")]
    pub(crate) player_oxygen_blackout_damage_rate: f32,
    #[serde(default = "default_player_zero_oxygen_threshold")]
    pub(crate) player_zero_oxygen_threshold: f32,
    #[serde(default = "default_player_death_stun_duration")]
    pub(crate) player_death_stun_duration: f32,
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
pub(crate) struct AtmosphereBalanceConfig {
    pub(crate) initial_tile_oxygen: f32,
    pub(crate) max_tile_oxygen: f32,
    pub(crate) equalization_rate: f32,
    pub(crate) leak_rate_per_edge: f32,
    pub(crate) destroyed_leak_multiplier: f32,
    #[serde(default = "default_minimum_breach_leak_multiplier")]
    pub(crate) minimum_breach_leak_multiplier: f32,
    #[serde(default = "default_breach_leak_sqrt_divisor")]
    pub(crate) breach_leak_sqrt_divisor: f32,
    pub(crate) player_warning_threshold: f32,
    pub(crate) player_critical_threshold: f32,
    pub(crate) low_oxygen_speed_multiplier: f32,
    pub(crate) critical_oxygen_speed_multiplier: f32,
    pub(crate) hostile_decompression_threshold: f32,
    #[serde(default = "default_decompression_pull_acceleration")]
    pub(crate) decompression_pull_acceleration: f32,
    #[serde(default = "default_decompression_pull_falloff_per_tile")]
    pub(crate) decompression_pull_falloff_per_tile: f32,
}

fn default_minimum_breach_leak_multiplier() -> f32 {
    0.45
}

fn default_breach_leak_sqrt_divisor() -> f32 {
    2.0
}

fn default_decompression_pull_acceleration() -> f32 {
    90.0
}

fn default_decompression_pull_falloff_per_tile() -> f32 {
    0.6
}
fn default_ship_throttle_activation_threshold() -> f32 {
    0.05
}

fn default_player_heat_buildup_rate() -> f32 {
    0.12
}
fn default_player_heat_decay_rate() -> f32 {
    1.4
}
fn default_player_heat_damage_threshold() -> f32 {
    2.5
}
fn default_player_heat_damage_rate() -> f32 {
    0.22
}
fn default_player_heat_buildup_cap() -> f32 {
    12.0
}
fn default_player_electrical_buildup_rate() -> f32 {
    0.32
}
fn default_player_electrical_decay_rate() -> f32 {
    2.0
}
fn default_player_electrical_stun_threshold() -> f32 {
    3.0
}
fn default_player_electrical_stun_duration() -> f32 {
    2.8
}
fn default_player_electrical_stun_damage() -> i32 {
    2
}
fn default_player_oxygen_blackout_zero_rate() -> f32 {
    0.40
}
fn default_player_oxygen_blackout_critical_rate() -> f32 {
    0.18
}
fn default_player_oxygen_blackout_warning_rate() -> f32 {
    0.08
}
fn default_player_oxygen_blackout_recovery_rate() -> f32 {
    0.22
}
fn default_player_oxygen_blackout_damage_threshold() -> f32 {
    0.98
}
fn default_player_oxygen_blackout_damage_rate() -> f32 {
    0.18
}
fn default_player_zero_oxygen_threshold() -> f32 {
    0.2
}
fn default_player_death_stun_duration() -> f32 {
    4.0
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
    #[serde(default = "default_collision_push_stiffness")]
    pub(crate) collision_push_stiffness: f32,
    #[serde(default = "default_collision_restitution")]
    pub(crate) collision_restitution: f32,
    #[serde(default = "default_collision_heat_from_damage")]
    pub(crate) collision_heat_from_damage: f32,
    #[serde(default = "default_collision_max_effective_mass")]
    pub(crate) collision_max_effective_mass: f32,
    #[serde(default = "default_collision_max_effective_speed")]
    pub(crate) collision_max_effective_speed: f32,
    #[serde(default = "default_collision_damage_energy_divisor")]
    pub(crate) collision_damage_energy_divisor: f32,
    #[serde(default = "default_collision_damage_energy_threshold")]
    pub(crate) collision_damage_energy_threshold: f32,
    #[serde(default = "default_component_collider_radius")]
    pub(crate) component_collider_radius: f32,
    #[serde(default = "default_shield_collider_radius")]
    pub(crate) shield_collider_radius: f32,
    #[serde(default = "default_turret_manual_aim_speed")]
    pub(crate) turret_manual_aim_speed: f32,
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

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PlayerBalanceConfig {
    pub(crate) interior_camera_scale: f32,
    pub(crate) exterior_camera_scale: f32,
    pub(crate) walk_acceleration: f32,
    pub(crate) walk_max_speed: f32,
    pub(crate) walk_damping: f32,
    pub(crate) eva_acceleration: f32,
    pub(crate) eva_max_speed: f32,
    pub(crate) eva_damping: f32,
    pub(crate) interact_radius: f32,
    pub(crate) cargo_pickup_radius: f32,
    pub(crate) collision_radius: f32,
    pub(crate) standard_heat_multiplier: f32,
    pub(crate) radiation_heat_multiplier: f32,
    pub(crate) welder_heat_multiplier: f32,
    pub(crate) eva_heat_multiplier: f32,
    pub(crate) standard_electrical_multiplier: f32,
    pub(crate) radiation_electrical_multiplier: f32,
    pub(crate) welder_electrical_multiplier: f32,
    pub(crate) eva_electrical_multiplier: f32,
    pub(crate) standard_oxygen_warning_threshold: f32,
    pub(crate) radiation_oxygen_warning_threshold: f32,
    pub(crate) welder_oxygen_warning_threshold: f32,
    pub(crate) eva_oxygen_warning_threshold: f32,
    pub(crate) standard_oxygen_critical_threshold: f32,
    pub(crate) radiation_oxygen_critical_threshold: f32,
    pub(crate) welder_oxygen_critical_threshold: f32,
    pub(crate) eva_oxygen_critical_threshold: f32,
    pub(crate) standard_eva_speed_multiplier: f32,
    pub(crate) radiation_eva_speed_multiplier: f32,
    pub(crate) welder_eva_speed_multiplier: f32,
    pub(crate) eva_eva_speed_multiplier: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InteractionBalanceConfig {
    pub(crate) repair_hold_duration: f32,
    pub(crate) extract_hold_duration: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MissionBalanceConfig {
    pub(crate) return_delay_seconds: f32,
    pub(crate) hull_wear_heat_threshold: f32,
    pub(crate) low_oxygen_hint_threshold: f32,
}

fn default_collision_push_stiffness() -> f32 {
    0.92
}

fn default_collision_restitution() -> f32 {
    0.22
}

fn default_collision_heat_from_damage() -> f32 {
    0.12
}

fn default_collision_max_effective_mass() -> f32 {
    512.0
}

fn default_collision_max_effective_speed() -> f32 {
    320.0
}

fn default_collision_damage_energy_divisor() -> f32 {
    3600.0
}

fn default_collision_damage_energy_threshold() -> f32 {
    1260.0
}

fn default_component_collider_radius() -> f32 {
    14.08
}

fn default_shield_collider_radius() -> f32 {
    24.32
}

fn default_turret_manual_aim_speed() -> f32 {
    1.8
}

impl Default for PlayerBalanceConfig {
    fn default() -> Self {
        Self {
            interior_camera_scale: 0.58,
            exterior_camera_scale: 1.0,
            walk_acceleration: 260.0,
            walk_max_speed: 120.0,
            walk_damping: 8.0,
            eva_acceleration: 180.0,
            eva_max_speed: 140.0,
            eva_damping: 1.6,
            interact_radius: 28.0,
            cargo_pickup_radius: 20.0,
            collision_radius: 7.0,
            standard_heat_multiplier: 1.0,
            radiation_heat_multiplier: 0.55,
            welder_heat_multiplier: 0.8,
            eva_heat_multiplier: 0.9,
            standard_electrical_multiplier: 1.0,
            radiation_electrical_multiplier: 0.6,
            welder_electrical_multiplier: 0.85,
            eva_electrical_multiplier: 0.95,
            standard_oxygen_warning_threshold: 6.0,
            radiation_oxygen_warning_threshold: 4.0,
            welder_oxygen_warning_threshold: 5.0,
            eva_oxygen_warning_threshold: 2.0,
            standard_oxygen_critical_threshold: 3.0,
            radiation_oxygen_critical_threshold: 2.0,
            welder_oxygen_critical_threshold: 2.0,
            eva_oxygen_critical_threshold: 1.0,
            standard_eva_speed_multiplier: 1.0,
            radiation_eva_speed_multiplier: 0.95,
            welder_eva_speed_multiplier: 0.85,
            eva_eva_speed_multiplier: 1.65,
        }
    }
}

impl Default for InteractionBalanceConfig {
    fn default() -> Self {
        Self {
            repair_hold_duration: 1.8,
            extract_hold_duration: 2.4,
        }
    }
}

impl Default for MissionBalanceConfig {
    fn default() -> Self {
        Self {
            return_delay_seconds: 2.5,
            hull_wear_heat_threshold: 10.0,
            low_oxygen_hint_threshold: 3.0,
        }
    }
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
                throttle_activation_threshold: default_ship_throttle_activation_threshold(),
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
                player_heat_buildup_rate: default_player_heat_buildup_rate(),
                player_heat_decay_rate: default_player_heat_decay_rate(),
                player_heat_damage_threshold: default_player_heat_damage_threshold(),
                player_heat_damage_rate: default_player_heat_damage_rate(),
                player_heat_buildup_cap: default_player_heat_buildup_cap(),
                player_electrical_buildup_rate: default_player_electrical_buildup_rate(),
                player_electrical_decay_rate: default_player_electrical_decay_rate(),
                player_electrical_stun_threshold: default_player_electrical_stun_threshold(),
                player_electrical_stun_duration: default_player_electrical_stun_duration(),
                player_electrical_stun_damage: default_player_electrical_stun_damage(),
                player_oxygen_blackout_zero_rate: default_player_oxygen_blackout_zero_rate(),
                player_oxygen_blackout_critical_rate: default_player_oxygen_blackout_critical_rate(
                ),
                player_oxygen_blackout_warning_rate: default_player_oxygen_blackout_warning_rate(),
                player_oxygen_blackout_recovery_rate: default_player_oxygen_blackout_recovery_rate(
                ),
                player_oxygen_blackout_damage_threshold:
                    default_player_oxygen_blackout_damage_threshold(),
                player_oxygen_blackout_damage_rate: default_player_oxygen_blackout_damage_rate(),
                player_zero_oxygen_threshold: default_player_zero_oxygen_threshold(),
                player_death_stun_duration: default_player_death_stun_duration(),
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
            atmosphere: AtmosphereBalanceConfig {
                initial_tile_oxygen: 10.0,
                max_tile_oxygen: 10.0,
                equalization_rate: 2.2,
                leak_rate_per_edge: 2.6,
                destroyed_leak_multiplier: 1.8,
                minimum_breach_leak_multiplier: default_minimum_breach_leak_multiplier(),
                breach_leak_sqrt_divisor: default_breach_leak_sqrt_divisor(),
                player_warning_threshold: 5.0,
                player_critical_threshold: 2.2,
                low_oxygen_speed_multiplier: 0.72,
                critical_oxygen_speed_multiplier: 0.48,
                hostile_decompression_threshold: 2.0,
                decompression_pull_acceleration: default_decompression_pull_acceleration(),
                decompression_pull_falloff_per_tile: default_decompression_pull_falloff_per_tile(),
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
                collision_push_stiffness: default_collision_push_stiffness(),
                collision_restitution: default_collision_restitution(),
                collision_heat_from_damage: default_collision_heat_from_damage(),
                collision_max_effective_mass: default_collision_max_effective_mass(),
                collision_max_effective_speed: default_collision_max_effective_speed(),
                collision_damage_energy_divisor: default_collision_damage_energy_divisor(),
                collision_damage_energy_threshold: default_collision_damage_energy_threshold(),
                component_collider_radius: default_component_collider_radius(),
                shield_collider_radius: default_shield_collider_radius(),
                turret_manual_aim_speed: default_turret_manual_aim_speed(),
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
            player: PlayerBalanceConfig::default(),
            interaction: InteractionBalanceConfig::default(),
            mission: MissionBalanceConfig::default(),
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
