use std::collections::{HashMap, HashSet};

use super::paste_clipboard_group;
use crate::ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule};

/// Rebuilds the derived hull shell around all non-hull structure modules in the editor ship.
pub(super) fn apply_auto_hull_to_ship(ship: &mut ShipDefinition) -> bool {
    let base = ship
        .modules
        .iter()
        .filter(|module| !is_generated_hull_kind(module.kind))
        .cloned()
        .collect::<Vec<_>>();
    if base.is_empty() {
        return false;
    }

    let occupied = base
        .iter()
        .filter(|module| !is_manual_hull_kind(module.kind))
        .map(|module| (module.grid_x, module.grid_y))
        .collect::<HashSet<_>>();

    let existing_hull = ship
        .modules
        .iter()
        .filter(|module| is_manual_hull_kind(module.kind))
        .map(|module| {
            (
                (module.grid_x, module.grid_y),
                (module.kind, module.rotation_quadrants),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut candidates = HashSet::new();
    for module in &base {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let target = (module.grid_x + dx, module.grid_y + dy);
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

    ship.modules
        .retain(|module| !is_generated_hull_kind(module.kind));

    for ((grid_x, grid_y), (kind, rotation)) in desired_hull {
        if existing_hull.contains_key(&(grid_x, grid_y)) {
            continue;
        }
        let variant = ModuleVariant::default_for_kind(kind);
        let next_id = ship.next_module_id();
        let mut hull = ShipModule::new(next_id, kind, grid_x, grid_y, rotation);
        hull.variant = variant;
        ship.replace_module(hull);
    }

    true
}

fn is_generated_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )
}

fn is_manual_hull_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull
            | ModuleKind::HullInnerCorner
            | ModuleKind::HullOuterCorner
            | ModuleKind::Airlock
            | ModuleKind::Engine
            | ModuleKind::Turret
    )
}

/// Chooses the hull piece and orientation that best wraps one empty shell cell around structure.
fn auto_hull_kind_for_cell(
    occupied: &HashSet<(i32, i32)>,
    target: (i32, i32),
) -> Option<(ModuleKind, u8)> {
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
        Some((ModuleKind::HullInnerCorner, 0))
    } else if south && east && southeast {
        Some((ModuleKind::HullInnerCorner, 1))
    } else if south && west && southwest {
        Some((ModuleKind::HullInnerCorner, 2))
    } else if north && west && northwest {
        Some((ModuleKind::HullInnerCorner, 3))
    } else if south {
        Some((ModuleKind::Hull, 0))
    } else if west {
        Some((ModuleKind::Hull, 1))
    } else if north {
        Some((ModuleKind::Hull, 2))
    } else if east {
        Some((ModuleKind::Hull, 3))
    } else if southeast {
        Some((ModuleKind::HullOuterCorner, 0))
    } else if southwest {
        Some((ModuleKind::HullOuterCorner, 1))
    } else if northwest {
        Some((ModuleKind::HullOuterCorner, 2))
    } else if northeast {
        Some((ModuleKind::HullOuterCorner, 3))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_auto_hull_to_ship, paste_clipboard_group};
    use crate::{
        ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule},
        state::{EditorMode, EditorSelectionState, Progression, ShipModuleSnapshot},
    };

    #[test]
    fn auto_hull_adds_shell_tiles_around_structure() {
        let mut ship = ShipDefinition::empty("Hull Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        assert!(ship.modules.iter().any(|module| {
            matches!(
                module.kind,
                ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
            )
        }));
    }

    #[test]
    fn auto_hull_builds_edges_and_outer_corners_around_single_component() {
        let mut ship = ShipDefinition::empty("Outer Shell Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let shell = ship
            .modules
            .iter()
            .filter(|module| module.kind != ModuleKind::Core)
            .map(|module| {
                (
                    (module.grid_x, module.grid_y),
                    (module.kind, module.rotation_quadrants),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(
            shell.get(&(-1, -1)),
            Some(&(ModuleKind::HullOuterCorner, 0))
        );
        assert_eq!(shell.get(&(0, -1)), Some(&(ModuleKind::Hull, 0)));
        assert_eq!(shell.get(&(1, -1)), Some(&(ModuleKind::HullOuterCorner, 1)));
        assert_eq!(shell.get(&(-1, 0)), Some(&(ModuleKind::Hull, 3)));
        assert_eq!(shell.get(&(1, 0)), Some(&(ModuleKind::Hull, 1)));
        assert_eq!(shell.get(&(-1, 1)), Some(&(ModuleKind::HullOuterCorner, 3)));
        assert_eq!(shell.get(&(0, 1)), Some(&(ModuleKind::Hull, 2)));
        assert_eq!(shell.get(&(1, 1)), Some(&(ModuleKind::HullOuterCorner, 2)));
    }

    #[test]
    fn auto_hull_uses_inner_corner_for_l_shaped_structure() {
        let mut ship = ShipDefinition::empty("Inner Corner Test");
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));
        ship.replace_module(ShipModule::new(2, ModuleKind::Interior, 1, 0, 0));
        ship.replace_module(ShipModule::new(3, ModuleKind::Interior, 0, 1, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        let corner = ship
            .module_at(1, 1)
            .expect("expected derived inner corner at the L-shape notch");
        assert_eq!(corner.kind, ModuleKind::HullInnerCorner);
        assert_eq!(corner.rotation_quadrants, 3);
    }

    #[test]
    fn pasting_group_consumes_variant_inventory_in_player_mode() {
        let mut ship = ShipDefinition::empty("Paste Test");
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
