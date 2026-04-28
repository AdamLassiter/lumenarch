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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipControlMode {
    ShipFlight,
    Internal,
}

#[derive(Component)]
pub(crate) struct ShipboardControlState {
    pub(crate) mode: ShipControlMode,
}

#[derive(Component)]
pub(crate) struct PlayerFieldState {
    pub(crate) local_heat: Fx,
    pub(crate) local_electrical: Fx,
    pub(crate) heat_danger: bool,
    pub(crate) electrical_danger: bool,
}
