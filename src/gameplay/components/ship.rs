use bevy::prelude::*;

use super::ResourceKind;
use crate::{
    helpers::{FixedVec2, Fx},
    ship::ModuleKind,
};

#[derive(Component, Clone)]
#[allow(dead_code)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InfrastructureRouteKind {
    Power,
    OxygenDuct,
    Resource(ResourceKind),
}

impl InfrastructureRouteKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::OxygenDuct => "oxygen duct",
            Self::Resource(ResourceKind::RawSalvage) => "raw salvage",
            Self::Resource(ResourceKind::RepairCharge) => "repair charge",
            Self::Resource(ResourceKind::Fuel) => "fuel",
            Self::Resource(ResourceKind::Ammunition) => "ammunition",
            Self::Resource(ResourceKind::Oxygen) => "oxygen resource",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct InfrastructureNetworkSummary {
    pub(crate) id: u32,
    pub(crate) kind: Option<InfrastructureRouteKind>,
    pub(crate) tile_count: u32,
    pub(crate) tiles: Vec<(i32, i32)>,
    pub(crate) attached_modules: Vec<u64>,
    pub(crate) supply: Fx,
    pub(crate) demand: Fx,
    pub(crate) reserve: Fx,
    pub(crate) reserve_capacity: Fx,
    pub(crate) flow: Fx,
    pub(crate) blockers: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct InfrastructureServiceStatus {
    pub(crate) route_kind: InfrastructureRouteKind,
    pub(crate) network_id: Option<u32>,
    pub(crate) service_coord: Option<(i32, i32)>,
    pub(crate) required: bool,
    pub(crate) blocked_reason: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ModuleInfrastructureStatus {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) power_network: Option<u32>,
    pub(crate) duct_network: Option<u32>,
    pub(crate) resource_networks: Vec<(ResourceKind, u32)>,
    pub(crate) service_statuses: Vec<InfrastructureServiceStatus>,
    pub(crate) powered: bool,
    pub(crate) power_required: bool,
    pub(crate) blocked_reason: Option<String>,
}

impl ModuleInfrastructureStatus {
    pub(crate) fn network_for_resource(&self, resource: ResourceKind) -> Option<u32> {
        self.resource_networks
            .iter()
            .find_map(|(kind, network_id)| (*kind == resource).then_some(*network_id))
    }
}

#[derive(Component, Clone, Debug, Default)]
pub(crate) struct ShipInfrastructureState {
    pub(crate) networks: Vec<InfrastructureNetworkSummary>,
    pub(crate) module_statuses: Vec<ModuleInfrastructureStatus>,
    pub(crate) strict_routing: bool,
}

impl ShipInfrastructureState {
    pub(crate) fn status_for_module(&self, module_id: u64) -> Option<&ModuleInfrastructureStatus> {
        self.module_statuses
            .iter()
            .find(|status| status.module_id == module_id)
    }

    pub(crate) fn network(&self, id: u32) -> Option<&InfrastructureNetworkSummary> {
        self.networks.iter().find(|network| network.id == id)
    }

    pub(crate) fn module_powered(&self, module_id: u64) -> bool {
        self.status_for_module(module_id)
            .map(|status| status.powered)
            .unwrap_or(false)
    }

    pub(crate) fn module_resource_network(
        &self,
        module_id: u64,
        resource: ResourceKind,
    ) -> Option<u32> {
        self.status_for_module(module_id)
            .and_then(|status| status.network_for_resource(resource))
    }

    pub(crate) fn route_status_label(&self, module_id: u64) -> String {
        let Some(status) = self.status_for_module(module_id) else {
            return "Infrastructure: no routed network".to_string();
        };
        let power = status
            .power_network
            .map(|id| format!("P{id}"))
            .unwrap_or_else(|| "P-".to_string());
        let resources = if status.resource_networks.is_empty() {
            "R-".to_string()
        } else {
            status
                .resource_networks
                .iter()
                .map(|(kind, id)| format!("{}:{id}", resource_label(*kind)))
                .collect::<Vec<_>>()
                .join(",")
        };
        let duct = status
            .duct_network
            .map(|id| format!("O2D{id}"))
            .unwrap_or_else(|| "O2D-".to_string());
        let state = status
            .blocked_reason
            .as_deref()
            .unwrap_or(if status.powered {
                "online"
            } else {
                "unpowered"
            });
        let service_issue = status
            .service_statuses
            .iter()
            .find_map(|service| service.blocked_reason.as_deref())
            .unwrap_or(state);
        format!("Infrastructure: service ports {power} {duct} {resources} [{service_issue}]")
    }
}

fn resource_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::RawSalvage => "raw",
        ResourceKind::RepairCharge => "repair",
        ResourceKind::Fuel => "fuel",
        ResourceKind::Ammunition => "ammo",
        ResourceKind::Oxygen => "oxygen",
    }
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
    pub(crate) junction_open: Option<bool>,
    pub(crate) valve_open: Option<bool>,
}

#[derive(Component, Clone)]
pub(crate) struct ShipDamageSensorState {
    pub(crate) recent_direction: FixedVec2,
    pub(crate) recent_distance: Fx,
    pub(crate) recent_intensity: Fx,
    pub(crate) recent_timer: Fx,
}

impl Default for ShipDamageSensorState {
    fn default() -> Self {
        Self {
            recent_direction: FixedVec2::zero(),
            recent_distance: Fx::from_num(0),
            recent_intensity: Fx::from_num(0),
            recent_timer: Fx::from_num(0),
        }
    }
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
            junction_open: None,
            valve_open: None,
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
