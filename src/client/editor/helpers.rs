use bevy::prelude::*;

use super::super::{
    HALF_TILE_SIZE,
    TILE_SIZE,
    TOOLBOX_WIDTH,
    state::{EditorMode, LastMissionReport},
};
use crate::ship::ModuleKind;

pub(super) fn editor_status_line(
    mode: EditorMode,
    entry_label: &str,
    ship_name: &str,
    selected_kind: &ModuleKind,
    selected_rotation: u8,
    module_count: usize,
    scrap_total: u32,
    last_mission_report: &LastMissionReport,
) -> String {
    let selected_cost = module_kind_cost(*selected_kind);
    let affordability = if scrap_total >= selected_cost {
        "ready"
    } else {
        "need more scrap"
    };
    let mission_summary = match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!(
            "\nLast Mission: {headline}\n{detail}\nScrap Awarded: {}\nTotal Scrap: {}\nHottest Module: {}\nFirst Disabled: {}\nRepairs / Stabilizations: {} / {}\nAutomation Used: {}\nARCH Program: {}\nARCH Invalid / Recent Writes: {} / {}\nRecovered Raw: {}\nProcessed / Used Charges: {} / {}\nTransfers / Processor Cycles: {} / {}\nLogistics Bottleneck: {}{}",
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
            "\nLast Mission: {headline}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap,
        ),
        _ => String::new(),
    };

    format!(
        "{}\nEntry: {entry_label}\nShip: {ship_name}\nSelected Tool: {selected_kind}\nRotation: {selected_rotation}\nPlaced Modules: {module_count}\nScrap: {scrap_total}\nPlacement Cost: {selected_cost} ({affordability}){mission_summary}",
        match mode {
            EditorMode::Player => "Player Refit",
            EditorMode::Enemy => "Enemy Ship Debug Editor",
        }
    )
}

pub(super) fn module_kind_cost(kind: ModuleKind) -> u32 {
    match kind {
        ModuleKind::Interior => 0,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner => 1,
        ModuleKind::Battery | ModuleKind::Cargo | ModuleKind::Airlock => 2,
        ModuleKind::Engine => 3,
        ModuleKind::Cockpit | ModuleKind::Computer | ModuleKind::Processor | ModuleKind::Turret => {
            4
        }
        ModuleKind::Reactor => 5,
        ModuleKind::Core => 6,
    }
}

pub(super) fn cursor_grid_position(
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

pub(super) fn grid_to_world(grid_x: i32, grid_y: i32, z: f32) -> Vec3 {
    Vec3::new(grid_x as f32 * TILE_SIZE, -(grid_y as f32) * TILE_SIZE, z)
}

pub(super) fn is_cursor_over_toolbox(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    cursor.x <= TOOLBOX_WIDTH
}

pub(super) fn is_cursor_over_editor_ui(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };

    if cursor.x <= TOOLBOX_WIDTH {
        return true;
    }

    let width = window.width();
    let height = window.height();

    let over_arch_panel = cursor.x >= TOOLBOX_WIDTH + 16.0
        && cursor.x <= TOOLBOX_WIDTH + 16.0 + 360.0
        && cursor.y >= height - 280.0;
    let over_status_panel = cursor.x >= width - 360.0 && cursor.y <= 220.0;
    let over_controls_panel = cursor.x >= width - 340.0 && cursor.y >= height - 200.0;

    over_arch_panel || over_status_panel || over_controls_panel
}

pub(super) fn sprite_path_for_kind(kind: &ModuleKind) -> String {
    format!("tiles/{}.png", kind.as_str())
}
