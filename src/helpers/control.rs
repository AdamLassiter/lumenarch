use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    gameplay::components::{
        AirlockCommandState,
        DestroyedModule,
        InternalPosition,
        LinearVelocity,
        PlayerMotionState,
        PlayerReferenceFrame,
        ResourceInventory,
        ResourceKind,
        RuntimeShipModule,
        ShipAtmosphereState,
        ShipRoot,
        ShipboardControlState,
        SimPosition,
        SimRotation,
    },
    helpers::{FixedVec2, Fx, ship_tile_contains_point, ship_tile_overlaps_point},
    ship::ModuleKind,
};

pub(crate) fn nearby_logistics_target_ids(
    focused_module_id: u64,
    candidate_query: &Query<&RuntimeShipModule>,
) -> Vec<u64> {
    candidate_query
        .iter()
        .filter(|runtime_module| {
            runtime_module.kind == ModuleKind::Cargo
                || runtime_module.kind == ModuleKind::Processor
                || runtime_module.kind == ModuleKind::Airlock
        })
        .filter(|runtime_module| runtime_module.module_id != focused_module_id)
        .map(|runtime_module| runtime_module.module_id)
        .collect()
}

pub(crate) fn take_first_available(
    inventory: &mut ResourceInventory,
) -> Option<(ResourceKind, u32)> {
    if inventory.raw_salvage > 0 {
        inventory.raw_salvage -= 1;
        Some((ResourceKind::RawSalvage, 1))
    } else if inventory.repair_charge > 0 {
        inventory.repair_charge -= 1;
        Some((ResourceKind::RepairCharge, 1))
    } else if inventory.fuel > 0 {
        inventory.fuel -= 1;
        Some((ResourceKind::Fuel, 1))
    } else if inventory.ammunition > 0 {
        inventory.ammunition -= 1;
        Some((ResourceKind::Ammunition, 1))
    } else {
        None
    }
}

pub(crate) fn anchor_player_to_focused_station(
    motion: &mut PlayerMotionState,
    position: &mut InternalPosition,
    control_state: &ShipboardControlState,
    ship_query: &Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipAtmosphereState,
        ),
        With<ShipRoot>,
    >,
    module_query: &Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) {
    let Some(focused_entity) = control_state.focused_entity else {
        return;
    };
    let Ok((parent, runtime_module, _, _)) = module_query.get(focused_entity) else {
        return;
    };
    let Ok((ship_entity, ship_position, ship_rotation, ship_velocity, _)) =
        ship_query.get(parent.get())
    else {
        return;
    };

    motion.frame = PlayerReferenceFrame::Ship(ship_entity);
    motion.local_position = runtime_module.local_position;
    motion.local_velocity = FixedVec2::zero();
    motion.world_position =
        ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
    motion.world_velocity = ship_velocity.value;

    position.grid_x = runtime_module.grid_x;
    position.grid_y = runtime_module.grid_y;
    position.local_position = runtime_module.local_position;
}

#[derive(Clone, Copy)]
pub(crate) enum ShipCollisionShape {
    Open,
    FullTile,
    ExteriorWall,
}

#[derive(Clone, Copy)]
pub(crate) struct ShipCollisionTile {
    pub(crate) center: FixedVec2,
    pub(crate) exterior_edges: u8,
    pub(crate) shape: ShipCollisionShape,
    pub(crate) opening: bool,
}

pub(crate) fn ship_collision_tiles(
    ship_entity: Entity,
    atmosphere_state: &ShipAtmosphereState,
    module_query: &Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> Vec<ShipCollisionTile> {
    let module_state = module_query
        .iter()
        .filter(|(parent, _, _, _)| parent.get() == ship_entity)
        .map(|(_, runtime_module, airlock_state, destroyed)| {
            (
                runtime_module.module_id,
                (
                    destroyed.is_some(),
                    airlock_state.is_some_and(|state| state.open),
                ),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    atmosphere_state
        .tiles
        .iter()
        .map(|tile| {
            let (destroyed, airlock_open) = module_state
                .get(&tile.module_id)
                .copied()
                .unwrap_or((false, false));
            let shape = shipboard_collision_shape(tile.kind, destroyed, airlock_open);
            let opening = tile.exterior_edges != 0 && (destroyed || airlock_open);
            ShipCollisionTile {
                center: tile.local_position,
                exterior_edges: tile.exterior_edges,
                shape,
                opening,
            }
        })
        .collect()
}

pub(crate) fn resolve_ship_local_motion(
    start: FixedVec2,
    desired: FixedVec2,
    collision_tiles: &[ShipCollisionTile],
    collision_radius: Fx,
) -> FixedVec2 {
    let mut resolved = start;
    let try_x = FixedVec2::new(desired.x, resolved.y);
    if !movement_blocked(resolved, try_x, collision_tiles, collision_radius) {
        resolved.x = desired.x;
    }
    let try_y = FixedVec2::new(resolved.x, desired.y);
    if !movement_blocked(resolved, try_y, collision_tiles, collision_radius) {
        resolved.y = desired.y;
    }
    resolved
}

fn movement_blocked(
    from: FixedVec2,
    to: FixedVec2,
    collision_tiles: &[ShipCollisionTile],
    collision_radius: Fx,
) -> bool {
    if collision_tiles
        .iter()
        .any(|tile| tile.blocks_position(to, collision_radius))
    {
        return true;
    }

    let from_tile = collision_tiles.iter().find(|tile| {
        !matches!(tile.shape, ShipCollisionShape::FullTile)
            && ship_tile_contains_point(from, tile.center)
    });
    let to_tile = collision_tiles.iter().find(|tile| {
        !matches!(tile.shape, ShipCollisionShape::FullTile)
            && ship_tile_contains_point(to, tile.center)
    });

    match (from_tile, to_tile) {
        (None, Some(tile)) => !allows_exterior_crossing(from, tile),
        (Some(tile), None) => !allows_exterior_crossing(to, tile),
        _ => false,
    }
}

impl ShipCollisionTile {
    fn blocks_position(self, point: FixedVec2, radius: Fx) -> bool {
        match self.shape {
            ShipCollisionShape::Open => false,
            ShipCollisionShape::FullTile => ship_tile_overlaps_point(point, self.center, radius),
            ShipCollisionShape::ExteriorWall => {
                if self.exterior_edges == 0 || !ship_tile_contains_point(point, self.center) {
                    return false;
                }
                let horizontal_blocked = (self.exterior_edges & (1 << 1) != 0
                    && point.x > self.center.x)
                    || (self.exterior_edges & (1 << 3) != 0 && point.x < self.center.x);
                let vertical_blocked = (self.exterior_edges & 1 != 0 && point.y > self.center.y)
                    || (self.exterior_edges & (1 << 2) != 0 && point.y < self.center.y);
                horizontal_blocked || vertical_blocked
            }
        }
    }
}

fn allows_exterior_crossing(point: FixedVec2, tile: &ShipCollisionTile) -> bool {
    if !tile.opening || tile.exterior_edges == 0 {
        return false;
    }

    let tile_half = Fx::from_num(16);
    let left = tile.center.x - tile_half;
    let right = tile.center.x + tile_half;
    let top = tile.center.y + tile_half;
    let bottom = tile.center.y - tile_half;

    let mut exterior_mask = 0u8;
    if point.y > top {
        exterior_mask |= 1;
    }
    if point.x > right {
        exterior_mask |= 1 << 1;
    }
    if point.y < bottom {
        exterior_mask |= 1 << 2;
    }
    if point.x < left {
        exterior_mask |= 1 << 3;
    }

    exterior_mask != 0 && (tile.exterior_edges & exterior_mask) != 0
}

fn is_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )
}

fn shipboard_collision_shape(
    kind: ModuleKind,
    destroyed: bool,
    airlock_open: bool,
) -> ShipCollisionShape {
    if destroyed {
        return ShipCollisionShape::Open;
    }
    if is_hull_kind(kind) {
        return ShipCollisionShape::FullTile;
    }
    match kind {
        ModuleKind::Airlock if !airlock_open => ShipCollisionShape::ExteriorWall,
        ModuleKind::Engine | ModuleKind::Turret => ShipCollisionShape::ExteriorWall,
        _ => ShipCollisionShape::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FixedVec2,
        Fx,
        ShipCollisionShape,
        ShipCollisionTile,
        movement_blocked,
        shipboard_collision_shape,
    };
    use crate::ship::ModuleKind;

    #[test]
    fn hull_fixtures_block_shipboard_motion_like_hull() {
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Engine, false, false),
            ShipCollisionShape::ExteriorWall
        ));
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Turret, false, false),
            ShipCollisionShape::ExteriorWall
        ));
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Airlock, false, false),
            ShipCollisionShape::ExteriorWall
        ));
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Airlock, false, true),
            ShipCollisionShape::Open
        ));
    }

    #[test]
    fn destroyed_hull_fixtures_do_not_remain_solid() {
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Engine, true, false),
            ShipCollisionShape::Open
        ));
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Turret, true, false),
            ShipCollisionShape::Open
        ));
        assert!(matches!(
            shipboard_collision_shape(ModuleKind::Airlock, true, false),
            ShipCollisionShape::Open
        ));
    }

    #[test]
    fn hull_fixtures_allow_interior_half_tile_approach() {
        let tile = ShipCollisionTile {
            center: FixedVec2::new(Fx::from_num(0), Fx::from_num(0)),
            exterior_edges: 1,
            shape: ShipCollisionShape::ExteriorWall,
            opening: false,
        };
        let interior_tile = ShipCollisionTile {
            center: FixedVec2::new(Fx::from_num(0), Fx::from_num(-32)),
            exterior_edges: 0,
            shape: ShipCollisionShape::Open,
            opening: false,
        };
        let radius = Fx::from_num(7);
        assert!(!movement_blocked(
            FixedVec2::new(Fx::from_num(0), Fx::from_num(-20)),
            FixedVec2::new(Fx::from_num(0), Fx::from_num(-1)),
            &[tile, interior_tile],
            radius,
        ));
        assert!(movement_blocked(
            FixedVec2::new(Fx::from_num(0), Fx::from_num(-1)),
            FixedVec2::new(Fx::from_num(0), Fx::from_num(1)),
            &[tile, interior_tile],
            radius,
        ));
    }
}
