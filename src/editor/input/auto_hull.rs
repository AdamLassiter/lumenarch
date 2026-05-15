use std::collections::{HashMap, HashSet};

use crate::{
    helpers::editor::{
        foundation_supports_module,
        is_hull_foundation_kind,
        is_hull_module_kind,
        logistics_tile_must_be_enclosed,
        module_requires_hull_foundation,
    },
    ship::{ShipDefinition, ShipFoundationKind, ShipFoundationTile},
};

/// Rebuilds the derived hull shell around all placed structure while leaving manual exterior
/// fixtures like turrets, airlocks, and engines sitting on their own hull tiles.
pub(super) fn apply_auto_hull_to_ship(ship: &mut ShipDefinition) -> bool {
    let component_occupied = ship
        .modules
        .iter()
        .filter(|module| {
            !module_requires_hull_foundation(module.kind) && !is_hull_module_kind(module.kind)
        })
        .map(|module| (module.grid_x, module.grid_y))
        .collect::<HashSet<_>>();
    let enclosed_logistics = ship
        .foundation_tiles
        .iter()
        .filter(|tile| logistics_tile_must_be_enclosed(tile.kind))
        .map(|tile| (tile.grid_x, tile.grid_y))
        .collect::<HashSet<_>>();
    let occupied = component_occupied
        .iter()
        .copied()
        .chain(enclosed_logistics.iter().copied())
        .collect::<HashSet<_>>();
    if occupied.is_empty() {
        return false;
    }

    let preserved_hull = ship
        .hull_tiles
        .iter()
        .filter(|tile| is_hull_foundation_kind(tile.kind))
        .filter(|tile| {
            ship.module_at(tile.grid_x, tile.grid_y)
                .is_some_and(|module| {
                    foundation_supports_module(
                        ship.logistics_at(tile.grid_x, tile.grid_y)
                            .map(|entry| entry.kind),
                        Some(tile.kind),
                        module.kind,
                    )
                })
        })
        .map(|tile| {
            let rotation = ship
                .module_at(tile.grid_x, tile.grid_y)
                .map(|module| module.rotation_quadrants)
                .unwrap_or(tile.rotation_quadrants);
            ((tile.grid_x, tile.grid_y), (tile.kind, rotation))
        })
        .collect::<HashMap<_, _>>();

    let mut candidates = HashSet::new();
    for &(grid_x, grid_y) in &occupied {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let target = (grid_x + dx, grid_y + dy);
                if !occupied.contains(&target) {
                    candidates.insert(target);
                }
            }
        }
    }

    let mut desired_hull = candidates
        .into_iter()
        .filter_map(|target| {
            auto_hull_kind_for_cell(&occupied, target)
                .map(|(kind, rotation)| (target, (kind, rotation)))
        })
        .collect::<Vec<_>>();
    desired_hull.sort_by_key(|((x, y), _)| (*y, *x));

    ship.hull_tiles.retain(|tile| {
        !is_hull_foundation_kind(tile.kind)
            || preserved_hull.contains_key(&(tile.grid_x, tile.grid_y))
    });

    let mut next_id = ship.next_foundation_id();
    for ((grid_x, grid_y), (kind, rotation)) in desired_hull {
        if let Some((preserved_kind, preserved_rotation)) = preserved_hull.get(&(grid_x, grid_y)) {
            ship.replace_hull_tile(ShipFoundationTile::new(
                ship.hull_at(grid_x, grid_y)
                    .map(|tile| tile.id)
                    .unwrap_or(next_id),
                *preserved_kind,
                grid_x,
                grid_y,
                *preserved_rotation,
            ));
            continue;
        }
        ship.replace_hull_tile(ShipFoundationTile::new(
            next_id, kind, grid_x, grid_y, rotation,
        ));
        next_id += 1;
    }

    true
}

/// Chooses the hull foundation tile and orientation that best wraps one empty shell cell around
/// the occupied ship footprint.
fn auto_hull_kind_for_cell(
    occupied: &HashSet<(i32, i32)>,
    target: (i32, i32),
) -> Option<(ShipFoundationKind, u8)> {
    let north = occupied.contains(&(target.0, target.1 - 1));
    let south = occupied.contains(&(target.0, target.1 + 1));
    let west = occupied.contains(&(target.0 - 1, target.1));
    let east = occupied.contains(&(target.0 + 1, target.1));
    let northeast = occupied.contains(&(target.0 + 1, target.1 - 1));
    let northwest = occupied.contains(&(target.0 - 1, target.1 - 1));
    let southeast = occupied.contains(&(target.0 + 1, target.1 + 1));
    let southwest = occupied.contains(&(target.0 - 1, target.1 + 1));

    if (south && north) || (east && west) {
        None
    } else if north && east && northeast {
        Some((ShipFoundationKind::HullInnerCorner, 0))
    } else if south && east && southeast {
        Some((ShipFoundationKind::HullInnerCorner, 1))
    } else if south && west && southwest {
        Some((ShipFoundationKind::HullInnerCorner, 2))
    } else if north && west && northwest {
        Some((ShipFoundationKind::HullInnerCorner, 3))
    } else if south {
        Some((ShipFoundationKind::Hull, 0))
    } else if west {
        Some((ShipFoundationKind::Hull, 1))
    } else if north {
        Some((ShipFoundationKind::Hull, 2))
    } else if east {
        Some((ShipFoundationKind::Hull, 3))
    } else if southeast {
        Some((ShipFoundationKind::HullOuterCorner, 0))
    } else if southwest {
        Some((ShipFoundationKind::HullOuterCorner, 1))
    } else if northwest {
        Some((ShipFoundationKind::HullOuterCorner, 2))
    } else if northeast {
        Some((ShipFoundationKind::HullOuterCorner, 3))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::apply_auto_hull_to_ship;
    use crate::{
        editor::input::selection::paste_clipboard_group,
        ship::{
            ModuleKind,
            ModuleVariant,
            ShipDefinition,
            ShipFoundationKind,
            ShipFoundationTile,
            ShipModule,
        },
        state::{EditorMode, EditorSelectionState, Progression, ShipModuleSnapshot},
    };

    #[test]
    fn auto_hull_adds_shell_tiles_around_structure() {
        let mut ship = ShipDefinition::empty("Hull Test");
        ship.replace_logistics_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        ship.replace_logistics_tile(ShipFoundationTile::new(
            2,
            ShipFoundationKind::Floor,
            1,
            0,
            0,
        ));
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        assert!(ship.hull_tiles.iter().any(|tile| {
            matches!(
                tile.kind,
                ShipFoundationKind::Hull
                    | ShipFoundationKind::HullInnerCorner
                    | ShipFoundationKind::HullOuterCorner
            )
        }));
    }

    #[test]
    fn auto_hull_builds_edges_and_outer_corners_around_single_component() {
        let mut ship = ShipDefinition::empty("Outer Shell Test");
        ship.replace_logistics_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let shell = ship
            .hull_tiles
            .iter()
            .filter(|tile| tile.kind != ShipFoundationKind::Floor)
            .map(|tile| {
                (
                    (tile.grid_x, tile.grid_y),
                    (tile.kind, tile.rotation_quadrants),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(
            shell.get(&(-1, -1)),
            Some(&(ShipFoundationKind::HullOuterCorner, 0))
        );
        assert_eq!(shell.get(&(0, -1)), Some(&(ShipFoundationKind::Hull, 0)));
        assert_eq!(
            shell.get(&(1, -1)),
            Some(&(ShipFoundationKind::HullOuterCorner, 1))
        );
        assert_eq!(shell.get(&(-1, 0)), Some(&(ShipFoundationKind::Hull, 3)));
        assert_eq!(shell.get(&(1, 0)), Some(&(ShipFoundationKind::Hull, 1)));
        assert_eq!(
            shell.get(&(-1, 1)),
            Some(&(ShipFoundationKind::HullOuterCorner, 3))
        );
        assert_eq!(shell.get(&(0, 1)), Some(&(ShipFoundationKind::Hull, 2)));
        assert_eq!(
            shell.get(&(1, 1)),
            Some(&(ShipFoundationKind::HullOuterCorner, 2))
        );
    }

    #[test]
    fn auto_hull_uses_inner_corner_for_l_shaped_structure() {
        let mut ship = ShipDefinition::empty("Inner Corner Test");
        ship.replace_logistics_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        ship.replace_logistics_tile(ShipFoundationTile::new(
            2,
            ShipFoundationKind::Floor,
            1,
            0,
            0,
        ));
        ship.replace_logistics_tile(ShipFoundationTile::new(
            3,
            ShipFoundationKind::Floor,
            0,
            1,
            0,
        ));
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));
        ship.replace_module(ShipModule::new(3, ModuleKind::Interior, 0, 1, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let corner = ship
            .hull_at(1, 1)
            .expect("expected derived inner corner at the L-shape notch");
        assert_eq!(corner.kind, ShipFoundationKind::HullInnerCorner);
        assert_eq!(corner.rotation_quadrants, 3);
    }

    #[test]
    fn pasting_group_consumes_variant_inventory_in_player_mode() {
        let mut ship = ShipDefinition::empty("Paste Test");
        ship.replace_hull_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Hull,
            4,
            5,
            0,
        ));
        let mut progression = Progression::default();
        progression.add_ready_component(ModuleKind::Turret, ModuleVariant::BallisticTurret, 1);
        let mut selection_state = EditorSelectionState {
            clipboard: vec![ShipModuleSnapshot {
                kind: ModuleKind::Turret,
                variant: ModuleVariant::BallisticTurret,
                grid_x: 0,
                grid_y: 0,
                rotation_quadrants: 0,
                channel: 2,
            }],
            ..Default::default()
        };

        let pasted = paste_clipboard_group(
            &mut ship,
            &mut progression,
            EditorMode::Player,
            false,
            &mut selection_state,
            (4, 5),
        );

        assert!(pasted);
        assert_eq!(
            progression.ready_count(ModuleKind::Turret, ModuleVariant::BallisticTurret),
            0
        );
        assert!(ship.module_at(4, 5).is_some());
    }
}
