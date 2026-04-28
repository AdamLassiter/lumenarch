use bevy::prelude::*;

use super::super::helpers::{FixedVec2, Fx};

#[derive(Component)]
pub(crate) struct Projectile {
    pub(crate) velocity: FixedVec2,
    pub(crate) remaining_life: Fx,
    pub(crate) damage: i32,
    pub(crate) faction: ProjectileFaction,
    pub(crate) heat_damage: Fx,
    pub(crate) electrical_damage: Fx,
}

#[derive(Component)]
pub(crate) struct HostileTarget;

#[derive(Component)]
pub(crate) struct HostileTurretPlatform;

#[derive(Component)]
pub(crate) struct HostileWeaponState {
    pub(crate) cooldown_remaining: Fx,
    pub(crate) cooldown_duration: Fx,
    pub(crate) heat_damage: Fx,
    pub(crate) electrical_damage: Fx,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectileFaction {
    Player,
    Hostile,
}
