use bevy::prelude::*;

use super::{FixedVec2, Fx, fx_ratio};
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::components::{ShipMovementModel, ShipPowerModel, ShipPowerState},
    ship::{ModuleKind, ModuleSpec, ShipDefinition, ShipModule},
};

pub(crate) fn module_local_translation(module: &ShipModule, center_x: f32, center_y: f32) -> Vec3 {
    Vec3::new(
        (module.grid_x as f32 - center_x) * TILE_SIZE,
        -((module.grid_y as f32) - center_y) * TILE_SIZE,
        1.0,
    )
}

pub(crate) fn module_local_position(module: &ShipModule, center_x: Fx, center_y: Fx) -> FixedVec2 {
    FixedVec2::new(
        (Fx::from_num(module.grid_x) - center_x) * Fx::from_num(TILE_SIZE),
        (center_y - Fx::from_num(module.grid_y)) * Fx::from_num(TILE_SIZE),
    )
}

pub(crate) fn module_integrity(kind: ModuleKind, variant: crate::ship::ModuleVariant) -> i32 {
    ModuleSpec::for_module(kind, variant).integrity
}

pub(crate) fn ship_movement_model(
    module_count: usize,
    engine_count: u32,
    balance: &BalanceConfig,
) -> ShipMovementModel {
    ship_movement_model_with_effective(
        module_count,
        engine_count,
        Fx::from_num(engine_count.max(1)),
        Fx::from_num(1),
        balance,
    )
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

pub(crate) fn ship_power_model(
    module_count: usize,
    reactor_count: u32,
    battery_count: u32,
    engine_count: u32,
    turret_count: u32,
    balance: &BalanceConfig,
) -> ShipPowerModel {
    ship_power_model_with_effective(
        module_count,
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
        Fx::from_num(reactor_count.max(1)),
        Fx::from_num(battery_count),
        Fx::from_num(battery_count.max(1)),
        Fx::from_num(battery_count.max(1)),
        Fx::from_num(engine_count),
        Fx::from_num(turret_count),
        balance,
    )
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

fn power_draw_for_requested_systems(
    power_model: &ShipPowerModel,
    throttle_demand: Fx,
    turn_input: Fx,
    weapon_demand: Fx,
) -> (Fx, Fx, Fx) {
    let throttle = throttle_demand.clamp(Fx::from_num(0), Fx::from_num(1));
    let steering_fraction =
        turn_input.abs().clamp(Fx::from_num(0), Fx::from_num(1)) * fx_ratio(2, 5);
    let engine_requested = power_model.engine_draw * throttle.max(steering_fraction);
    let weapon_requested =
        power_model.weapon_draw * weapon_demand.clamp(Fx::from_num(0), Fx::from_num(1));

    (power_model.passive_draw, weapon_requested, engine_requested)
}

pub(crate) fn update_ship_power_state(
    dt: Fx,
    throttle_demand: Fx,
    turn_input: Fx,
    weapon_demand: Fx,
    power_model: &ShipPowerModel,
    power_state: &mut ShipPowerState,
) {
    let (passive_draw, weapon_draw, engine_draw) =
        power_draw_for_requested_systems(power_model, throttle_demand, turn_input, weapon_demand);
    let requested_draw = passive_draw + weapon_draw + engine_draw;
    let mut effective_draw = requested_draw;
    let mut engine_power_ratio = if engine_draw > Fx::from_num(0) {
        Fx::from_num(1)
    } else {
        Fx::from_num(0)
    };

    let safe_dt = if dt > fx_ratio(1, 1000) {
        dt
    } else {
        fx_ratio(1, 1000)
    };
    let discharge_limit = (power_model.battery_discharge_rate * safe_dt).max(Fx::from_num(0));
    let available_energy = power_model.reactor_output + discharge_limit / safe_dt;

    let weapons_powered = if effective_draw > available_energy {
        effective_draw -= weapon_draw;
        false
    } else {
        weapon_draw > Fx::from_num(0)
    };

    if effective_draw > available_energy {
        let baseline_draw = effective_draw - engine_draw;
        let remaining_for_engines = (available_energy - baseline_draw).max(Fx::from_num(0));
        if engine_draw > Fx::from_num(0) {
            engine_power_ratio =
                (remaining_for_engines / engine_draw).clamp(Fx::from_num(0), Fx::from_num(1));
            effective_draw = baseline_draw + engine_draw * engine_power_ratio;
        } else {
            engine_power_ratio = Fx::from_num(0);
        }
    }

    let net_power = power_model.reactor_output - effective_draw;
    let charge_delta = if net_power >= Fx::from_num(0) {
        (net_power * dt).min(power_model.battery_charge_rate * dt)
    } else {
        (net_power * dt).max(-power_model.battery_discharge_rate * dt)
    };
    let new_stored_energy = (power_state.stored_energy + charge_delta)
        .clamp(Fx::from_num(0), power_model.battery_capacity);

    power_state.stored_energy = new_stored_energy;
    power_state.generation = power_model.reactor_output;
    power_state.draw = effective_draw;
    power_state.surplus = net_power;
    power_state.engine_power_ratio = engine_power_ratio;
    power_state.weapons_powered = weapons_powered;
    power_state.engines_powered = engine_power_ratio > Fx::from_num(0);
}

pub(crate) fn count_modules(ship: &ShipDefinition, kind: ModuleKind) -> u32 {
    ship.modules
        .iter()
        .filter(|module| module.kind == kind)
        .count() as u32
}
