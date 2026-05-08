use bevy::prelude::*;

use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{components::*, helpers::*, systems::simulation::helpers::*},
};

pub(crate) fn fire_player_weapons(
    mut commands: Commands,
    balance: Res<BalanceConfig>,
    player_ship_query: Single<
        (
            &Children,
            &SimPosition,
            &SimRotation,
            &ShipPowerState,
            &ShipArchCommandState,
            &mut ShipWeaponState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    weapon_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &ModuleRuntimeState,
            &WeaponModule,
            &TurretCommandState,
            Option<&DestroyedModule>,
        ),
        With<WeaponModule>,
    >,
    mut storage_query: Query<(&ChildOf, &mut StorageModule)>,
) {
    let (children, ship_position, ship_rotation, power_state, arch_commands, mut weapon_state) =
        player_ship_query.into_inner();

    let fire_requested = (arch_commands.turret_auto_fire && !arch_commands.turret_fire_hold)
        || weapon_query
            .iter()
            .any(|(_, _, _, _, turret_state, destroyed)| {
                destroyed.is_none() && turret_state.fire_intent
            });
    if !fire_requested
        || !power_state.weapons_powered
        || weapon_state.turret_count == 0
        || weapon_state.cooldown_remaining > Fx::from_num(0)
    {
        return;
    }

    let mut fired_any = false;
    let mut cooldown_after_shot = Fx::from_num(0);
    for child in children.iter() {
        let Ok((_, weapon_module, runtime_state, weapon_stats, turret_state, destroyed)) =
            weapon_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() || runtime_state.is_disabled {
            continue;
        }
        let is_manual_turret = turret_state.fire_intent;
        if (arch_commands.turret_fire_hold || !arch_commands.turret_auto_fire) && !is_manual_turret
        {
            continue;
        }
        if is_manual_turret && !turret_state.fire_intent {
            continue;
        }
        if weapon_stats.requires_ammo
            && !consume_ship_resource(
                &mut storage_query,
                children,
                crate::gameplay::components::ResourceKind::Ammunition,
                weapon_stats.ammo_per_shot.max(1),
            )
        {
            continue;
        }
        let world_angle = ship_rotation.radians + turret_state.actual_angle;
        let projectile_velocity = facing_vector(world_angle)
            * Fx::from_num(balance.combat.projectile_speed)
            * weapon_stats.projectile_speed_multiplier;
        if projectile_velocity.is_near_zero() {
            continue;
        }
        let muzzle_offset = facing_vector(world_angle)
            * Fx::from_num(TILE_SIZE * balance.combat.muzzle_offset_tiles);
        let origin = ship_position.value
            + weapon_module.local_position.rotate(ship_rotation.radians)
            + muzzle_offset;
        spawn_projectile_entity(
            &mut commands,
            origin,
            projectile_velocity,
            &balance,
            ProjectileFaction::Player,
            weapon_stats.damage,
            Fx::from_num(0),
            Fx::from_num(0),
            if weapon_stats.requires_ammo {
                Color::srgb(0.98, 0.64, 0.24)
            } else {
                Color::srgb(0.98, 0.84, 0.30)
            },
        );
        fired_any = true;
        cooldown_after_shot = cooldown_after_shot.max(
            Fx::from_num(balance.combat.player_weapon_cooldown) * weapon_stats.cooldown_multiplier,
        );
    }

    if fired_any {
        weapon_state.cooldown_remaining = cooldown_after_shot.max(weapon_state.cooldown_duration);
    }
}
