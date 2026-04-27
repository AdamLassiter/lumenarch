use bevy::prelude::*;

use super::helpers::{FixedVec2, Fx};
use crate::ship::ModuleKind;

#[derive(Component)]
pub(crate) struct PlayerShip;

#[derive(Component)]
pub(crate) struct ShipRoot;

#[derive(Component)]
pub(crate) struct ShipboardPlayer;

#[derive(Component)]
pub(crate) struct ShipboardMarker;

#[derive(Component)]
pub(crate) struct PlayerShipAssignment {
    pub(crate) _ship_entity: Entity,
}

#[derive(Clone)]
pub(crate) struct ShipInteriorNode {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component, Default)]
pub(crate) struct ShipInteriorMap {
    pub(crate) walkable_nodes: Vec<ShipInteriorNode>,
}

#[derive(Component)]
pub(crate) struct InternalPosition {
    pub(crate) node_index: usize,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component)]
pub(crate) struct CurrentStation {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipControlMode {
    ShipFlight,
    Internal,
}

#[derive(Component)]
pub(crate) struct ShipboardControlState {
    pub(crate) mode: ShipControlMode,
}

#[derive(Component, Default)]
pub(crate) struct NearbyInteraction {
    pub(crate) target: Option<Entity>,
    pub(crate) kind: Option<InteractionKind>,
    pub(crate) prompt: Option<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Component, Default)]
pub(crate) struct HeldInteraction {
    pub(crate) target: Option<Entity>,
    pub(crate) kind: Option<InteractionKind>,
    pub(crate) progress: Fx,
    pub(crate) required: Fx,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InteractionKind {
    Cockpit,
    Computer,
    Reactor,
    Turret,
    Engine,
    Repair,
}

#[derive(Component)]
pub(crate) struct Interactable;

#[derive(Event)]
pub(crate) struct InteractWithModule {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}

#[derive(Event)]
pub(crate) struct BeginHeldInteraction {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
    pub(crate) required: Fx,
}

#[derive(Event)]
pub(crate) struct CompleteHeldInteraction {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}

#[derive(Component)]
pub(crate) struct RuntimeShipModule {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component)]
pub(crate) struct Integrity {
    pub(crate) current: i32,
    pub(crate) max: i32,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct PowerProducer {
    pub(crate) output: i32,
}

#[allow(dead_code)]
#[derive(Component)]
pub(crate) struct PowerConsumer {
    pub(crate) draw: i32,
}

#[derive(Component)]
pub(crate) struct EngineModule;

#[derive(Component)]
pub(crate) struct WeaponModule;

#[derive(Component)]
pub(crate) struct ArchComputerModule;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleCondition {
    Healthy,
    Degraded,
    Disabled,
    Destroyed,
}

#[derive(Component)]
pub(crate) struct ModuleRuntimeState {
    pub(crate) current_heat: Fx,
    pub(crate) electrical_instability: Fx,
    pub(crate) sampled_heat: Fx,
    pub(crate) sampled_electrical: Fx,
    pub(crate) is_disabled: bool,
    pub(crate) was_disabled_last_frame: bool,
    pub(crate) needs_attention: bool,
    pub(crate) last_interaction_age: Fx,
}

#[derive(Component)]
pub(crate) struct ModuleFieldEmitter {
    pub(crate) heat_output: Fx,
    pub(crate) cooling_output: Fx,
    pub(crate) electrical_output: Fx,
    pub(crate) grounding_output: Fx,
}

#[derive(Component)]
pub(crate) struct PlayerFieldState {
    pub(crate) local_heat: Fx,
    pub(crate) local_electrical: Fx,
    pub(crate) heat_danger: bool,
    pub(crate) electrical_danger: bool,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipAutomationMode {
    Off,
    ReactorGuard,
}

#[derive(Component)]
pub(crate) struct ShipAutomationState {
    pub(crate) mode: ShipAutomationMode,
    pub(crate) active: bool,
    pub(crate) output_scale: Fx,
    pub(crate) trigger_count: u32,
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
    pub(crate) repairs_performed: u32,
    pub(crate) stabilizations_performed: u32,
    pub(crate) automation_used: bool,
    pub(crate) automation_trigger_count: u32,
    pub(crate) highest_heat: Fx,
    pub(crate) hottest_module_kind: Option<ModuleKind>,
    pub(crate) first_disabled_module_kind: Option<ModuleKind>,
    pub(crate) recent_action: Option<String>,
    pub(crate) recent_action_timer: Fx,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectileFaction {
    Player,
    Hostile,
}
