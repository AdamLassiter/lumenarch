use super::*;

pub(crate) fn wrap_angle_f32(angle: f32) -> f32 {
    let mut angle = angle;
    while angle <= -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }
    while angle > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    angle
}

pub(crate) fn fixed_square(value: Fx) -> crate::gameplay::helpers::WideFx {
    crate::gameplay::helpers::widen(value) * crate::gameplay::helpers::widen(value)
}

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
    inventory: &mut crate::gameplay::components::ResourceInventory,
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
pub(crate) struct ShipCollisionTile {
    pub(crate) center: FixedVec2,
    pub(crate) exterior_edges: u8,
    pub(crate) solid: bool,
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
            let solid = is_hull_kind(tile.kind) && !destroyed;
            let opening = tile.exterior_edges != 0 && (destroyed || airlock_open);
            ShipCollisionTile {
                center: tile.local_position,
                exterior_edges: tile.exterior_edges,
                solid,
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
        .any(|tile| tile.solid && point_overlaps_tile(to, tile.center, collision_radius))
    {
        return true;
    }

    let from_tile = collision_tiles
        .iter()
        .find(|tile| !tile.solid && point_inside_tile(from, tile.center));
    let to_tile = collision_tiles
        .iter()
        .find(|tile| !tile.solid && point_inside_tile(to, tile.center));

    match (from_tile, to_tile) {
        (None, Some(tile)) => !allows_exterior_crossing(from, tile),
        (Some(tile), None) => !allows_exterior_crossing(to, tile),
        _ => false,
    }
}

fn point_overlaps_tile(point: FixedVec2, tile_center: FixedVec2, radius: Fx) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half + radius
        && (point.y - tile_center.y).abs() <= tile_half + radius
}

fn point_inside_tile(point: FixedVec2, tile_center: FixedVec2) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half && (point.y - tile_center.y).abs() <= tile_half
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
