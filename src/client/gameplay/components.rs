use bevy::prelude::*;

use crate::ship::ModuleKind;

#[derive(Component)]
pub(crate) struct PlayerShip;

#[derive(Component)]
pub(crate) struct ShipRoot;

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct RuntimeShipModule {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
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
    pub(crate) velocity: Vec2,
    pub(crate) remaining_life: f32,
    pub(crate) damage: i32,
    pub(crate) faction: ProjectileFaction,
}

#[derive(Component)]
pub(crate) struct HostileTarget;

#[derive(Component)]
pub(crate) struct HostileTurretPlatform;

#[derive(Component)]
pub(crate) struct HostileWeaponState {
    pub(crate) cooldown_remaining: f32,
    pub(crate) cooldown_duration: f32,
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
    pub(crate) value: Vec2,
}

#[derive(Component)]
pub(crate) struct AngularVelocity {
    pub(crate) radians_per_second: f32,
}

#[derive(Component)]
pub(crate) struct ShipMovementModel {
    pub(crate) engine_count: u32,
    pub(crate) thrust_acceleration: f32,
    pub(crate) turn_speed: f32,
    pub(crate) max_speed: f32,
    pub(crate) linear_damping: f32,
    pub(crate) angular_damping: f32,
}

#[derive(Component)]
pub(crate) struct ShipPowerModel {
    pub(crate) reactor_output: f32,
    pub(crate) battery_capacity: f32,
    pub(crate) passive_draw: f32,
    pub(crate) engine_draw: f32,
    pub(crate) weapon_draw: f32,
}

#[derive(Component)]
pub(crate) struct ShipPowerState {
    pub(crate) stored_energy: f32,
    pub(crate) generation: f32,
    pub(crate) draw: f32,
    pub(crate) surplus: f32,
    pub(crate) engine_power_ratio: f32,
    pub(crate) weapons_powered: bool,
    pub(crate) engines_powered: bool,
}

#[derive(Component, Default)]
pub(crate) struct ShipControlState {
    pub(crate) thrust_active: bool,
    pub(crate) turn_input: f32,
    pub(crate) fire_pressed: bool,
}

#[derive(Component)]
pub(crate) struct ShipWeaponState {
    pub(crate) turret_count: u32,
    pub(crate) cooldown_remaining: f32,
    pub(crate) cooldown_duration: f32,
}

#[derive(Component)]
pub(crate) struct MissionState {
    pub(crate) failed: bool,
    pub(crate) failure_reason: Option<String>,
    pub(crate) completed: bool,
    pub(crate) completion_reason: Option<String>,
    pub(crate) salvage_collected: bool,
    pub(crate) salvage_scrap_awarded: u32,
    pub(crate) return_delay_remaining: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectileFaction {
    Player,
    Hostile,
}
