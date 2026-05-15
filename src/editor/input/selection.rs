use std::collections::HashMap;

use crate::{
    helpers::editor::{foundation_supports_module, is_hull_foundation_kind, is_hull_module_kind},
    ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipFoundationTile, ShipModule},
    state::{
        EditorMode,
        EditorSelectionState,
        Progression,
        ShipFoundationSnapshot,
        ShipModuleSnapshot,
    },
};

pub(super) fn selected_or_all_modules(
    ship: &ShipDefinition,
    selected_module_ids: &[u64],
) -> Vec<ShipModule> {
    ship.modules
        .iter()
        .filter(|module| selected_module_ids.is_empty() || selected_module_ids.contains(&module.id))
        .cloned()
        .collect()
}

pub(super) fn module_snapshot(module: ShipModule) -> ShipModuleSnapshot {
    ShipModuleSnapshot {
        kind: module.kind,
        variant: module.variant,
        grid_x: module.grid_x,
        grid_y: module.grid_y,
        rotation_quadrants: module.rotation_quadrants,
        channel: module.channel,
    }
}

pub(super) fn foundation_snapshot(tile: ShipFoundationTile) -> ShipFoundationSnapshot {
    ShipFoundationSnapshot {
        kind: tile.kind,
        grid_x: tile.grid_x,
        grid_y: tile.grid_y,
        rotation_quadrants: tile.rotation_quadrants,
    }
}

pub(super) fn ship_anchor(ship: &ShipDefinition) -> (i32, i32) {
    ship.bounds()
        .map(|(min_x, _, min_y, _)| (min_x, min_y))
        .unwrap_or((0, 0))
}

pub(super) fn move_selected_group(
    ship: &mut ShipDefinition,
    selection_state: &EditorSelectionState,
    dx: i32,
    dy: i32,
) -> bool {
    if selection_state.selected_module_ids.is_empty() {
        return false;
    }

    let selected = ship
        .modules
        .iter()
        .filter(|module| selection_state.selected_module_ids.contains(&module.id))
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return false;
    }

    for module in &selected {
        let next_x = module.grid_x + dx;
        let next_y = module.grid_y + dy;
        if is_hull_module_kind(module.kind)
            || !foundation_supports_module(
                ship.logistics_at(next_x, next_y).map(|tile| tile.kind),
                ship.hull_at(next_x, next_y).map(|tile| tile.kind),
                module.kind,
            )
        {
            return false;
        }
        if let Some(blocker) = ship.module_at(next_x, next_y)
            && !selection_state.selected_module_ids.contains(&blocker.id)
        {
            return false;
        }
    }

    for module in &selected {
        if let Some(target) = ship.modules.iter_mut().find(|entry| entry.id == module.id) {
            target.grid_x += dx;
            target.grid_y += dy;
        }
    }
    true
}

pub(super) fn move_selected_foundation_group(
    ship: &mut ShipDefinition,
    selection_state: &EditorSelectionState,
    dx: i32,
    dy: i32,
) -> bool {
    if selection_state.selected_foundation_ids.is_empty() {
        return false;
    }
    let selected = ship
        .foundation_tiles
        .iter()
        .chain(ship.hull_tiles.iter())
        .filter(|tile| selection_state.selected_foundation_ids.contains(&tile.id))
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return false;
    }
    for tile in &selected {
        if ship.module_at(tile.grid_x, tile.grid_y).is_some() {
            return false;
        }
        let next_x = tile.grid_x + dx;
        let next_y = tile.grid_y + dy;
        if is_hull_foundation_kind(tile.kind) {
            if let Some(blocker) = ship.hull_at(next_x, next_y)
                && !selection_state
                    .selected_foundation_ids
                    .contains(&blocker.id)
            {
                return false;
            }
        } else if let Some(blocker) = ship.logistics_at(next_x, next_y)
            && !selection_state
                .selected_foundation_ids
                .contains(&blocker.id)
        {
            return false;
        }
        if let Some(module) = ship.module_at(next_x, next_y)
            && !foundation_supports_module(
                if is_hull_foundation_kind(tile.kind) {
                    ship.logistics_at(next_x, next_y)
                        .map(|existing| existing.kind)
                } else {
                    Some(tile.kind)
                },
                if is_hull_foundation_kind(tile.kind) {
                    Some(tile.kind)
                } else {
                    ship.hull_at(next_x, next_y).map(|existing| existing.kind)
                },
                module.kind,
            )
        {
            return false;
        }
    }
    for tile in &selected {
        let target = if is_hull_foundation_kind(tile.kind) {
            ship.hull_tiles.iter_mut().find(|entry| entry.id == tile.id)
        } else {
            ship.foundation_tiles
                .iter_mut()
                .find(|entry| entry.id == tile.id)
        };
        if let Some(target) = target {
            target.grid_x += dx;
            target.grid_y += dy;
        }
    }
    true
}

pub(super) fn delete_selected_group(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    ignore_component_limits: bool,
    selection_state: &mut EditorSelectionState,
) -> bool {
    if !selection_state.selected_foundation_ids.is_empty() {
        let selected = selection_state.selected_foundation_ids.clone();
        if ship
            .foundation_tiles
            .iter()
            .chain(ship.hull_tiles.iter())
            .filter(|tile| selected.contains(&tile.id))
            .any(|tile| ship.module_at(tile.grid_x, tile.grid_y).is_some())
        {
            return false;
        }
        ship.foundation_tiles
            .retain(|tile| !selected.contains(&tile.id));
        ship.hull_tiles.retain(|tile| !selected.contains(&tile.id));
        selection_state.selected_foundation_ids.clear();
        return true;
    }

    let selected = selected_or_all_modules(ship, &selection_state.selected_module_ids);
    if selected.is_empty() {
        return false;
    }

    for module in &selected {
        if mode == EditorMode::Player && !ignore_component_limits {
            progression.add_ready_component(module.kind, module.variant, 1);
        }
        ship.modules.retain(|entry| entry.id != module.id);
    }
    selection_state.selected_module_ids.clear();
    true
}

pub(super) fn paste_clipboard_group(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    ignore_component_limits: bool,
    selection_state: &mut EditorSelectionState,
    anchor: (i32, i32),
) -> bool {
    if selection_state.clipboard.is_empty() {
        return false;
    }
    let min_x = selection_state
        .clipboard
        .iter()
        .map(|module| module.grid_x)
        .min()
        .unwrap_or(anchor.0);
    let min_y = selection_state
        .clipboard
        .iter()
        .map(|module| module.grid_y)
        .min()
        .unwrap_or(anchor.1);

    let planned = selection_state
        .clipboard
        .iter()
        .map(|module| {
            (
                module,
                anchor.0 + (module.grid_x - min_x),
                anchor.1 + (module.grid_y - min_y),
            )
        })
        .collect::<Vec<_>>();

    for (_, grid_x, grid_y) in &planned {
        if ship.module_at(*grid_x, *grid_y).is_some() {
            return false;
        }
    }
    if planned.iter().any(|(module, grid_x, grid_y)| {
        is_hull_module_kind(module.kind)
            || !foundation_supports_module(
                ship.logistics_at(*grid_x, *grid_y).map(|tile| tile.kind),
                ship.hull_at(*grid_x, *grid_y).map(|tile| tile.kind),
                module.kind,
            )
    }) {
        return false;
    }

    if mode == EditorMode::Player && !ignore_component_limits {
        let mut needed = HashMap::<(ModuleKind, ModuleVariant), u32>::new();
        for (module, _, _) in &planned {
            *needed.entry((module.kind, module.variant)).or_default() += 1;
        }
        if needed
            .into_iter()
            .any(|((kind, variant), amount)| progression.ready_count(kind, variant) < amount)
        {
            return false;
        }
        for (module, _, _) in &planned {
            progression.try_consume_ready_component(module.kind, module.variant);
        }
    }

    selection_state.selected_module_ids.clear();
    for (module, grid_x, grid_y) in planned {
        let mut next = ShipModule::new(
            ship.next_module_id(),
            module.kind,
            grid_x,
            grid_y,
            module.rotation_quadrants,
        );
        next.variant = module.variant;
        next.channel = module.channel;
        let new_id = next.id;
        ship.replace_module(next);
        selection_state.selected_module_ids.push(new_id);
    }
    true
}

pub(super) fn paste_foundation_clipboard_group(
    ship: &mut ShipDefinition,
    selection_state: &mut EditorSelectionState,
    anchor: (i32, i32),
) -> bool {
    if selection_state.foundation_clipboard.is_empty() {
        return false;
    }
    let min_x = selection_state
        .foundation_clipboard
        .iter()
        .map(|tile| tile.grid_x)
        .min()
        .unwrap_or(anchor.0);
    let min_y = selection_state
        .foundation_clipboard
        .iter()
        .map(|tile| tile.grid_y)
        .min()
        .unwrap_or(anchor.1);

    let planned = selection_state
        .foundation_clipboard
        .iter()
        .map(|tile| {
            (
                tile,
                anchor.0 + (tile.grid_x - min_x),
                anchor.1 + (tile.grid_y - min_y),
            )
        })
        .collect::<Vec<_>>();

    for (tile, grid_x, grid_y) in &planned {
        if is_hull_foundation_kind(tile.kind) {
            if ship.hull_at(*grid_x, *grid_y).is_some() {
                return false;
            }
        } else if ship.logistics_at(*grid_x, *grid_y).is_some() {
            return false;
        }
    }
    if planned.iter().any(|(tile, grid_x, grid_y)| {
        ship.module_at(*grid_x, *grid_y).is_some_and(|module| {
            !foundation_supports_module(
                if is_hull_foundation_kind(tile.kind) {
                    ship.logistics_at(*grid_x, *grid_y)
                        .map(|existing| existing.kind)
                } else {
                    Some(tile.kind)
                },
                if is_hull_foundation_kind(tile.kind) {
                    Some(tile.kind)
                } else {
                    ship.hull_at(*grid_x, *grid_y).map(|existing| existing.kind)
                },
                module.kind,
            )
        })
    }) {
        return false;
    }

    selection_state.selected_foundation_ids.clear();
    selection_state.selected_module_ids.clear();
    for (tile, grid_x, grid_y) in planned {
        let next = ShipFoundationTile::new(
            ship.next_foundation_id(),
            tile.kind,
            grid_x,
            grid_y,
            tile.rotation_quadrants,
        );
        let new_id = next.id;
        if is_hull_foundation_kind(next.kind) {
            ship.replace_hull_tile(next);
        } else {
            ship.replace_logistics_tile(next);
        }
        selection_state.selected_foundation_ids.push(new_id);
    }
    true
}
