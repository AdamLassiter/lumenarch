use super::{FixedVec2, Fx, fx_ratio};
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::components::{ShipMovementModel, ShipPowerModel},
    ship::{ModuleKind, ModuleSpec, ModuleVariant, ShipModule},
};

pub(crate) fn module_local_position(module: &ShipModule, center_x: Fx, center_y: Fx) -> FixedVec2 {
    FixedVec2::new(
        (Fx::from_num(module.grid_x) - center_x) * Fx::from_num(TILE_SIZE),
        (center_y - Fx::from_num(module.grid_y)) * Fx::from_num(TILE_SIZE),
    )
}

pub(crate) fn ship_grid_from_local_position(local_position: FixedVec2) -> (i32, i32) {
    (
        (local_position.x / Fx::from_num(TILE_SIZE)).to_num::<i32>(),
        (-local_position.y / Fx::from_num(TILE_SIZE)).to_num::<i32>(),
    )
}

pub(crate) fn module_integrity(kind: ModuleKind, variant: ModuleVariant) -> i32 {
    ModuleSpec::for_module(kind, variant).integrity
}

pub(crate) fn ship_movement_model_with_effective(
    module_count: usize,
    engine_count: u32,
    effective_engines: Fx,
    helm_multiplier: Fx,
    balance: &BalanceConfig,
) -> ShipMovementModel {
    let engine_scalar = effective_engines.max(fx_ratio(1, 4)).to_num::<f32>();
    let mass_factor = (module_count.max(1) as f32).sqrt();
    let helm_scalar = helm_multiplier.max(Fx::from_num(1)).to_num::<f32>();

    ShipMovementModel {
        engine_count,
        helm_multiplier,
        thrust_acceleration: Fx::from_num(
            balance.ship.thrust_base_acceleration * engine_scalar * helm_scalar / mass_factor,
        ),
        turn_speed: Fx::from_num(
            (balance.ship.turn_speed_base + balance.ship.turn_speed_per_engine * engine_scalar)
                * helm_scalar,
        ),
        max_speed: Fx::from_num(
            (balance.ship.max_speed_base + balance.ship.max_speed_per_engine * engine_scalar)
                * (0.9 + helm_scalar * 0.1),
        ),
        linear_damping: Fx::from_num(balance.ship.linear_damping),
        angular_damping: Fx::from_num(balance.ship.angular_damping),
    }
}

pub(crate) fn ship_power_model_with_effective(
    module_count: usize,
    _reactor_count: u32,
    battery_count: u32,
    engine_count: u32,
    turret_count: u32,
    effective_reactors: Fx,
    effective_batteries: Fx,
    effective_charge_rate: Fx,
    effective_discharge_rate: Fx,
    effective_engines: Fx,
    effective_turrets: Fx,
    balance: &BalanceConfig,
) -> ShipPowerModel {
    ShipPowerModel {
        reactor_output: effective_reactors * Fx::from_num(balance.ship.reactor_output_per_reactor),
        battery_capacity: effective_batteries.max(Fx::from_num(battery_count))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        battery_charge_rate: effective_charge_rate.max(Fx::from_num(1))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        battery_discharge_rate: effective_discharge_rate.max(Fx::from_num(1))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        passive_draw: Fx::from_num(
            balance.ship.passive_draw_base
                + module_count as f32 * balance.ship.passive_draw_per_module,
        ),
        engine_draw: effective_engines.max(Fx::from_num(engine_count))
            * Fx::from_num(balance.ship.engine_draw_per_engine),
        weapon_draw: effective_turrets.max(Fx::from_num(turret_count))
            * Fx::from_num(balance.ship.weapon_draw_per_turret),
        shield_draw: Fx::from_num(0),
    }
}
