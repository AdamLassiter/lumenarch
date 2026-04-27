use bevy::prelude::*;

use crate::ship::ModuleKind;

use super::super::{
    state::LastMissionReport, HALF_TILE_SIZE, TILE_SIZE, TOOLBOX_WIDTH,
};

pub(super) fn editor_status_line(
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
            "\nLast Mission: {headline}\n{detail}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap,
        ),
        (Some(headline), None) => format!(
            "\nLast Mission: {headline}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap,
        ),
        _ => String::new(),
    };

    format!(
        "Editor Status\nShip: {ship_name}\nSelected Tool: {selected_kind}\nRotation: {selected_rotation}\nPlaced Modules: {module_count}\nScrap: {scrap_total}\nPlacement Cost: {selected_cost} ({affordability}){mission_summary}"
    )
}

pub(super) fn module_kind_cost(kind: ModuleKind) -> u32 {
    match kind {
        ModuleKind::Interior => 0,
        ModuleKind::Hull | ModuleKind::HullCorner => 1,
        ModuleKind::Battery | ModuleKind::Cargo | ModuleKind::Airlock => 2,
        ModuleKind::Engine => 3,
        ModuleKind::Cockpit | ModuleKind::Turret => 4,
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

pub(super) fn sprite_path_for_kind(kind: &ModuleKind) -> String {
    format!("tiles/{}.png", kind.as_str())
}
