use bevy::prelude::*;

use super::{
    super::helpers::{FixedVec2, Fx},
    logistics::ResourceKind,
};
use crate::ship::{ModuleKind, ModuleVariant, arch::ArchProgram, lumen::LumenProgram};

#[derive(Component, Clone)]
pub(crate) struct RuntimeShipModule {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) rotation_quadrants: u8,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component)]
pub(crate) struct HostileShipModule;

#[derive(Component, Clone)]
pub(crate) struct Integrity {
    pub(crate) current: i32,
    pub(crate) max: i32,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct PowerProducer {
    pub(crate) output: i32,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct PowerConsumer {
    pub(crate) draw: i32,
}

#[derive(Component, Clone)]
pub(crate) struct EngineModule {
    pub(crate) thrust_multiplier: Fx,
}

#[derive(Component, Clone)]
pub(crate) struct WeaponModule {
    pub(crate) damage: i32,
    pub(crate) requires_ammo: bool,
    pub(crate) ammo_per_shot: u32,
    pub(crate) projectile_speed_multiplier: Fx,
    pub(crate) cooldown_multiplier: Fx,
    pub(crate) automation_difficulty: Fx,
}

#[derive(Component)]
pub(crate) struct ArchComputerModule;

#[derive(Clone, Debug, Default)]
pub(crate) struct ArchExecutionResult {
    pub(crate) executed: u32,
    pub(crate) budget: u32,
    pub(crate) recent_writes: Vec<String>,
    pub(crate) halted_reason: Option<String>,
    pub(crate) program_name: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LumenExecutionResult {
    pub(crate) resolved_targets: u32,
    pub(crate) recent_effects: Vec<String>,
    pub(crate) halted_reason: Option<String>,
    pub(crate) program_name: String,
}

#[derive(Component, Clone)]
pub(crate) struct RuntimeArchComputer {
    pub(crate) enabled: bool,
    pub(crate) instruction_budget: u32,
    pub(crate) program: ArchProgram,
    pub(crate) last_result: ArchExecutionResult,
    pub(crate) lumen_program: LumenProgram,
    pub(crate) last_lumen_result: LumenExecutionResult,
}

#[derive(Component)]
pub(crate) struct TurretTopSprite;

#[derive(Component, Clone)]
pub(crate) struct TurretCommandState {
    pub(crate) desired_angle: Fx,
    pub(crate) actual_angle: Fx,
    pub(crate) fire_intent: bool,
}

#[derive(Component, Clone)]
pub(crate) struct ReactorCommandState {
    pub(crate) variant: ModuleVariant,
    pub(crate) reaction_rate: Fx,
    pub(crate) turbine_load: Fx,
    pub(crate) confinement: Fx,
    pub(crate) power_output: Fx,
    pub(crate) fuel_remaining: Fx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessorRecipe {
    RepairCharge,
    Ammunition,
    Fuel,
}

impl ProcessorRecipe {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::RepairCharge => "Repair Charge",
            Self::Ammunition => "Ammunition",
            Self::Fuel => "Fuel",
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct StorageCommandState {
    pub(crate) allow_intake: bool,
}

#[derive(Component, Clone)]
pub(crate) struct AirlockCommandState {
    pub(crate) open: bool,
}

#[derive(Component, Clone)]
pub(crate) struct ManipulatorCommandState {
    pub(crate) manual_mode: bool,
    pub(crate) transfer_enabled: bool,
    pub(crate) source_module_id: Option<u64>,
    pub(crate) target_module_id: Option<u64>,
    pub(crate) resource_kind: ResourceKind,
}

#[derive(Component, Clone)]
pub(crate) struct ProcessorCommandState {
    pub(crate) selected_recipe: ProcessorRecipe,
    pub(crate) enabled: bool,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct ShieldCommandState {
    pub(crate) desired_angle: Fx,
    pub(crate) width: Fx,
    pub(crate) strength: Fx,
    pub(crate) max_strength: Fx,
    pub(crate) regen_rate: Fx,
    pub(crate) directional: bool,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct DetectorCommandState {
    pub(crate) show_targets: bool,
    pub(crate) show_heat: bool,
    pub(crate) show_electrical: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DroneTask {
    Idle,
    Salvage,
    Repair,
    Defend,
}

#[allow(dead_code)]
impl DroneTask {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Salvage => "Salvage",
            Self::Repair => "Repair",
            Self::Defend => "Defend",
        }
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct DroneStationCommandState {
    pub(crate) selected_task: DroneTask,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct MemoryBankState {
    pub(crate) words: [i32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleCondition {
    Healthy,
    Degraded,
    Disabled,
    Destroyed,
}

#[derive(Component, Clone)]
pub(crate) struct ModuleRuntimeState {
    pub(crate) current_heat: Fx,
    pub(crate) electrical_instability: Fx,
    pub(crate) sampled_heat: Fx,
    pub(crate) sampled_electrical: Fx,
    pub(crate) is_disabled: bool,
    pub(crate) was_disabled_last_frame: bool,
    pub(crate) needs_attention: bool,
    pub(crate) extracted: bool,
    pub(crate) last_interaction_age: Fx,
}

#[derive(Component, Clone)]
pub(crate) struct ModuleFieldEmitter {
    pub(crate) heat_output: Fx,
    pub(crate) cooling_output: Fx,
    pub(crate) electrical_output: Fx,
    pub(crate) grounding_output: Fx,
}

#[derive(Component)]
pub(crate) struct DestroyedModule;
