use std::collections::HashSet;

use crate::{
    {
        balance::BalanceConfig,
        gameplay::{
            components::{ShipAtmosphereTile, ShipInteriorNode},
            helpers::{Fx, module_local_position},
        },
    },
    ship::ShipDefinition,
};

pub(super) fn build_interior_nodes(
    ship: &ShipDefinition,
    center_x_fixed: Fx,
    center_y_fixed: Fx,
) -> Vec<ShipInteriorNode> {
    ship.modules
        .iter()
        .map(|module| ShipInteriorNode {
            module_id: module.id,
            kind: module.kind,
            grid_x: module.grid_x,
            grid_y: module.grid_y,
            local_position: module_local_position(module, center_x_fixed, center_y_fixed),
        })
        .collect()
}

pub(super) fn build_atmosphere_tiles(
    ship: &ShipDefinition,
    center_x_fixed: Fx,
    center_y_fixed: Fx,
    balance: &BalanceConfig,
) -> Vec<ShipAtmosphereTile> {
    let occupied: HashSet<(i32, i32)> = ship
        .modules
        .iter()
        .map(|module| (module.grid_x, module.grid_y))
        .collect();

    ship.modules
        .iter()
        .map(|module| {
            let mut exterior_edges = 0u8;
            if !occupied.contains(&(module.grid_x, module.grid_y - 1)) {
                exterior_edges |= 1;
            }
            if !occupied.contains(&(module.grid_x + 1, module.grid_y)) {
                exterior_edges |= 1 << 1;
            }
            if !occupied.contains(&(module.grid_x, module.grid_y + 1)) {
                exterior_edges |= 1 << 2;
            }
            if !occupied.contains(&(module.grid_x - 1, module.grid_y)) {
                exterior_edges |= 1 << 3;
            }

            ShipAtmosphereTile {
                module_id: module.id,
                kind: module.kind,
                grid_x: module.grid_x,
                grid_y: module.grid_y,
                local_position: module_local_position(module, center_x_fixed, center_y_fixed),
                oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
                exterior_edges,
            }
        })
        .collect()
}
