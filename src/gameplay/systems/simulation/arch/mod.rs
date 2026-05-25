use bevy::prelude::*;
use cordic::{acos, asin, atan, atan2, cos, sin, sqrt, tan};

use crate::{
    gameplay::{
        components::{
            ArchComputerModule,
            ArchExecutionResult,
            ArchLogisticsPreference,
            DestroyedModule,
            DetectorModule,
            HostileTarget,
            JunctionCommandState,
            LumenExecutionResult,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            ProcessorModule,
            ReactorCommandState,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipArchCommandState,
            ShipAutomationMode,
            ShipAutomationState,
            ShipDamageSensorState,
            ShipInfrastructureState,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            StorageModule,
            TurretCommandState,
            ValveCommandState,
        },
        helpers::{Fx, fx_from_time_delta},
        systems::simulation::snapshots,
    },
    ship::{
        ModuleKind,
        arch::{ArchInstruction, ArchProgram, ArchProgramTemplate, ArchRegister, ArchValueRef},
        lumen::{LumenAspect, LumenInstruction, LumenOp, LumenProgram, LumenTarget},
    },
};

mod interpreter;
mod lumen;
mod systems;
#[cfg(test)]
mod tests;
mod types;

use interpreter::*;
pub(crate) use interpreter::{bool_fx, detector_axis, detector_metric};
use lumen::*;
pub(crate) use systems::{run_arch_automation, tick_recent_action_feedback};
use types::*;
pub(crate) use types::{ArchSnapshot, LumenSnapshot};
