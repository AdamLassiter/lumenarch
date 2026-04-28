use bevy::prelude::*;

use super::{fx_ratio, FixedVec2, Fx};
use crate::client::{TILE_SIZE};
use crate::client::gameplay::components::{ShipMovementModel, ShipPowerModel, ShipPowerState};
use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

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

pub(crate) fn module_integrity(kind: ModuleKind) -> i32 {
    match kind {
        ModuleKind::Hull | ModuleKind::HullCorner => 12,
        ModuleKind::Core => 20,
        ModuleKind::Cockpit => 10,
        ModuleKind::Computer => 8,
        ModuleKind::Reactor => 14,
        ModuleKind::Engine => 10,
        ModuleKind::Cargo => 10,
        ModuleKind::Battery => 8,
        ModuleKind::Airlock => 8,
        ModuleKind::Turret => 8,
        ModuleKind::Processor => 8,
        ModuleKind::Interior => 6,
    }
}

pub(crate) fn ship_movement_model(module_count: usize, engine_count: u32) -> ShipMovementModel {
    ship_movement_model_with_effective(
        module_count,
        engine_count,
        Fx::from_num(engine_count.max(1)),
    )
}

pub(crate) fn ship_movement_model_with_effective(
    module_count: usize,
    engine_count: u32,
    effective_engines: Fx,
) -> ShipMovementModel {
    let engine_scalar = effective_engines.max(fx_ratio(1, 4)).to_num::<f32>();
    let mass_factor = (module_count.max(1) as f32).sqrt();

    ShipMovementModel {
        engine_count,
        thrust_acceleration: Fx::from_num(260.0 * engine_scalar / mass_factor),
        turn_speed: Fx::from_num(1.2 + 0.35 * engine_scalar),
        max_speed: Fx::from_num(110.0 + 24.0 * engine_scalar),
        linear_damping: Fx::from_num(0.9),
        angular_damping: Fx::from_num(5.0),
    }
}

pub(crate) fn ship_power_model(
    module_count: usize,
    reactor_count: u32,
    battery_count: u32,
    engine_count: u32,
    turret_count: u32,
) -> ShipPowerModel {
    ship_power_model_with_effective(
        module_count,
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
        Fx::from_num(reactor_count.max(1)),
        Fx::from_num(battery_count),
        Fx::from_num(engine_count),
        Fx::from_num(turret_count),
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
    effective_engines: Fx,
    effective_turrets: Fx,
) -> ShipPowerModel {
    ShipPowerModel {
        reactor_output: effective_reactors * Fx::from_num(8),
        battery_capacity: effective_batteries.max(Fx::from_num(battery_count)) * Fx::from_num(24),
        passive_draw: Fx::from_num(1.0 + module_count as f32 * 0.08),
        engine_draw: effective_engines.max(Fx::from_num(engine_count)) * Fx::from_num(2.5),
        weapon_draw: effective_turrets.max(Fx::from_num(turret_count)) * Fx::from_num(2),
    }
}

fn power_draw_for_requested_systems(
    power_model: &ShipPowerModel,
    throttle_demand: Fx,
    turn_input: Fx,
) -> (Fx, Fx, Fx) {
    let throttle = throttle_demand.clamp(Fx::from_num(0), Fx::from_num(1));
    let steering_fraction = turn_input.abs().clamp(Fx::from_num(0), Fx::from_num(1)) * fx_ratio(2, 5);
    let engine_requested =
        power_model.engine_draw * throttle.max(steering_fraction);

    (power_model.passive_draw, power_model.weapon_draw, engine_requested)
}

pub(crate) fn update_ship_power_state(
    dt: Fx,
    throttle_demand: Fx,
    turn_input: Fx,
    power_model: &ShipPowerModel,
    power_state: &mut ShipPowerState,
) {
    let (passive_draw, weapon_draw, engine_draw) =
        power_draw_for_requested_systems(power_model, throttle_demand, turn_input);
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
    let available_energy = power_model.reactor_output + power_state.stored_energy / safe_dt;

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
    let new_stored_energy = (power_state.stored_energy + net_power * dt)
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
