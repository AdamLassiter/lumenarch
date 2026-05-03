use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            AngularVelocity,
            DestroyedModule,
            HostileShip,
            HostileShipAi,
            HostileShipModule,
            HostileTarget,
            HostileTurretPlatform,
            HostileWeaponState,
            Integrity,
            LinearVelocity,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            Projectile,
            ProjectileFaction,
            RuntimeShipModule,
            ShieldCommandState,
            ShipArchCommandState,
            ShipMovementModel,
            ShipPowerModel,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            SimPosition,
            SimRotation,
            StorageModule,
            TurretCommandState,
            WeaponModule,
        },
        helpers::{
            Fx,
            angle_from_vector,
            clamp_position_to_arena,
            damp_scalar,
            damp_vec2,
            facing_vector,
            fixed_radius_sq,
            fx_from_time_delta,
            is_inside_arena,
            module_effectiveness,
            render_translation,
            ship_movement_model_with_effective,
            ship_power_model_with_effective,
            spawn_projectile_entity,
            update_ship_power_state,
            wrap_radians,
        },
    },
    ship::ModuleKind,
    state::PlayingCleanup,
};

mod collisions;
mod helpers;
mod hostile;
mod player;
mod projectiles;

pub(crate) use collisions::handle_ship_collisions;
pub(crate) use hostile::{
    aim_hostile_turrets,
    drive_hostile_ships,
    fire_hostile_ship_weapons,
    fire_hostile_targets,
    integrate_hostile_ship_motion,
    sync_hostile_ship_state,
};
pub(crate) use player::fire_player_weapons;
pub(crate) use projectiles::{advance_projectiles, handle_projectile_hits};
