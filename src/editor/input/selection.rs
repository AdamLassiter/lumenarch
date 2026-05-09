use std::collections::HashMap;

use crate::{
    ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule},
    state::{EditorMode, EditorSelectionState, Progression, ShipModuleSnapshot},
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

pub(super) fn delete_selected_group(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    selection_state: &mut EditorSelectionState,
) -> bool {
    let selected = selected_or_all_modules(ship, &selection_state.selected_module_ids);
    if selected.is_empty() {
        return false;
    }

    for module in &selected {
        if mode == EditorMode::Player {
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

    if mode == EditorMode::Player {
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
