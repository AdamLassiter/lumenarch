use bevy::prelude::*;

use super::super::helpers::{FixedVec2, Fx};
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

#[derive(Component)]
pub(crate) struct ShipboardControlState {
    pub(crate) mode: ShipControlMode,
    pub(crate) focus_mode: StationFocusMode,
    pub(crate) focused_entity: Option<Entity>,
    pub(crate) focused_module_id: Option<u64>,
    pub(crate) focused_kind: Option<ModuleKind>,
    pub(crate) focused_family: Option<StationFamily>,
}

#[derive(Component)]
pub(crate) struct PlayerFieldState {
    pub(crate) local_heat: Fx,
    pub(crate) local_electrical: Fx,
    pub(crate) heat_danger: bool,
    pub(crate) electrical_danger: bool,
}
