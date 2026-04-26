use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

use super::{
    components::{
        MissionState, Projectile, ProjectileFaction, SalvagePickup, SalvageWreck, ShipMovementModel,
        ShipPowerModel, ShipPowerState,
    },
    ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, PROJECTILE_LIFETIME,
};
use super::super::{state::PlayingCleanup, TILE_SIZE};

pub(super) fn gameplay_status_line(ship: &ShipDefinition) -> String {
    format!(
        "Ship: {}\nModules: {}\nRuntime arena bootstrap active\nPress Tab or use the button to return",
        ship.name,
        ship.modules.len()
    )
}

pub(super) fn mission_status_line(mission_state: &MissionState) -> &str {
    if mission_state.failed {
        mission_state
            .failure_reason
            .as_deref()
            .unwrap_or("Mission failed")
    } else if mission_state.salvage_collected {
        "Salvage recovered"
    } else if mission_state.completed {
        mission_state
            .completion_reason
            .as_deref()
            .unwrap_or("Mission complete")
    } else {
        "Operational"
    }
}

pub(super) fn mission_return_line(mission_state: &MissionState) -> Option<String> {
    mission_state.return_delay_remaining.map(|seconds| {
        format!("returning to editor in {:.1}s", seconds.max(0.0))
    })
}

pub(super) fn salvage_status_line(
    ship_position: Vec2,
    mission_state: &MissionState,
    salvage_query: &Query<(&Transform, &SalvagePickup), (With<SalvageWreck>, Without<super::components::CollectedSalvage>)>,
) -> String {
    if mission_state.salvage_collected {
        return format!("recovered {} scrap", mission_state.salvage_scrap_awarded);
    }

    if !mission_state.completed || mission_state.failed {
        return "secure the encounter first".to_string();
    }

    for (transform, salvage) in salvage_query.iter() {
        let distance = ship_position.distance(transform.translation.truncate());
        if distance <= super::SALVAGE_PICKUP_RADIUS {
            return format!("press F for {} scrap", salvage.scrap_value);
        }
    }

    "find the salvage wreck".to_string()
}

pub(super) fn module_local_translation(module: &ShipModule, center_x: f32, center_y: f32) -> Vec3 {
    Vec3::new(
        (module.grid_x as f32 - center_x) * TILE_SIZE,
        -((module.grid_y as f32) - center_y) * TILE_SIZE,
        1.0,
    )
}

pub(super) fn module_integrity(kind: ModuleKind) -> i32 {
    match kind {
        ModuleKind::Hull | ModuleKind::HullCorner => 12,
        ModuleKind::Core => 20,
        ModuleKind::Cockpit => 10,
        ModuleKind::Reactor => 14,
        ModuleKind::Engine => 10,
        ModuleKind::Cargo => 10,
        ModuleKind::Battery => 8,
        ModuleKind::Airlock => 8,
        ModuleKind::Turret => 8,
        ModuleKind::Interior => 6,
    }
}

pub(super) fn ship_movement_model(module_count: usize, engine_count: u32) -> ShipMovementModel {
    let effective_engines = engine_count.max(1) as f32;
    let mass_factor = (module_count.max(1) as f32).sqrt();

    ShipMovementModel {
        engine_count,
        thrust_acceleration: 260.0 * effective_engines / mass_factor,
        turn_speed: 1.5 + (0.25 * effective_engines),
        max_speed: 120.0 + 28.0 * effective_engines,
        linear_damping: 0.9,
        angular_damping: 5.0,
    }
}

fn power_draw_for_requested_systems(
    power_model: &ShipPowerModel,
    thrust_active: bool,
    turn_input: f32,
) -> (f32, f32, f32) {
    let engine_requested = if thrust_active || turn_input.abs() > f32::EPSILON {
        power_model.engine_draw
    } else {
        0.0
    };

    (
        power_model.passive_draw,
        power_model.weapon_draw,
        engine_requested,
    )
}

pub(super) fn update_ship_power_state(
    dt: f32,
    thrust_active: bool,
    turn_input: f32,
    power_model: &ShipPowerModel,
    power_state: &mut ShipPowerState,
) {
    let (passive_draw, weapon_draw, engine_draw) =
        power_draw_for_requested_systems(power_model, thrust_active, turn_input);
    let requested_draw = passive_draw + weapon_draw + engine_draw;
    let mut effective_draw = requested_draw;
    let weapons_powered;
    let mut engine_power_ratio = if engine_draw > 0.0 { 1.0 } else { 0.0 };

    let available_energy = power_model.reactor_output + power_state.stored_energy / dt.max(0.001);

    if effective_draw > available_energy {
        effective_draw -= weapon_draw;
        weapons_powered = false;
    } else {
        weapons_powered = weapon_draw > f32::EPSILON;
    }

    if effective_draw > available_energy {
        let baseline_draw = effective_draw - engine_draw;
        let remaining_for_engines = (available_energy - baseline_draw).max(0.0);
        if engine_draw > 0.0 {
            engine_power_ratio = (remaining_for_engines / engine_draw).clamp(0.0, 1.0);
            effective_draw = baseline_draw + engine_draw * engine_power_ratio;
        } else {
            engine_power_ratio = 0.0;
        }
    }

    let net_power = power_model.reactor_output - effective_draw;
    let new_stored_energy =
        (power_state.stored_energy + net_power * dt).clamp(0.0, power_model.battery_capacity);

    power_state.stored_energy = new_stored_energy;
    power_state.generation = power_model.reactor_output;
    power_state.draw = effective_draw;
    power_state.surplus = net_power;
    power_state.engine_power_ratio = engine_power_ratio;
    power_state.weapons_powered = weapons_powered;
    power_state.engines_powered = engine_power_ratio > 0.0;
}

pub(super) fn ship_power_model(
    module_count: usize,
    reactor_count: u32,
    battery_count: u32,
    engine_count: u32,
    turret_count: u32,
) -> ShipPowerModel {
    let reactor_output = reactor_count as f32 * 8.0;
    let battery_capacity = battery_count as f32 * 24.0;
    let passive_draw = 1.0 + module_count as f32 * 0.08;
    let engine_draw = engine_count as f32 * 2.5;
    let weapon_draw = turret_count as f32 * 2.0;

    ShipPowerModel {
        reactor_output,
        battery_capacity,
        passive_draw,
        engine_draw,
        weapon_draw,
    }
}

pub(super) fn count_modules(ship: &ShipDefinition, kind: ModuleKind) -> u32 {
    ship.modules
        .iter()
        .filter(|module| module.kind == kind)
        .count() as u32
}

pub(super) fn spawn_player_projectile(commands: &mut Commands, origin: Vec2, velocity: Vec2) {
    spawn_projectile_entity(
        commands,
        origin,
        velocity,
        ProjectileFaction::Player,
        2,
        Color::srgb(0.98, 0.84, 0.30),
    );
}

pub(super) fn spawn_projectile_entity(
    commands: &mut Commands,
    origin: Vec2,
    velocity: Vec2,
    faction: ProjectileFaction,
    damage: i32,
    color: Color,
) {
    commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 6.0)),
        Transform {
            translation: origin.extend(2.0),
            rotation: Quat::from_rotation_z(-velocity.to_angle() + FRAC_PI_2),
            ..default()
        },
        Projectile {
            velocity,
            remaining_life: PROJECTILE_LIFETIME,
            damage,
            faction,
        },
        PlayingCleanup,
    ));
}

pub(super) fn is_inside_arena(translation: Vec3) -> bool {
    let arena_half_w = ARENA_WIDTH_TILES as f32 * TILE_SIZE * 0.5;
    let arena_half_h = ARENA_HEIGHT_TILES as f32 * TILE_SIZE * 0.5;

    translation.x >= -arena_half_w
        && translation.x <= arena_half_w
        && translation.y >= -arena_half_h
        && translation.y <= arena_half_h
}

pub(super) fn damp_scalar(value: f32, damping: f32, dt: f32) -> f32 {
    value * (1.0 / (1.0 + damping * dt))
}

pub(super) fn damp_vec2(value: Vec2, damping: f32, dt: f32) -> Vec2 {
    value * (1.0 / (1.0 + damping * dt))
}

pub(super) fn clamp_ship_to_arena(transform: &mut Transform) {
    let arena_half_w = ARENA_WIDTH_TILES as f32 * TILE_SIZE * 0.5 - TILE_SIZE;
    let arena_half_h = ARENA_HEIGHT_TILES as f32 * TILE_SIZE * 0.5 - TILE_SIZE;
    transform.translation.x = transform.translation.x.clamp(-arena_half_w, arena_half_w);
    transform.translation.y = transform.translation.y.clamp(-arena_half_h, arena_half_h);
}

pub(super) fn sprite_path_for_kind(kind: ModuleKind) -> String {
    format!("tiles/{}.png", kind.as_str())
}
