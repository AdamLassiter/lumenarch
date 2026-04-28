use bevy::prelude::*;

use super::super::helpers::{FixedVec2, Fx};
use crate::ship::arch::ArchProgram;
use crate::ship::ModuleKind;

#[derive(Component)]
pub(crate) struct RuntimeShipModule {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component)]
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

#[derive(Component)]
pub(crate) struct EngineModule;

#[derive(Component)]
pub(crate) struct WeaponModule;

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

#[derive(Component)]
pub(crate) struct RuntimeArchComputer {
    pub(crate) enabled: bool,
    pub(crate) instruction_budget: u32,
    pub(crate) program: ArchProgram,
    pub(crate) last_result: ArchExecutionResult,
}

#[derive(Component)]
pub(crate) struct TurretTopSprite;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleCondition {
    Healthy,
    Degraded,
    Disabled,
    Destroyed,
}

#[derive(Component)]
pub(crate) struct ModuleRuntimeState {
    pub(crate) current_heat: Fx,
    pub(crate) electrical_instability: Fx,
    pub(crate) sampled_heat: Fx,
    pub(crate) sampled_electrical: Fx,
    pub(crate) is_disabled: bool,
    pub(crate) was_disabled_last_frame: bool,
    pub(crate) needs_attention: bool,
    pub(crate) last_interaction_age: Fx,
}

#[derive(Component)]
pub(crate) struct ModuleFieldEmitter {
    pub(crate) heat_output: Fx,
    pub(crate) cooling_output: Fx,
    pub(crate) electrical_output: Fx,
    pub(crate) grounding_output: Fx,
}

#[derive(Component)]
pub(crate) struct DestroyedModule;
