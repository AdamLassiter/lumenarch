use crate::{
    client::gameplay::{
        components::ShipInteriorNode,
        helpers::{Fx, module_local_position},
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
