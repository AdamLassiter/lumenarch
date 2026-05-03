use bevy::prelude::*;

use super::super::{
    HALF_TILE_SIZE,
    TILE_SIZE,
    TOOLBOX_WIDTH,
    state::{EditorMode, LastMissionReport},
};
use crate::{
    ship::{ModuleKind, ModuleSpec, ModuleVariant},
    state::DemoProgression,
};

pub(crate) fn editor_status_line(
    mode: EditorMode,
    entry_label: &str,
    ship_name: &str,
    selected_kind: &ModuleKind,
    selected_variant: ModuleVariant,
    selected_rotation: u8,
    selected_channel: u8,
    module_count: usize,
    scrap_total: u32,
    progression: &DemoProgression,
) -> String {
    let ready_count = progression.ready_count(*selected_kind, selected_variant);
    let damaged_count = progression.damaged_count(*selected_kind, selected_variant);
    let repair_cost = module_kind_cost(*selected_kind, selected_variant).max(1);
    let availability = if ready_count > 0 {
        "ready"
    } else if damaged_count > 0 {
        "repair needed"
    } else {
        "none available"
    };

    format!(
        "{}\nEntry: {entry_label}\nShip: {ship_name}\nSelected Tool: {selected_kind} / {}\nRotation: {selected_rotation}\nChannel: {}\nPlaced Modules: {module_count}\nScrap: {scrap_total}\nAvailable: ready {} / damaged {}\nRepair Cost: {} ({availability})",
        match mode {
            EditorMode::Player => "Player Refit",
            EditorMode::Enemy => "Enemy Ship Debug Editor",
        },
        selected_variant.display_name(),
        selected_channel,
        ready_count,
        damaged_count,
        repair_cost,
    )
}

pub(crate) fn editor_mission_report_text(last_mission_report: &LastMissionReport) -> String {
    match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!(
            "Last Mission: {headline}\n{detail}\nScrap Awarded: {}\nTotal Scrap: {}\nHottest Module: {}\nFirst Disabled: {}\nRepairs / Stabilizations: {} / {}\nAutomation Used: {}\nARCH Program: {}\nARCH Invalid / Recent Writes: {} / {}\nRecovered Raw: {}\nProcessed / Used Charges: {} / {}\nTransfers / Processor Cycles: {} / {}\nLogistics Bottleneck: {}{}",
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
            "Last Mission: {headline}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap,
        ),
        _ => "No completed sorties yet.".to_string(),
    }
}

pub(crate) fn module_kind_cost(kind: ModuleKind, variant: ModuleVariant) -> u32 {
    ModuleSpec::for_module(kind, variant).placement_cost
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

pub(crate) fn is_cursor_over_toolbox(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    cursor.x <= TOOLBOX_WIDTH
}

pub(crate) fn is_cursor_over_editor_ui(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };

    if cursor.x <= TOOLBOX_WIDTH {
        return true;
    }

    let width = window.width();
    let height = window.height();

    let over_arch_panel = cursor.x >= TOOLBOX_WIDTH + 16.0
        && cursor.x <= TOOLBOX_WIDTH + 16.0 + 640.0
        && cursor.y >= height - 360.0;
    let over_status_panel = cursor.x >= width - 380.0 && cursor.y <= 420.0;
    let over_controls_panel = cursor.x <= 380.0 && cursor.y >= height - 180.0;

    over_arch_panel || over_status_panel || over_controls_panel
}

pub(crate) fn sprite_path_for_kind(kind: &ModuleKind, variant: ModuleVariant) -> String {
    let _ = variant;
    let asset_name = match kind {
        ModuleKind::Shield => "battery",
        _ => kind.as_str(),
    };
    format!("tiles/{asset_name}.png")
}
