use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use ggrs::PlayerHandle;

use super::{
    super::helpers::{FixedVec2, Fx},
    logistics::ResourceKind,
};
use crate::ship::ModuleKind;

#[derive(Component)]
pub(crate) struct PlayerShip;

#[derive(Component)]
pub(crate) struct ShipRoot;

#[derive(Component)]
pub(crate) struct HostileShip;

#[derive(Component, Clone)]
pub(crate) struct HostileShipAi {
    pub(crate) preferred_range: Fx,
    pub(crate) aggression: Fx,
    pub(crate) salvage_reward: u32,
}

#[derive(Component)]
pub(crate) struct ShipboardPlayer;

#[derive(Component, Clone, Copy)]
pub(crate) struct PlayerHandleComponent {
    pub(crate) handle: PlayerHandle,
}

#[derive(Component)]
pub(crate) struct ObservedLocalPlayerMarker;

#[derive(Component)]
pub(crate) struct ShipboardMarker;

#[derive(Component, Clone)]
pub(crate) struct ShipInertiaField {
    pub(crate) radius: Fx,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerShipAssignment {
    pub(crate) _ship_entity: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerReferenceFrame {
    World,
    Ship(Entity),
}

#[derive(Component, Clone)]
pub(crate) struct PlayerMotionState {
    pub(crate) frame: PlayerReferenceFrame,
    pub(crate) world_position: FixedVec2,
    pub(crate) world_velocity: FixedVec2,
    pub(crate) local_position: FixedVec2,
    pub(crate) local_velocity: FixedVec2,
}

#[derive(Component, Default, Clone)]
pub(crate) struct CarriedResource {
    pub(crate) kind: Option<ResourceKind>,
    pub(crate) amount: u32,
}

#[derive(Clone)]
pub(crate) struct ShipInteriorNode {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component, Default, Clone)]
pub(crate) struct ShipInteriorMap {
    pub(crate) walkable_nodes: Vec<ShipInteriorNode>,
}

#[derive(Clone)]
pub(crate) struct ShipAtmosphereTile {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
    pub(crate) oxygen: Fx,
    pub(crate) exterior_edges: u8,
}

#[derive(Component, Default, Clone)]
pub(crate) struct ShipAtmosphereState {
    pub(crate) tiles: Vec<ShipAtmosphereTile>,
    pub(crate) average_oxygen: Fx,
    pub(crate) minimum_oxygen: Fx,
    pub(crate) venting_tiles: u32,
    pub(crate) decompression_reported: bool,
}

#[derive(Component, Clone)]
pub(crate) struct InternalPosition {
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component, Clone)]
pub(crate) struct CurrentStation {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StationFamily {
    Cockpit,
    Turret,
    Reactor,
    Storage,
    Manipulator,
    Processor,
    Computer,
    Shield,
    Detector,
    Drone,
    Memory,
}

impl StationFamily {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Cockpit => "Cockpit",
            Self::Turret => "Turret",
            Self::Reactor => "Reactor",
            Self::Storage => "Storage",
            Self::Manipulator => "Manipulator",
            Self::Processor => "Processor",
            Self::Computer => "Computer",
            Self::Shield => "Shield",
            Self::Detector => "Detector",
            Self::Drone => "Drone Station",
            Self::Memory => "Memory Bank",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipControlMode {
    Interior,
    Cockpit,
    Turret,
    Reactor,
    Logistics,
    Computer,
}

impl ShipControlMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Interior => "Interior",
            Self::Cockpit => "Cockpit",
            Self::Turret => "Turret",
            Self::Reactor => "Reactor",
            Self::Logistics => "Logistics",
            Self::Computer => "Computer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StationFocusMode {
    Internal,
    Focused,
}

#[derive(Component, Clone)]
pub(crate) struct ShipboardControlState {
    pub(crate) mode: ShipControlMode,
    pub(crate) focus_mode: StationFocusMode,
    pub(crate) focused_entity: Option<Entity>,
    pub(crate) focused_module_id: Option<u64>,
    pub(crate) focused_kind: Option<ModuleKind>,
    pub(crate) focused_family: Option<StationFamily>,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerFieldState {
    pub(crate) local_heat: Fx,
    pub(crate) local_electrical: Fx,
    pub(crate) local_oxygen: Fx,
    pub(crate) heat_danger: bool,
    pub(crate) electrical_danger: bool,
    pub(crate) oxygen_warning: bool,
    pub(crate) oxygen_critical: bool,
}

impl MapEntities for PlayerShipAssignment {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self._ship_entity = entity_mapper.map_entity(self._ship_entity);
    }
}

impl MapEntities for PlayerReferenceFrame {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Self::Ship(entity) = self {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

impl MapEntities for PlayerMotionState {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.frame.map_entities(entity_mapper);
    }
}

impl MapEntities for ShipboardControlState {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.focused_entity {
            self.focused_entity = Some(entity_mapper.map_entity(entity));
        }
    }
}
