use bevy::prelude::*;

use super::super::helpers::Fx;
use crate::ship::ModuleKind;

#[derive(Component, Clone)]
pub(crate) struct ShipMovementModel {
    pub(crate) engine_count: u32,
    pub(crate) helm_multiplier: Fx,
    pub(crate) thrust_acceleration: Fx,
    pub(crate) turn_speed: Fx,
    pub(crate) max_speed: Fx,
    pub(crate) linear_damping: Fx,
    pub(crate) angular_damping: Fx,
}

#[derive(Component, Clone)]
pub(crate) struct ShipPowerModel {
    pub(crate) reactor_output: Fx,
    pub(crate) battery_capacity: Fx,
    pub(crate) battery_charge_rate: Fx,
    pub(crate) battery_discharge_rate: Fx,
    pub(crate) passive_draw: Fx,
    pub(crate) engine_draw: Fx,
    pub(crate) weapon_draw: Fx,
    pub(crate) shield_draw: Fx,
}

#[derive(Component, Clone)]
pub(crate) struct ShipPowerState {
    pub(crate) stored_energy: Fx,
    pub(crate) generation: Fx,
    pub(crate) draw: Fx,
    pub(crate) surplus: Fx,
    pub(crate) engine_power_ratio: Fx,
    pub(crate) weapons_powered: bool,
    pub(crate) engines_powered: bool,
}

#[derive(Component, Default, Clone)]
pub(crate) struct ShipControlState {
    pub(crate) throttle_demand: Fx,
    pub(crate) thrust_active: bool,
    pub(crate) turn_input: Fx,
    pub(crate) fire_pressed: bool,
}

#[derive(Component, Clone)]
pub(crate) struct ShipWeaponState {
    pub(crate) turret_count: u32,
    pub(crate) cooldown_remaining: Fx,
    pub(crate) cooldown_duration: Fx,
    pub(crate) shield_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipAutomationMode {
    Off,
    ReactorGuard,
    LogisticsFeed,
    TurretAssist,
    BalancedOps,
    Mixed,
}

#[derive(Component, Clone)]
pub(crate) struct ShipAutomationState {
    pub(crate) mode: ShipAutomationMode,
    pub(crate) active: bool,
    pub(crate) output_scale: Fx,
    pub(crate) trigger_count: u32,
    pub(crate) invalid_executions: u32,
    pub(crate) last_primary_program: Option<String>,
    pub(crate) last_secondary_program: Option<String>,
    pub(crate) recent_writes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum ArchLogisticsPreference {
    #[default]
    FeedProcessor,
    StoreCharges,
}

#[derive(Component, Clone)]
pub(crate) struct ShipArchCommandState {
    pub(crate) reactor_cooling_bias: Fx,
    pub(crate) logistics_enabled: bool,
    pub(crate) logistics_preference: ArchLogisticsPreference,
    pub(crate) turret_assist_enabled: bool,
    pub(crate) turret_auto_fire: bool,
    pub(crate) turret_fire_hold: bool,
}

impl Default for ShipArchCommandState {
    fn default() -> Self {
        Self {
            reactor_cooling_bias: Fx::from_num(0),
            logistics_enabled: false,
            logistics_preference: ArchLogisticsPreference::FeedProcessor,
            turret_assist_enabled: false,
            turret_auto_fire: false,
            turret_fire_hold: false,
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct MissionState {
    pub(crate) node_id: u32,
    pub(crate) node_name: String,
    pub(crate) node_kind_name: String,
    pub(crate) reward_multiplier: u32,
    pub(crate) ambient_heat_pressure: Fx,
    pub(crate) ambient_electrical_pressure: Fx,
    pub(crate) failed: bool,
    pub(crate) failure_reason: Option<String>,
    pub(crate) encounter_cleared: bool,
    pub(crate) completed: bool,
    pub(crate) completion_reason: Option<String>,
    pub(crate) salvage_collected: bool,
    pub(crate) salvage_scrap_awarded: u32,
    pub(crate) return_delay_remaining: Option<Fx>,
    pub(crate) repairs_performed: u32,
    pub(crate) stabilizations_performed: u32,
    pub(crate) automation_used: bool,
    pub(crate) automation_trigger_count: u32,
    pub(crate) highest_heat: Fx,
    pub(crate) hottest_module_kind: Option<ModuleKind>,
    pub(crate) first_disabled_module_kind: Option<ModuleKind>,
    pub(crate) recent_action: Option<String>,
    pub(crate) recent_action_timer: Fx,
    pub(crate) recovered_raw_salvage: u32,
    pub(crate) processed_repair_charge: u32,
    pub(crate) consumed_repair_charge: u32,
    pub(crate) transfer_count: u32,
    pub(crate) processor_cycles: u32,
    pub(crate) logistics_bottleneck: Option<String>,
    pub(crate) logistics_automation_used: bool,
    pub(crate) lowest_player_oxygen: Fx,
    pub(crate) hostile_decompression_events: u32,
    pub(crate) player_ship_breached: bool,
    pub(crate) airlocks_cycled: u32,
    pub(crate) active_contract_id: Option<String>,
    pub(crate) contract_title: Option<String>,
    pub(crate) mission_briefing: Option<String>,
    pub(crate) opposition_summary: Option<String>,
    pub(crate) opposition_comms: Option<String>,
}
