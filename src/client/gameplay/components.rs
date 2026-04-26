use bevy::prelude::*;

use crate::ship::ModuleKind;

use super::helpers::{FixedVec2, Fx};

#[derive(Component)]
pub(crate) struct PlayerShip;

#[derive(Component)]
pub(crate) struct ShipRoot;

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct RuntimeShipModule {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct Integrity {
    pub(crate) current: i32,
    pub(crate) max: i32,
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct PowerProducer {
    pub(crate) output: i32,
}

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct PowerConsumer {
    pub(crate) draw: i32,
}

#[derive(Component)]
pub(crate) struct EngineModule;

#[derive(Component)]
pub(crate) struct WeaponModule;

#[derive(Component)]
pub(crate) struct Projectile {
    pub(crate) velocity: FixedVec2,
    pub(crate) remaining_life: Fx,
    pub(crate) damage: i32,
    pub(crate) faction: ProjectileFaction,
}

#[derive(Component)]
pub(crate) struct HostileTarget;

#[derive(Component)]
pub(crate) struct HostileTurretPlatform;

#[derive(Component)]
pub(crate) struct HostileWeaponState {
    pub(crate) cooldown_remaining: Fx,
    pub(crate) cooldown_duration: Fx,
}

#[derive(Component)]
pub(crate) struct SalvagePickup {
    pub(crate) scrap_value: u32,
}

#[derive(Component)]
pub(crate) struct SalvageWreck;

#[derive(Component)]
pub(crate) struct CollectedSalvage;

#[derive(Component)]
pub(crate) struct DestroyedModule;

#[derive(Component)]
pub(crate) struct LinearVelocity {
    pub(crate) value: FixedVec2,
}

#[derive(Component)]
pub(crate) struct AngularVelocity {
    pub(crate) radians_per_second: Fx,
}

#[derive(Component)]
pub(crate) struct SimPosition {
    pub(crate) value: FixedVec2,
}

#[derive(Component)]
pub(crate) struct SimRotation {
    pub(crate) radians: Fx,
}

#[derive(Component)]
pub(crate) struct ShipMovementModel {
    pub(crate) engine_count: u32,
    pub(crate) thrust_acceleration: Fx,
    pub(crate) turn_speed: Fx,
    pub(crate) max_speed: Fx,
    pub(crate) linear_damping: Fx,
    pub(crate) angular_damping: Fx,
}

#[derive(Component)]
pub(crate) struct ShipPowerModel {
    pub(crate) reactor_output: Fx,
    pub(crate) battery_capacity: Fx,
    pub(crate) passive_draw: Fx,
    pub(crate) engine_draw: Fx,
    pub(crate) weapon_draw: Fx,
}

#[derive(Component)]
pub(crate) struct ShipPowerState {
    pub(crate) stored_energy: Fx,
    pub(crate) generation: Fx,
    pub(crate) draw: Fx,
    pub(crate) surplus: Fx,
    pub(crate) engine_power_ratio: Fx,
    pub(crate) weapons_powered: bool,
    pub(crate) engines_powered: bool,
}

#[derive(Component, Default)]
pub(crate) struct ShipControlState {
    pub(crate) thrust_active: bool,
    pub(crate) turn_input: Fx,
    pub(crate) fire_pressed: bool,
}

#[derive(Component)]
pub(crate) struct ShipWeaponState {
    pub(crate) turret_count: u32,
    pub(crate) cooldown_remaining: Fx,
    pub(crate) cooldown_duration: Fx,
}

#[derive(Component)]
pub(crate) struct MissionState {
    pub(crate) failed: bool,
    pub(crate) failure_reason: Option<String>,
    pub(crate) encounter_cleared: bool,
    pub(crate) completed: bool,
    pub(crate) completion_reason: Option<String>,
    pub(crate) salvage_collected: bool,
    pub(crate) salvage_scrap_awarded: u32,
    pub(crate) return_delay_remaining: Option<Fx>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectileFaction {
    Player,
    Hostile,
}
