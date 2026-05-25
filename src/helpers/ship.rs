use cordic::{cos, sin};

use super::{FixedVec2, Fx, fx_ratio};
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::components::{ShipMovementModel, ShipPowerModel},
    ship::{ModuleKind, ModuleSpec, ModuleVariant, ShipFoundationKind, ShipModule},
};

pub(crate) fn module_local_position(module: &ShipModule, center_x: Fx, center_y: Fx) -> FixedVec2 {
    FixedVec2::new(
        (Fx::from_num(module.grid_x) - center_x) * Fx::from_num(TILE_SIZE),
        (center_y - Fx::from_num(module.grid_y)) * Fx::from_num(TILE_SIZE),
    )
}

pub(crate) fn ship_grid_from_local_position(local_position: FixedVec2) -> (i32, i32) {
    ship_grid_from_local_position_with_origin(local_position, FixedVec2::zero())
}

pub(crate) fn ship_grid_from_local_position_with_origin(
    local_position: FixedVec2,
    grid_origin: FixedVec2,
) -> (i32, i32) {
    let local_position = local_position - grid_origin;
    let tile_size = Fx::from_num(TILE_SIZE);
    let tile_half = tile_size / Fx::from_num(2);
    (
        ((local_position.x + tile_half) / tile_size)
            .floor()
            .to_num::<i32>(),
        ((-local_position.y + tile_half) / tile_size)
            .floor()
            .to_num::<i32>(),
    )
}

pub(crate) fn ship_grid_facing_offset(facing_radians: Fx) -> (i32, i32) {
    let radians = super::wrap_radians(facing_radians);
    let facing_x = cos(radians);
    let facing_y = sin(radians);
    let threshold = Fx::from_num(0.382_683_43);
    let x = if facing_x > threshold {
        1
    } else if facing_x < -threshold {
        -1
    } else {
        0
    };
    let y = if facing_y > threshold {
        -1
    } else if facing_y < -threshold {
        1
    } else {
        0
    };
    (x, y)
}

pub(crate) fn focused_ship_grid_tile(local_position: FixedVec2, facing_radians: Fx) -> (i32, i32) {
    focused_ship_grid_tile_with_origin(local_position, facing_radians, FixedVec2::zero())
}

pub(crate) fn focused_ship_grid_tile_with_origin(
    local_position: FixedVec2,
    facing_radians: Fx,
    grid_origin: FixedVec2,
) -> (i32, i32) {
    let (grid_x, grid_y) = ship_grid_from_local_position_with_origin(local_position, grid_origin);
    let (offset_x, offset_y) = ship_grid_facing_offset(facing_radians);
    (grid_x + offset_x, grid_y + offset_y)
}

pub(crate) fn cardinal_neighbors((x, y): (i32, i32)) -> [(i32, i32); 4] {
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

pub(crate) fn component_service_coords((x, y): (i32, i32)) -> [(i32, i32); 5] {
    [(x, y), (x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

pub(crate) fn sprite_path_for_foundation(kind: ShipFoundationKind) -> String {
    format!(
        "tiles/{}/{}.png",
        foundation_asset_category(kind),
        kind.as_str()
    )
}

fn foundation_asset_category(kind: ShipFoundationKind) -> &'static str {
    match kind {
        ShipFoundationKind::Hull
        | ShipFoundationKind::HullInnerCorner
        | ShipFoundationKind::HullOuterCorner => "hull",
        ShipFoundationKind::Floor
        | ShipFoundationKind::Wire
        | ShipFoundationKind::OxygenDuct
        | ShipFoundationKind::PipeRawSalvage
        | ShipFoundationKind::PipeRepairCharge
        | ShipFoundationKind::PipeFuel
        | ShipFoundationKind::PipeAmmunition
        | ShipFoundationKind::PipeOxygen => "logistics",
    }
}

pub(crate) fn sprite_path_for_foundation_connections(
    kind: ShipFoundationKind,
    north: bool,
    east: bool,
    south: bool,
    west: bool,
) -> (String, u8) {
    if !kind.is_route() {
        return (sprite_path_for_foundation(kind), 0);
    }
    let count = [north, east, south, west]
        .into_iter()
        .filter(|connected| *connected)
        .count();
    let base = kind.as_str();
    let category = foundation_asset_category(kind);
    match count {
        4 => (format!("tiles/{category}/{base}_cross.png"), 0),
        3 => {
            let missing = if !north {
                2
            } else if !east {
                3
            } else if !south {
                0
            } else {
                1
            };
            (format!("tiles/{category}/{base}_tee.png"), missing)
        }
        2 if (north && south) || (east && west) => {
            let rotation = if east && west { 1 } else { 0 };
            (format!("tiles/{category}/{base}_straight.png"), rotation)
        }
        2 => {
            let rotation = match (north, east, south, west) {
                (true, true, false, false) => 0,
                (false, true, true, false) => 1,
                (false, false, true, true) => 2,
                (true, false, false, true) => 3,
                _ => 0,
            };
            (format!("tiles/{category}/{base}_corner.png"), rotation)
        }
        1 => {
            let rotation = if east {
                1
            } else if south {
                2
            } else if west {
                3
            } else {
                0
            };
            (format!("tiles/{category}/{base}_end.png"), rotation)
        }
        _ => (sprite_path_for_foundation(kind), 0),
    }
}

pub(crate) fn ship_tile_contains_point(point: FixedVec2, tile_center: FixedVec2) -> bool {
    let tile_half = Fx::from_num(TILE_SIZE) / Fx::from_num(2);
    (point.x - tile_center.x).abs() <= tile_half && (point.y - tile_center.y).abs() <= tile_half
}

pub(crate) fn ship_tile_overlaps_point(
    point: FixedVec2,
    tile_center: FixedVec2,
    radius: Fx,
) -> bool {
    let tile_half = Fx::from_num(TILE_SIZE) / Fx::from_num(2);
    (point.x - tile_center.x).abs() <= tile_half + radius
        && (point.y - tile_center.y).abs() <= tile_half + radius
}

pub(crate) fn module_integrity(kind: ModuleKind, variant: ModuleVariant) -> i32 {
    ModuleSpec::for_module(kind, variant).integrity
}

pub(crate) fn ship_movement_model_with_effective(
    module_count: usize,
    engine_count: u32,
    effective_engines: Fx,
    helm_multiplier: Fx,
    balance: &BalanceConfig,
) -> ShipMovementModel {
    let engine_scalar = effective_engines.max(fx_ratio(1, 4)).to_num::<f32>();
    let mass_factor = (module_count.max(1) as f32).sqrt();
    let helm_scalar = helm_multiplier.max(Fx::from_num(1)).to_num::<f32>();

    ShipMovementModel {
        engine_count,
        helm_multiplier,
        thrust_acceleration: Fx::from_num(
            balance.ship.thrust_base_acceleration * engine_scalar * helm_scalar / mass_factor,
        ),
        turn_speed: Fx::from_num(
            (balance.ship.turn_speed_base + balance.ship.turn_speed_per_engine * engine_scalar)
                * helm_scalar,
        ),
        max_speed: Fx::from_num(
            (balance.ship.max_speed_base + balance.ship.max_speed_per_engine * engine_scalar)
                * (0.9 + helm_scalar * 0.1),
        ),
        linear_damping: Fx::from_num(balance.ship.linear_damping),
        angular_damping: Fx::from_num(balance.ship.angular_damping),
    }
}

pub(crate) fn ship_power_model_with_effective(
    module_count: usize,
    _reactor_count: u32,
    battery_count: u32,
    engine_count: u32,
    turret_count: u32,
    effective_reactors: Fx,
    effective_batteries: Fx,
    effective_charge_rate: Fx,
    effective_discharge_rate: Fx,
    effective_engines: Fx,
    effective_turrets: Fx,
    balance: &BalanceConfig,
) -> ShipPowerModel {
    ShipPowerModel {
        reactor_output: effective_reactors * Fx::from_num(balance.ship.reactor_output_per_reactor),
        battery_capacity: effective_batteries.max(Fx::from_num(battery_count))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        battery_charge_rate: effective_charge_rate.max(Fx::from_num(1))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        battery_discharge_rate: effective_discharge_rate.max(Fx::from_num(1))
            * Fx::from_num(balance.ship.battery_capacity_per_battery),
        passive_draw: Fx::from_num(
            balance.ship.passive_draw_base
                + module_count as f32 * balance.ship.passive_draw_per_module,
        ),
        engine_draw: effective_engines.max(Fx::from_num(engine_count))
            * Fx::from_num(balance.ship.engine_draw_per_engine),
        weapon_draw: effective_turrets.max(Fx::from_num(turret_count))
            * Fx::from_num(balance.ship.weapon_draw_per_turret),
        shield_draw: Fx::from_num(0),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FixedVec2,
        Fx,
        cardinal_neighbors,
        component_service_coords,
        focused_ship_grid_tile,
        focused_ship_grid_tile_with_origin,
        module_local_position,
        ship_grid_facing_offset,
        ship_grid_from_local_position,
        ship_grid_from_local_position_with_origin,
        ship_tile_contains_point,
        ship_tile_overlaps_point,
        sprite_path_for_foundation_connections,
    };
    use crate::{
        TILE_SIZE,
        ship::{ModuleKind, ShipFoundationKind, ShipModule},
    };

    #[test]
    fn module_centers_round_trip_to_their_ship_grid_tile() {
        for grid_x in -4..=4 {
            for grid_y in -4..=4 {
                let module = ShipModule::new(1, ModuleKind::Core, grid_x, grid_y, 0);
                let local_position =
                    module_local_position(&module, Fx::from_num(0), Fx::from_num(0));

                assert_eq!(
                    ship_grid_from_local_position(local_position),
                    (grid_x, grid_y),
                    "module center should map back to its grid coordinate"
                );
            }
        }
    }

    #[test]
    fn points_inside_a_tile_map_to_that_same_tile_underfoot() {
        let tile_size = Fx::from_num(TILE_SIZE);
        let inside_edge = tile_size / Fx::from_num(2) - Fx::from_num(1) / Fx::from_num(16);

        for grid_x in -3..=3 {
            for grid_y in -3..=3 {
                let center = FixedVec2::new(
                    Fx::from_num(grid_x) * tile_size,
                    Fx::from_num(-grid_y) * tile_size,
                );

                for offset in [
                    FixedVec2::zero(),
                    FixedVec2::new(inside_edge, Fx::from_num(0)),
                    FixedVec2::new(-inside_edge, Fx::from_num(0)),
                    FixedVec2::new(Fx::from_num(0), inside_edge),
                    FixedVec2::new(Fx::from_num(0), -inside_edge),
                    FixedVec2::new(inside_edge, -inside_edge),
                    FixedVec2::new(-inside_edge, inside_edge),
                ] {
                    let point = center + offset;
                    assert!(
                        ship_tile_contains_point(point, center),
                        "test point should be inside the source tile"
                    );
                    assert_eq!(
                        ship_grid_from_local_position(point),
                        (grid_x, grid_y),
                        "underfoot grid should stay on the tile containing the point"
                    );
                }
            }
        }
    }

    #[test]
    fn just_past_a_tile_midline_maps_to_the_adjacent_tile() {
        let tile_size = Fx::from_num(TILE_SIZE);
        let outside_edge = tile_size / Fx::from_num(2) + Fx::from_num(1) / Fx::from_num(16);
        let center = FixedVec2::zero();

        assert_eq!(
            ship_grid_from_local_position(center + FixedVec2::new(outside_edge, Fx::from_num(0))),
            (1, 0)
        );
        assert_eq!(
            ship_grid_from_local_position(center + FixedVec2::new(-outside_edge, Fx::from_num(0))),
            (-1, 0)
        );
        assert_eq!(
            ship_grid_from_local_position(center + FixedVec2::new(Fx::from_num(0), outside_edge)),
            (0, -1)
        );
        assert_eq!(
            ship_grid_from_local_position(center + FixedVec2::new(Fx::from_num(0), -outside_edge)),
            (0, 1)
        );
    }

    #[test]
    fn facing_offsets_follow_ship_grid_axes() {
        assert_eq!(ship_grid_facing_offset(Fx::FRAC_PI_2), (0, -1));
        assert_eq!(ship_grid_facing_offset(-Fx::FRAC_PI_2), (0, 1));
        assert_eq!(ship_grid_facing_offset(Fx::from_num(0)), (1, 0));
        assert_eq!(ship_grid_facing_offset(Fx::PI), (-1, 0));
    }

    #[test]
    fn diagonal_facing_offsets_select_diagonal_tiles() {
        assert_eq!(ship_grid_facing_offset(Fx::FRAC_PI_4), (1, -1));
        assert_eq!(ship_grid_facing_offset(Fx::PI - Fx::FRAC_PI_4), (-1, -1));
        assert_eq!(ship_grid_facing_offset(-Fx::PI + Fx::FRAC_PI_4), (-1, 1));
        assert_eq!(ship_grid_facing_offset(-Fx::FRAC_PI_4), (1, 1));
    }

    #[test]
    fn focused_tile_is_adjacent_to_containing_tile() {
        let tile_size = Fx::from_num(TILE_SIZE);
        let inside_edge = tile_size / Fx::from_num(2) - Fx::from_num(1) / Fx::from_num(16);
        let base = FixedVec2::new(Fx::from_num(2) * tile_size, Fx::from_num(3) * -tile_size);

        for offset in [
            FixedVec2::zero(),
            FixedVec2::new(inside_edge, Fx::from_num(0)),
            FixedVec2::new(-inside_edge, Fx::from_num(0)),
            FixedVec2::new(Fx::from_num(0), inside_edge),
            FixedVec2::new(Fx::from_num(0), -inside_edge),
        ] {
            assert_eq!(
                focused_ship_grid_tile(base + offset, Fx::from_num(0)),
                (3, 3),
                "focused tile should use the containing tile before applying facing"
            );
        }
    }

    #[test]
    fn focused_tile_examples_match_ship_grid_coordinates() {
        let origin = FixedVec2::zero();

        assert_eq!(focused_ship_grid_tile(origin, Fx::FRAC_PI_2), (0, -1));
        assert_eq!(focused_ship_grid_tile(origin, -Fx::FRAC_PI_2), (0, 1));
        assert_eq!(focused_ship_grid_tile(origin, Fx::from_num(0)), (1, 0));
        assert_eq!(focused_ship_grid_tile(origin, Fx::PI), (-1, 0));
        assert_eq!(focused_ship_grid_tile(origin, Fx::FRAC_PI_4), (1, -1));
    }

    #[test]
    fn focused_tile_accounts_for_shifted_runtime_grid_origin() {
        let tile_size = Fx::from_num(TILE_SIZE);
        let grid_origin = FixedVec2::new(Fx::from_num(0), tile_size);
        let origin_local_position = grid_origin;

        assert_eq!(
            ship_grid_from_local_position_with_origin(origin_local_position, grid_origin),
            (0, 0)
        );
        assert_eq!(
            focused_ship_grid_tile_with_origin(origin_local_position, Fx::FRAC_PI_2, grid_origin),
            (0, -1)
        );
        assert_eq!(
            focused_ship_grid_tile_with_origin(origin_local_position, -Fx::FRAC_PI_2, grid_origin),
            (0, 1)
        );
    }

    #[test]
    fn tile_overlap_expands_contains_by_radius() {
        let center = FixedVec2::zero();
        let tile_half = Fx::from_num(TILE_SIZE) / Fx::from_num(2);
        let radius = Fx::from_num(3);
        let point_inside_radius = FixedVec2::new(tile_half + radius, Fx::from_num(0));
        let point_outside_radius =
            FixedVec2::new(tile_half + radius + Fx::from_num(1), Fx::from_num(0));

        assert!(!ship_tile_contains_point(point_inside_radius, center));
        assert!(ship_tile_overlaps_point(
            point_inside_radius,
            center,
            radius
        ));
        assert!(!ship_tile_overlaps_point(
            point_outside_radius,
            center,
            radius
        ));
    }

    #[test]
    fn cardinal_neighbors_are_stable_and_cardinal_only() {
        assert_eq!(
            cardinal_neighbors((7, -2)),
            [(8, -2), (6, -2), (7, -1), (7, -3)]
        );
    }

    #[test]
    fn component_service_coords_cover_self_and_cardinal_neighbors_only() {
        assert_eq!(
            component_service_coords((7, -2)),
            [(7, -2), (8, -2), (6, -2), (7, -1), (7, -3)]
        );
    }

    #[test]
    fn route_sprite_selection_uses_end_variant_for_one_connection() {
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::Wire,
                true,
                false,
                false,
                false
            ),
            ("tiles/logistics/wire_end.png".to_string(), 0)
        );
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::Wire,
                false,
                true,
                false,
                false
            ),
            ("tiles/logistics/wire_end.png".to_string(), 1)
        );
    }

    #[test]
    fn route_sprite_selection_covers_all_connection_shapes() {
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::PipeFuel,
                false,
                false,
                false,
                false,
            ),
            ("tiles/logistics/pipe_fuel.png".to_string(), 0)
        );
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::PipeFuel,
                true,
                false,
                true,
                false,
            ),
            ("tiles/logistics/pipe_fuel_straight.png".to_string(), 0)
        );
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::PipeFuel,
                true,
                true,
                false,
                false,
            ),
            ("tiles/logistics/pipe_fuel_corner.png".to_string(), 0)
        );
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::PipeFuel,
                true,
                true,
                true,
                false,
            ),
            ("tiles/logistics/pipe_fuel_tee.png".to_string(), 1)
        );
        assert_eq!(
            sprite_path_for_foundation_connections(
                ShipFoundationKind::PipeFuel,
                true,
                true,
                true,
                true,
            ),
            ("tiles/logistics/pipe_fuel_cross.png".to_string(), 0)
        );
    }
}
