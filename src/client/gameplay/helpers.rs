use std::{
    f32::consts::FRAC_PI_2,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use bevy::prelude::*;
use cordic::{atan2, cos, sin};
use fixed::{types::extra::U16, FixedI32, FixedI64};

use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

use super::{
    components::{
        Integrity, InteractionKind, MissionState, ModuleCondition, ModuleFieldEmitter,
        ModuleRuntimeState, Projectile, ProjectileFaction, SalvagePickup,
        SalvageWreck, ShipMovementModel, ShipPowerModel, ShipPowerState, SimPosition,
    },
    ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, PROJECTILE_LIFETIME,
};
use super::super::{state::PlayingCleanup, TILE_SIZE};

pub(crate) type Fx = FixedI32<U16>;
pub(crate) type WideFx = FixedI64<U16>;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FixedVec2 {
    pub(crate) x: Fx,
    pub(crate) y: Fx,
}

impl FixedVec2 {
    pub(crate) fn new(x: Fx, y: Fx) -> Self {
        Self { x, y }
    }

    pub(crate) fn zero() -> Self {
        Self::default()
    }

    pub(crate) fn from_num(x: impl ToFixed, y: impl ToFixed) -> Self {
        Self::new(Fx::from_num(x.to_f64()), Fx::from_num(y.to_f64()))
    }

    pub(crate) fn from_vec2(value: Vec2) -> Self {
        Self::new(Fx::from_num(value.x), Fx::from_num(value.y))
    }

    pub(crate) fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x.to_num::<f32>(), self.y.to_num::<f32>())
    }

    pub(crate) fn length_sq(self) -> WideFx {
        widen(self.x) * widen(self.x) + widen(self.y) * widen(self.y)
    }

    pub(crate) fn length(self) -> Fx {
        Fx::from_num(self.length_sq().to_num::<f64>().sqrt())
    }

    pub(crate) fn distance_sq(self, other: Self) -> WideFx {
        (self - other).length_sq()
    }

    pub(crate) fn is_near_zero(self) -> bool {
        self.length_sq() <= wide_ratio(1, 4096)
    }

    pub(crate) fn normalized_or_zero(self) -> Self {
        let length = self.length();
        if length <= Fx::from_num(0) {
            return Self::zero();
        }
        self * (Fx::from_num(1) / length)
    }

    pub(crate) fn clamp_length(self, max_length: Fx) -> Self {
        let length_sq = self.length_sq();
        let max_sq = widen(max_length) * widen(max_length);
        if length_sq <= max_sq {
            return self;
        }

        self.normalized_or_zero() * max_length
    }

    pub(crate) fn rotate(self, radians: Fx) -> Self {
        let radians = wrap_radians(radians);
        let sin_theta = sin(radians);
        let cos_theta = cos(radians);
        Self::new(
            self.x * cos_theta - self.y * sin_theta,
            self.x * sin_theta + self.y * cos_theta,
        )
    }
}

impl Add for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for FixedVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for FixedVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Fx> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: Fx) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<Fx> for FixedVec2 {
    fn mul_assign(&mut self, rhs: Fx) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

pub(crate) trait ToFixed {
    fn to_f64(self) -> f64;
}

impl ToFixed for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToFixed for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToFixed for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

pub(crate) fn fx_ratio(num: i32, den: i32) -> Fx {
    Fx::from_num(num) / Fx::from_num(den.max(1))
}

pub(crate) fn wide_ratio(num: i32, den: i32) -> WideFx {
    WideFx::from_num(num) / WideFx::from_num(den.max(1))
}

pub(crate) fn widen(value: Fx) -> WideFx {
    WideFx::from_num(value)
}

pub(crate) fn fx_from_time_delta(time: &Time) -> Fx {
    Fx::from_num(time.delta_secs())
}

pub(crate) fn wrap_radians(phi: Fx) -> Fx {
    let pi = Fx::PI;
    let tau = 2 * Fx::PI;
    let mut angle = phi;

    while angle < -pi {
        angle += tau;
    }
    while pi <= angle {
        angle -= tau;
    }
    angle
}

pub(crate) fn format_fx0(value: Fx) -> String {
    format!("{:.0}", value.to_num::<f32>())
}

pub(crate) fn format_fx1(value: Fx) -> String {
    format!("{:.1}", value.to_num::<f32>())
}

pub(crate) fn format_fx2(value: Fx) -> String {
    format!("{:.2}", value.to_num::<f32>())
}

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
    } else if mission_state.encounter_cleared && !mission_state.salvage_collected {
        "Encounter cleared - recover salvage"
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
        format!("returning to editor in {:.1}s", seconds.to_num::<f32>().max(0.0))
    })
}

pub(super) fn salvage_status_line(
    ship_position: FixedVec2,
    mission_state: &MissionState,
    salvage_query: &Query<
        (&SimPosition, &SalvagePickup),
        (With<SalvageWreck>, Without<super::components::CollectedSalvage>),
    >,
) -> String {
    if mission_state.salvage_collected {
        return format!("recovered {} scrap", mission_state.salvage_scrap_awarded);
    }

    if !mission_state.encounter_cleared || mission_state.failed {
        return "secure the encounter first".to_string();
    }

    let pickup_radius_sq = fixed_radius_sq(super::SALVAGE_PICKUP_RADIUS);
    for (position, salvage) in salvage_query.iter() {
        if ship_position.distance_sq(position.value) <= pickup_radius_sq {
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

pub(super) fn module_local_position(module: &ShipModule, center_x: Fx, center_y: Fx) -> FixedVec2 {
    FixedVec2::new(
        (Fx::from_num(module.grid_x) - center_x) * Fx::from_num(TILE_SIZE),
        (center_y - Fx::from_num(module.grid_y)) * Fx::from_num(TILE_SIZE),
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
    ship_movement_model_with_effective(module_count, engine_count, Fx::from_num(engine_count.max(1)))
}

pub(super) fn ship_movement_model_with_effective(
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

pub(super) fn ship_power_model(
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

pub(super) fn ship_power_model_with_effective(
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
    thrust_active: bool,
    turn_input: Fx,
) -> (Fx, Fx, Fx) {
    let engine_requested = if thrust_active || turn_input != Fx::from_num(0) {
        power_model.engine_draw
    } else {
        Fx::from_num(0)
    };

    (power_model.passive_draw, power_model.weapon_draw, engine_requested)
}

pub(super) fn update_ship_power_state(
    dt: Fx,
    thrust_active: bool,
    turn_input: Fx,
    power_model: &ShipPowerModel,
    power_state: &mut ShipPowerState,
) {
    let (passive_draw, weapon_draw, engine_draw) =
        power_draw_for_requested_systems(power_model, thrust_active, turn_input);
    let requested_draw = passive_draw + weapon_draw + engine_draw;
    let mut effective_draw = requested_draw;
    let weapons_powered;
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

    if effective_draw > available_energy {
        effective_draw -= weapon_draw;
        weapons_powered = false;
    } else {
        weapons_powered = weapon_draw > Fx::from_num(0);
    }

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
    let new_stored_energy =
        (power_state.stored_energy + net_power * dt).clamp(Fx::from_num(0), power_model.battery_capacity);

    power_state.stored_energy = new_stored_energy;
    power_state.generation = power_model.reactor_output;
    power_state.draw = effective_draw;
    power_state.surplus = net_power;
    power_state.engine_power_ratio = engine_power_ratio;
    power_state.weapons_powered = weapons_powered;
    power_state.engines_powered = engine_power_ratio > Fx::from_num(0);
}

pub(super) fn count_modules(ship: &ShipDefinition, kind: ModuleKind) -> u32 {
    ship.modules
        .iter()
        .filter(|module| module.kind == kind)
        .count() as u32
}

pub(super) fn spawn_player_projectile(commands: &mut Commands, origin: FixedVec2, velocity: FixedVec2) {
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
    origin: FixedVec2,
    velocity: FixedVec2,
    faction: ProjectileFaction,
    damage: i32,
    color: Color,
) {
    let velocity_angle = angle_from_vector(velocity);

    commands.spawn((
        Sprite::from_color(color, Vec2::new(10.0, 6.0)),
        Transform {
            translation: render_translation(origin, 2.0),
            rotation: Quat::from_rotation_z(-velocity_angle.to_num::<f32>() + FRAC_PI_2),
            ..default()
        },
        SimPosition { value: origin },
        Projectile {
            velocity,
            remaining_life: Fx::from_num(PROJECTILE_LIFETIME),
            damage,
            faction,
        },
        PlayingCleanup,
    ));
}

pub(super) fn is_inside_arena(position: FixedVec2) -> bool {
    let arena_half_w = Fx::from_num(ARENA_WIDTH_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2);
    let arena_half_h = Fx::from_num(ARENA_HEIGHT_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2);

    position.x >= -arena_half_w
        && position.x <= arena_half_w
        && position.y >= -arena_half_h
        && position.y <= arena_half_h
}

pub(super) fn damp_scalar(value: Fx, damping: Fx, dt: Fx) -> Fx {
    value * (Fx::from_num(1) / (Fx::from_num(1) + damping * dt))
}

pub(super) fn damp_vec2(value: FixedVec2, damping: Fx, dt: Fx) -> FixedVec2 {
    value * (Fx::from_num(1) / (Fx::from_num(1) + damping * dt))
}

pub(super) fn clamp_position_to_arena(position: &mut FixedVec2) {
    let arena_half_w = Fx::from_num(ARENA_WIDTH_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2)
        - Fx::from_num(TILE_SIZE);
    let arena_half_h = Fx::from_num(ARENA_HEIGHT_TILES) * Fx::from_num(TILE_SIZE) * fx_ratio(1, 2)
        - Fx::from_num(TILE_SIZE);

    position.x = position.x.clamp(-arena_half_w, arena_half_w);
    position.y = position.y.clamp(-arena_half_h, arena_half_h);
}

pub(super) fn render_translation(position: FixedVec2, z: f32) -> Vec3 {
    Vec3::new(position.x.to_num::<f32>(), position.y.to_num::<f32>(), z)
}

pub(super) fn facing_vector(radians: Fx) -> FixedVec2 {
    let radians = wrap_radians(radians);
    FixedVec2::new(-sin(radians), cos(radians))
}

pub(super) fn angle_from_vector(vector: FixedVec2) -> Fx {
    if vector.is_near_zero() {
        return Fx::from_num(0);
    }

    Fx::from_num(atan2(widen(vector.y), widen(vector.x)))
}

pub(super) fn fixed_radius_sq(radius: f32) -> WideFx {
    let radius = WideFx::from_num(radius);
    radius * radius
}

pub(super) fn sprite_path_for_kind(kind: &ModuleKind) -> String {
    format!("tiles/{}.png", kind.as_str())
}

pub(super) fn interaction_for_module(
    kind: ModuleKind,
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> Option<InteractionKind> {
    if destroyed {
        return None;
    }
    if kind == ModuleKind::Cockpit {
        return Some(InteractionKind::Cockpit);
    }
    if kind == ModuleKind::Reactor {
        return Some(InteractionKind::Reactor);
    }
    if kind == ModuleKind::Turret {
        return Some(InteractionKind::Turret);
    }
    if integrity.current < integrity.max || runtime_state.needs_attention || runtime_state.is_disabled {
        return Some(InteractionKind::Repair);
    }
    None
}

pub(super) fn interaction_prompt(kind: InteractionKind) -> &'static str {
    match kind {
        InteractionKind::Cockpit => "E: return to flight controls",
        InteractionKind::Reactor => "Hold E: stabilize reactor",
        InteractionKind::Turret => "E: reset turret",
        InteractionKind::Repair => "Hold E: repair module",
    }
}

pub(super) fn is_hold_interaction(kind: InteractionKind) -> bool {
    matches!(kind, InteractionKind::Reactor | InteractionKind::Repair)
}

pub(super) fn interaction_hold_duration(kind: InteractionKind) -> Fx {
    match kind {
        InteractionKind::Cockpit | InteractionKind::Turret => Fx::from_num(0),
        InteractionKind::Reactor => Fx::from_num(1.2),
        InteractionKind::Repair => Fx::from_num(1.8),
    }
}

pub(super) fn module_effectiveness(
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> Fx {
    if destroyed || integrity.current <= 0 || runtime_state.is_disabled {
        return Fx::from_num(0);
    }

    let mut effectiveness = Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
    if runtime_state.needs_attention {
        effectiveness *= fx_ratio(3, 4);
    }
    effectiveness -= runtime_state.current_heat * fx_ratio(1, 48);
    effectiveness -= runtime_state.electrical_instability * fx_ratio(1, 40);
    effectiveness.clamp(Fx::from_num(0), Fx::from_num(1))
}

pub(super) fn module_condition(
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> ModuleCondition {
    if destroyed || integrity.current <= 0 {
        ModuleCondition::Destroyed
    } else if runtime_state.is_disabled {
        ModuleCondition::Disabled
    } else if runtime_state.needs_attention
        || runtime_state.current_heat >= Fx::from_num(8)
        || runtime_state.electrical_instability >= Fx::from_num(7)
        || integrity.current * 2 <= integrity.max
    {
        ModuleCondition::Degraded
    } else {
        ModuleCondition::Healthy
    }
}

pub(super) fn module_condition_label(condition: ModuleCondition) -> &'static str {
    match condition {
        ModuleCondition::Healthy => "healthy",
        ModuleCondition::Degraded => "degraded",
        ModuleCondition::Disabled => "disabled",
        ModuleCondition::Destroyed => "destroyed",
    }
}

pub(super) fn dynamic_field_output(
    emitter: &ModuleFieldEmitter,
    runtime_state: &ModuleRuntimeState,
    integrity: &Integrity,
    destroyed: bool,
) -> (Fx, Fx, Fx) {
    if destroyed || integrity.current <= 0 {
        return (Fx::from_num(0), Fx::from_num(0), Fx::from_num(0));
    }

    let damage_factor = Fx::from_num(1)
        - Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
    let attention_bonus = if runtime_state.needs_attention {
        Fx::from_num(1.5)
    } else {
        Fx::from_num(1)
    };
    let heat = emitter.heat_output * attention_bonus + damage_factor * Fx::from_num(3);
    let cooling = emitter.cooling_output;
    let electrical =
        emitter.electrical_output * attention_bonus + runtime_state.electrical_instability * fx_ratio(1, 6);

    (heat, cooling, electrical)
}

pub(super) fn local_field_distance(a: FixedVec2, b: FixedVec2) -> Fx {
    (a - b).length()
}

pub(super) fn field_attenuation(distance: Fx) -> Fx {
    let radius = Fx::from_num(TILE_SIZE * 3.5);
    if distance >= radius {
        Fx::from_num(0)
    } else {
        Fx::from_num(1) - distance / radius
    }
}

pub(super) fn danger_level(value: Fx, warning: Fx, critical: Fx) -> &'static str {
    if value >= critical {
        "critical"
    } else if value >= warning {
        "warning"
    } else {
        "safe"
    }
}
