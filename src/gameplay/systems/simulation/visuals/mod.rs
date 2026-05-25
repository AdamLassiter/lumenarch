use std::collections::HashMap;

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            ArenaBackdropLayer,
            BatteryPulseOverlay,
            CurrentStation,
            DecompressionAirLinesOverlay,
            DestroyedModule,
            ElectricalArcOverlay,
            EngineFlameOverlay,
            EquippedSuit,
            EvaThrusterOverlay,
            FabricatorDustOverlay,
            HeatFlameOverlay,
            HeldInteraction,
            InfrastructureRouteKind,
            Integrity,
            InteractionKind,
            LinearVelocity,
            ManipulatorModule,
            ModuleCondition,
            ModuleFieldEmitter,
            ModuleRuntimeState,
            ModuleWorkEffect,
            ModuleWorkProgressFill,
            ModuleWorkProgressRoot,
            ObservedLocalPlayerMarker,
            PlayerFocusedTile,
            PlayerMotionState,
            PlayerReferenceFrame,
            PlayerShip,
            PlayerSuit,
            ProcessorModule,
            ReactorCommandState,
            ReactorGlowOverlay,
            ResourceKind,
            RuntimeFoundationVisual,
            RuntimeShipModule,
            ServiceLinkOverlay,
            ShipAtmosphereState,
            ShipControlState,
            ShipInfrastructureState,
            ShipPowerState,
            ShipRoot,
            ShipSpeedLinesOverlay,
            ShipboardPlayer,
            SimPosition,
            SimRotation,
            SpaceBackdropLayer,
            TurretCommandState,
            TurretFlashOverlay,
            TurretFlashPulse,
            TurretTopSprite,
        },
        effects::{
            AirLinesMaterial,
            BatteryPulseMaterial,
            ElectricArcsMaterial,
            EngineFlameMaterial,
            FabricatorDustMaterial,
            ReactorGlowMaterial,
            SmallFlamesMaterial,
            SpaceBackdropMaterial,
            SpeedLinesMaterial,
            TurretFlashMaterial,
        },
        helpers::{Fx, component_service_coords, module_condition},
    },
    ship::ModuleKind,
    state::GameplayInfoPanelMode,
};

mod actor_effects;
mod debug;
mod effects;
mod overlays;
#[cfg(test)]
mod tests;

use actor_effects::*;
pub(crate) use actor_effects::{
    sync_backdrop_parallax,
    sync_eva_thruster_visuals,
    sync_module_work_effect_visuals,
};
pub(crate) use debug::draw_debug_overlay;
use debug::*;
pub(crate) use effects::{
    sync_engine_flame_visuals,
    sync_reactor_glow_visuals,
    sync_service_link_visuals,
    update_destroyed_module_visuals,
};
pub(crate) use overlays::{
    spawn_missing_effect_overlays,
    sync_battery_pulse_visuals,
    sync_fabricator_dust_visuals,
    sync_hazard_effect_visuals,
    sync_ship_environment_effect_visuals,
    sync_turret_flash_visuals,
};
