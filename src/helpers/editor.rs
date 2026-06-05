use std::collections::BTreeMap;

use bevy::{prelude::*, ui::UiGlobalTransform};

use crate::{
    HALF_TILE_SIZE,
    TILE_SIZE,
    ship::{
        ModuleKind,
        ModuleSpec,
        ModuleVariant,
        ShipDefinition,
        ShipFoundationKind,
        ShipFoundationTile,
    },
    state::{
        EditorLayer,
        EditorMode,
        EditorPlacementBlocker,
        EditorSelectionState,
        EditorToolMode,
        EditorToolboxPanel,
        LastMissionReport,
        Progression,
        SectorState,
    },
};

pub(crate) fn editor_status_line(
    mode: EditorMode,
    tool_mode: EditorToolMode,
    active_layer: EditorLayer,
    entry_label: &str,
    ship_name: &str,
    selected_kind: &ModuleKind,
    selected_foundation_kind: ShipFoundationKind,
    selected_variant: ModuleVariant,
    selected_rotation: u8,
    selected_channel: u8,
    ignore_component_limits: bool,
    module_count: usize,
    scrap_total: u32,
    progression: &Progression,
    editor_ship: &ShipDefinition,
    selection_state: &EditorSelectionState,
) -> String {
    let ready_count = progression.ready_count(*selected_kind, selected_variant);
    let damaged_count = progression.damaged_count(*selected_kind, selected_variant);
    let repair_cost = module_kind_cost(*selected_kind, selected_variant).max(1);
    let availability = if ignore_component_limits {
        "ignored"
    } else if ready_count > 0 {
        "ready"
    } else if damaged_count > 0 {
        "repair needed"
    } else {
        "none available"
    };

    let docking_status = if editor_ship.has_module_kind(ModuleKind::Airlock) {
        "ready"
    } else {
        "missing Airlock"
    };

    format!(
        "{}\nEntry: {entry_label}\nShip: {ship_name}\nMode: {}\nLayer: {}\nBuild: {} / {}\nRotation: {selected_rotation}\nChannel: {}\nPlaced Modules: {module_count}\nDocking: {docking_status}\nSelection: {}\nScrap: {scrap_total}\nComponent Limits: {}\nAvailable: ready {} / damaged {}\nRepair Cost: {} ({availability})",
        match mode {
            EditorMode::Player => "Player Refit",
            EditorMode::Enemy => "Enemy Ship Debug Editor",
            EditorMode::Station => "Station Debug Editor",
        },
        match tool_mode {
            EditorToolMode::Build => "Build",
            EditorToolMode::Select => "Select",
        },
        match active_layer {
            EditorLayer::Logistics => "Logistics",
            EditorLayer::Hull => "Hull",
            EditorLayer::Components => "Components",
        },
        editor_selected_build_label(active_layer, *selected_kind, selected_foundation_kind),
        selected_variant.display_name(),
        selected_channel,
        selection_summary(editor_ship, selection_state),
        if ignore_component_limits {
            "ignored"
        } else {
            "enforced"
        },
        ready_count,
        damaged_count,
        repair_cost,
    )
}

pub(crate) fn editor_mission_report_text(last_mission_report: &LastMissionReport) -> String {
    match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!(
            "Last Mission: {headline}\n{detail}\nScrap Awarded: {}\nTotal Scrap: {}\nHottest Module: {}\nFirst Disabled: {}\nRepairs / Stabilizations: {} / {}\nAutomation Used: {}\nARCH Program: {}\nARCH Invalid / Recent Writes: {} / {}\nRecovered Raw / Components: {} / {}\nRecovered Artifacts: {}\nObjective: {}\nProcessed / Used Charges: {} / {}\nTransfers / Processor Cycles: {} / {}\nLogistics Bottleneck: {}{}",
            last_mission_report.scrap_awarded,
            last_mission_report.total_scrap,
            last_mission_report
                .hottest_module
                .as_deref()
                .unwrap_or("n/a"),
            last_mission_report
                .first_disabled_module
                .as_deref()
                .unwrap_or("n/a"),
            last_mission_report.repairs_performed,
            last_mission_report.stabilizations_performed,
            if last_mission_report.automation_used {
                "yes"
            } else {
                "no"
            },
            last_mission_report
                .arch_primary_program
                .as_deref()
                .unwrap_or("n/a"),
            last_mission_report.arch_invalid_executions,
            if last_mission_report.arch_recent_writes.is_empty() {
                "none".to_string()
            } else {
                last_mission_report.arch_recent_writes.join(", ")
            },
            last_mission_report.recovered_raw_salvage,
            last_mission_report.recovered_damaged_components,
            if last_mission_report.recovered_artifacts.is_empty() {
                "none".to_string()
            } else {
                last_mission_report
                    .recovered_artifacts
                    .iter()
                    .map(|artifact| artifact.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            last_mission_report
                .contract_objective_status
                .as_deref()
                .unwrap_or("n/a"),
            last_mission_report.processed_repair_charge,
            last_mission_report.consumed_repair_charge,
            last_mission_report.transfer_count,
            last_mission_report.processor_cycles,
            last_mission_report
                .logistics_bottleneck
                .as_deref()
                .unwrap_or("none"),
            if last_mission_report.redesign_hints.is_empty() {
                String::new()
            } else {
                format!(
                    "\nHints:\n- {}",
                    last_mission_report.redesign_hints.join("\n- ")
                )
            }
        ),
        (Some(headline), None) => format!(
            "Last Mission: {headline}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap,
        ),
        _ => "No completed sorties yet.".to_string(),
    }
}

pub(crate) fn enemy_config_references_text(
    sector_state: &SectorState,
    enemy_library_state: &crate::state::EnemyShipLibraryState,
) -> String {
    let mut references: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in &sector_state.nodes {
        for enemy_id in &node.encounter.enemy_ship_ids {
            references
                .entry(enemy_id.clone())
                .or_default()
                .push(format!("{} {}", node.id, node.label));
        }
    }

    let mut lines = vec!["Config Enemy References".to_string()];
    if references.is_empty() {
        lines.push("No enemy ids referenced by sector config.".to_string());
    }
    let mut referenced_ids = Vec::new();
    for (enemy_id, nodes) in references {
        referenced_ids.push(enemy_id.clone());
        let library_label = enemy_library_state
            .library
            .find_by_id(&enemy_id)
            .map(|entry| entry.display_name.as_str())
            .unwrap_or("missing from library");
        let status = enemy_library_state
            .entry_statuses
            .get(&enemy_id)
            .map(|status| match status {
                crate::ship::enemy::EnemyShipEntryValidationStatus::Valid => "valid",
                crate::ship::enemy::EnemyShipEntryValidationStatus::RepairedInMemory => {
                    "repaired in memory"
                }
                crate::ship::enemy::EnemyShipEntryValidationStatus::Invalid => "invalid",
            })
            .unwrap_or(
                if enemy_library_state.library.find_by_id(&enemy_id).is_some() {
                    "valid"
                } else {
                    "missing"
                },
            );
        lines.push(format!(
            "{enemy_id} - {library_label} [{status}]\n  used by: {}",
            nodes.join(", ")
        ));
    }
    for entry in &enemy_library_state.library.entries {
        if !referenced_ids.iter().any(|id| id == &entry.id) {
            lines.push(format!(
                "{} - {} [unreferenced]",
                entry.id, entry.display_name
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn module_kind_cost(kind: ModuleKind, variant: ModuleVariant) -> u32 {
    ModuleSpec::for_module(kind, variant).placement_cost
}

pub(crate) fn selection_summary(
    editor_ship: &ShipDefinition,
    selection_state: &EditorSelectionState,
) -> String {
    let mut names = editor_ship
        .modules
        .iter()
        .filter(|module| {
            selection_state.selected_module_ids.is_empty()
                || selection_state.selected_module_ids.contains(&module.id)
        })
        .map(|module| module.display_name())
        .collect::<Vec<_>>();
    if !selection_state.selected_foundation_ids.is_empty() {
        names.extend(
            editor_ship
                .foundation_tiles
                .iter()
                .chain(editor_ship.hull_tiles.iter())
                .filter(|tile| selection_state.selected_foundation_ids.contains(&tile.id))
                .map(|tile| tile.display_name().to_string()),
        );
    }
    if names.is_empty() {
        return "none".to_string();
    }
    names.sort();
    if names.len() <= 3 {
        return names.join(", ");
    }
    format!("{}, {} more", names[..3].join(", "), names.len() - 3)
}

pub(crate) fn variant_inventory_label(
    mode: EditorMode,
    progression: &Progression,
    kind: ModuleKind,
    variant: ModuleVariant,
) -> String {
    match mode {
        EditorMode::Player => format!(
            "ready {} / damaged {}",
            progression.ready_count(kind, variant),
            progression.damaged_count(kind, variant),
        ),
        EditorMode::Enemy => "ready inf / damaged inf".to_string(),
        EditorMode::Station => "station tool".to_string(),
    }
}

pub(crate) fn variant_tooltip_text(
    mode: EditorMode,
    progression: &Progression,
    kind: ModuleKind,
    variant: ModuleVariant,
) -> String {
    let spec = ModuleSpec::for_module(kind, variant);
    format!(
        "{}\n{}\nCost {}  Integrity {}\nInventory: {}",
        variant.display_name(),
        module_family_label(kind),
        spec.placement_cost,
        spec.integrity,
        variant_inventory_label(mode, progression, kind, variant),
    )
}

pub(crate) fn module_family_label(kind: ModuleKind) -> &'static str {
    match kind {
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner => {
            "Hull and structure"
        }
        ModuleKind::Interior => "Interior and walkways",
        ModuleKind::Core => "Core systems",
        ModuleKind::Cockpit | ModuleKind::Computer => "Control and compute",
        ModuleKind::Detector => "Automation and sensors",
        ModuleKind::Processor | ModuleKind::Cargo => "Logistics and utility",
        ModuleKind::Airlock => "Exterior access",
        ModuleKind::Reactor | ModuleKind::Battery => "Power systems",
        ModuleKind::Engine => "Propulsion",
        ModuleKind::Turret | ModuleKind::Shield => "Combat systems",
        ModuleKind::InteriorWall => "Interior walls",
        ModuleKind::JunctionBox | ModuleKind::Valve | ModuleKind::O2Generator => {
            "Engineering control"
        }
    }
}

pub(crate) fn editor_selected_build_label(
    active_layer: EditorLayer,
    selected_kind: ModuleKind,
    selected_foundation_kind: ShipFoundationKind,
) -> String {
    match active_layer {
        EditorLayer::Logistics => selected_foundation_kind.display_name().to_string(),
        EditorLayer::Hull => {
            if module_belongs_to_hull_layer(selected_kind) {
                selected_kind.to_string()
            } else {
                selected_foundation_kind.display_name().to_string()
            }
        }
        EditorLayer::Components => selected_kind.to_string(),
    }
}

pub(crate) fn foundation_family_label(kind: ShipFoundationKind) -> &'static str {
    match kind {
        ShipFoundationKind::Floor => "Deck foundation",
        ShipFoundationKind::Hull
        | ShipFoundationKind::HullInnerCorner
        | ShipFoundationKind::HullOuterCorner => "Hull and structure",
        ShipFoundationKind::Wire => "Power routing",
        ShipFoundationKind::OxygenDuct => "Oxygen ducting",
        ShipFoundationKind::PipeRawSalvage
        | ShipFoundationKind::PipeRepairCharge
        | ShipFoundationKind::PipeFuel
        | ShipFoundationKind::PipeAmmunition
        | ShipFoundationKind::PipeOxygen => "Resource pipes",
    }
}

pub(crate) fn is_hull_module_kind(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )
}

pub(crate) fn is_hull_foundation_kind(kind: ShipFoundationKind) -> bool {
    matches!(
        kind,
        ShipFoundationKind::Hull
            | ShipFoundationKind::HullInnerCorner
            | ShipFoundationKind::HullOuterCorner
    )
}

pub(crate) fn hull_foundation_kind_for_module(kind: ModuleKind) -> Option<ShipFoundationKind> {
    match kind {
        ModuleKind::Hull => Some(ShipFoundationKind::Hull),
        ModuleKind::HullInnerCorner => Some(ShipFoundationKind::HullInnerCorner),
        ModuleKind::HullOuterCorner => Some(ShipFoundationKind::HullOuterCorner),
        _ => None,
    }
}

pub(crate) fn module_requires_hull_foundation(kind: ModuleKind) -> bool {
    matches!(
        kind,
        ModuleKind::Airlock | ModuleKind::Engine | ModuleKind::Turret
    )
}

pub(crate) fn module_belongs_to_hull_layer(kind: ModuleKind) -> bool {
    module_requires_hull_foundation(kind)
}

pub(crate) fn module_belongs_to_components_layer(kind: ModuleKind) -> bool {
    !is_hull_module_kind(kind) && !module_belongs_to_hull_layer(kind)
}

pub(crate) fn logistics_supports_component(foundation_kind: ShipFoundationKind) -> bool {
    !is_hull_foundation_kind(foundation_kind)
}

pub(crate) fn logistics_tile_must_be_enclosed(kind: ShipFoundationKind) -> bool {
    kind == ShipFoundationKind::Floor
}

pub(crate) fn foundation_supports_module(
    logistics_kind: Option<ShipFoundationKind>,
    hull_kind: Option<ShipFoundationKind>,
    module_kind: ModuleKind,
) -> bool {
    if is_hull_module_kind(module_kind) {
        return false;
    }
    if module_requires_hull_foundation(module_kind) {
        return hull_kind.is_some_and(is_hull_foundation_kind);
    }
    logistics_kind.is_some_and(logistics_supports_component)
}

pub(crate) fn normalize_editor_ship_layers(ship: &mut ShipDefinition) -> bool {
    let mut changed = false;
    let mut next_foundation_id = ship.next_foundation_id();
    let legacy_hull_modules = ship
        .modules
        .iter()
        .filter_map(|module| {
            hull_foundation_kind_for_module(module.kind).map(|foundation_kind| {
                let tile = ShipFoundationTile::new(
                    next_foundation_id,
                    foundation_kind,
                    module.grid_x,
                    module.grid_y,
                    module.rotation_quadrants,
                );
                next_foundation_id += 1;
                (module.id, tile)
            })
        })
        .collect::<Vec<_>>();

    if !legacy_hull_modules.is_empty() {
        for (_, tile) in &legacy_hull_modules {
            ship.replace_hull_tile(tile.clone());
        }
        let legacy_ids = legacy_hull_modules
            .into_iter()
            .map(|(id, _)| id)
            .collect::<std::collections::HashSet<_>>();
        ship.modules
            .retain(|module| !legacy_ids.contains(&module.id));
        changed = true;
    }

    let legacy_hull_foundations = ship
        .foundation_tiles
        .iter()
        .filter(|tile| is_hull_foundation_kind(tile.kind))
        .cloned()
        .collect::<Vec<_>>();
    if !legacy_hull_foundations.is_empty() {
        for tile in &legacy_hull_foundations {
            ship.replace_hull_tile(tile.clone());
        }
        ship.foundation_tiles
            .retain(|tile| !is_hull_foundation_kind(tile.kind));
        changed = true;
    }

    let missing_foundations = ship
        .modules
        .iter()
        .filter_map(|module| {
            if module_requires_hull_foundation(module.kind) {
                ship.hull_at(module.grid_x, module.grid_y)
                    .is_none()
                    .then(|| {
                        let tile = ShipFoundationTile::new(
                            next_foundation_id,
                            ShipFoundationKind::Hull,
                            module.grid_x,
                            module.grid_y,
                            module.rotation_quadrants,
                        );
                        next_foundation_id += 1;
                        (true, tile)
                    })
            } else {
                ship.logistics_at(module.grid_x, module.grid_y)
                    .is_none()
                    .then(|| {
                        let tile = ShipFoundationTile::new(
                            next_foundation_id,
                            ShipFoundationKind::Floor,
                            module.grid_x,
                            module.grid_y,
                            module.rotation_quadrants,
                        );
                        next_foundation_id += 1;
                        (false, tile)
                    })
            }
        })
        .collect::<Vec<_>>();

    if !missing_foundations.is_empty() {
        for (is_hull, tile) in missing_foundations {
            if is_hull {
                ship.replace_hull_tile(tile);
            } else {
                ship.replace_logistics_tile(tile);
            }
        }
        changed = true;
    }

    let exterior_modules = ship
        .modules
        .iter()
        .filter(|module| module_requires_hull_foundation(module.kind))
        .map(|module| (module.grid_x, module.grid_y, module.rotation_quadrants))
        .collect::<Vec<_>>();
    for (grid_x, grid_y, rotation_quadrants) in exterior_modules {
        let Some(tile) = ship.hull_at_mut(grid_x, grid_y) else {
            continue;
        };
        if !is_hull_foundation_kind(tile.kind) {
            continue;
        }
        if tile.rotation_quadrants != rotation_quadrants {
            tile.rotation_quadrants = rotation_quadrants;
            changed = true;
        }
    }

    changed
}

pub(crate) fn cursor_grid_position(
    window: &Window,
    (camera, camera_transform): (&Camera, &GlobalTransform),
) -> Option<(i32, i32)> {
    let cursor = window.cursor_position()?;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    Some((
        ((world.x + HALF_TILE_SIZE) / TILE_SIZE).floor() as i32,
        ((-world.y + HALF_TILE_SIZE) / TILE_SIZE).floor() as i32,
    ))
}

pub(crate) fn grid_to_world(grid_x: i32, grid_y: i32, z: f32) -> Vec3 {
    Vec3::new(grid_x as f32 * TILE_SIZE, -(grid_y as f32) * TILE_SIZE, z)
}

fn cursor_over_ui_region<M: Component>(
    window: &Window,
    query: &Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<M>,
    >,
) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    query.iter().any(|(node, transform, visibility)| {
        visibility.is_none_or(|visibility| visibility.get())
            && node.contains_point(*transform, cursor)
    })
}

pub(crate) fn is_cursor_over_toolbox(
    window: &Window,
    toolbox_query: &Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorToolboxPanel>,
    >,
) -> bool {
    cursor_over_ui_region(window, toolbox_query)
}

pub(crate) fn is_cursor_over_editor_ui(
    window: &Window,
    blocker_query: &Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorPlacementBlocker>,
    >,
) -> bool {
    cursor_over_ui_region(window, blocker_query)
}

pub(crate) fn sprite_path_for_kind(kind: &ModuleKind, variant: ModuleVariant) -> String {
    let (category, asset_name) = match kind {
        ModuleKind::Cargo => match variant {
            ModuleVariant::FuelTank => ("components", "cargo_fuel_tank"),
            ModuleVariant::AmmoRack => ("components", "cargo_ammo_rack"),
            ModuleVariant::RawSalvageCrate => ("components", "cargo_raw_salvage"),
            ModuleVariant::RepairChargeRack => ("components", "cargo_repair_charge"),
            ModuleVariant::O2Canister => ("components", "cargo_o2_canister"),
            _ => ("components", "cargo"),
        },
        ModuleKind::Turret => ("hull", "hardpoint"),
        ModuleKind::Airlock | ModuleKind::Engine => ("hull", kind.as_str()),
        ModuleKind::Shield => ("components", "battery"),
        ModuleKind::Detector => ("components", "computer"),
        _ => ("components", kind.as_str()),
    };
    format!("tiles/{category}/{asset_name}.png")
}

#[cfg(test)]
mod tests {
    use super::sprite_path_for_kind;
    use crate::ship::{ModuleKind, ModuleVariant};

    #[test]
    fn hull_layer_components_use_hull_asset_paths() {
        assert_eq!(
            sprite_path_for_kind(&ModuleKind::Airlock, ModuleVariant::Standard),
            "tiles/hull/airlock.png"
        );
        assert_eq!(
            sprite_path_for_kind(&ModuleKind::Engine, ModuleVariant::Standard),
            "tiles/hull/engine.png"
        );
        assert_eq!(
            sprite_path_for_kind(&ModuleKind::Turret, ModuleVariant::LaserTurret),
            "tiles/hull/hardpoint.png"
        );
    }

    #[test]
    fn ordinary_components_stay_in_component_asset_paths() {
        assert_eq!(
            sprite_path_for_kind(&ModuleKind::Core, ModuleVariant::BasicCore),
            "tiles/components/core.png"
        );
        assert_eq!(
            sprite_path_for_kind(&ModuleKind::Cargo, ModuleVariant::FuelTank),
            "tiles/components/cargo_fuel_tank.png"
        );
    }
}
