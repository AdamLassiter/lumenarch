use bevy::prelude::*;

use crate::state::{
    SectorMapDetailText, SectorMapStatusText, SectorMapViewState, SectorNodeButton,
    SectorNodeKind, SectorNodeStatus, SectorState,
};

pub(super) const MAP_CENTER_X: f32 = 360.0;
pub(super) const MAP_CENTER_Y: f32 = 230.0;
pub(super) const MAP_NODE_WIDTH: f32 = 170.0;
pub(super) const MAP_NODE_HEIGHT: f32 = 52.0;

pub(crate) fn sync_sector_map_layout(
    sector_state: Res<SectorState>,
    view_state: Res<SectorMapViewState>,
    mut node_query: Query<(&SectorNodeButton, &mut Node)>,
) {
    if !sector_state.is_changed() && !view_state.is_changed() {
        return;
    }

    for (button, mut node_style) in &mut node_query {
        let Some(node) = sector_state.node(button.node_id) else {
            continue;
        };
        *node_style = projected_node(node.position, view_state.zoom, view_state.offset);
    }
}

pub(crate) fn update_sector_map_text(
    sector_state: Res<SectorState>,
    mut status_query: Query<&mut Text, (With<SectorMapStatusText>, Without<SectorMapDetailText>)>,
    mut detail_query: Query<&mut Text, (With<SectorMapDetailText>, Without<SectorMapStatusText>)>,
) {
    if !sector_state.is_changed() {
        return;
    }

    for mut text in &mut status_query {
        **text = sector_status_text(&sector_state);
    }
    for mut text in &mut detail_query {
        **text = sector_detail_text(&sector_state);
    }
}

pub(super) fn sector_status_text(sector_state: &SectorState) -> String {
    let current = sector_state
        .current_node()
        .map(|node| node.label.as_str())
        .unwrap_or("Unknown");
    let selected = sector_state
        .selected_node()
        .map(|node| node.label.as_str())
        .unwrap_or("None");

    format!(
        "Current Node: {current}\nSelected Node: {selected}\nSeed: {}\nReachable Nodes: {}\nControls: scroll zoom, middle drag pan",
        sector_state.seed,
        sector_state.available_neighbors().len(),
    )
}

pub(super) fn sector_detail_text(sector_state: &SectorState) -> String {
    let Some(node) = sector_state.selected_node() else {
        return "Select a reachable node to inspect its encounter profile.".to_string();
    };

    format!(
        "{}\nType: {}\nRisk: {}\nStatus: {}\nReward: {}\nHostiles: {}\nSalvage: {}\nAmbient Heat / Electrical: {} / {}\nMultiplier: x{}",
        node.label,
        node.kind.as_str(),
        node.risk_tier,
        node.status.as_str(),
        node.reward_hint,
        node.encounter.hostile_count,
        node.encounter.salvage_value,
        node.encounter.ambient_heat_pressure,
        node.encounter.ambient_electrical_pressure,
        node.encounter.reward_multiplier,
    )
}

pub(super) fn node_button_color(
    kind: SectorNodeKind,
    status: SectorNodeStatus,
    reachable: bool,
    is_current: bool,
    is_selected: bool,
) -> Color {
    if is_current {
        return Color::srgb(0.76, 0.58, 0.22);
    }
    if is_selected {
        return Color::srgb(0.28, 0.62, 0.84);
    }
    if !reachable {
        return Color::srgb(0.20, 0.22, 0.28);
    }
    match status {
        SectorNodeStatus::Exhausted => Color::srgb(0.26, 0.26, 0.30),
        SectorNodeStatus::Completed => Color::srgb(0.20, 0.38, 0.24),
        SectorNodeStatus::Failed => Color::srgb(0.44, 0.20, 0.18),
        SectorNodeStatus::Fresh => match kind {
            SectorNodeKind::HubStation => Color::srgb(0.44, 0.34, 0.18),
            SectorNodeKind::TestRange => Color::srgb(0.24, 0.48, 0.54),
            SectorNodeKind::SalvageField => Color::srgb(0.18, 0.46, 0.26),
            SectorNodeKind::HostileHold => Color::srgb(0.56, 0.24, 0.20),
            SectorNodeKind::UnstableDerelict => Color::srgb(0.26, 0.34, 0.56),
        },
    }
}

pub(super) fn projected_node(position: [f32; 2], zoom: f32, offset: Vec2) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(position[0] * zoom + MAP_CENTER_X + offset.x),
        top: Val::Px(position[1] * zoom + MAP_CENTER_Y + offset.y),
        width: Val::Px(MAP_NODE_WIDTH * zoom.clamp(0.8, 1.35)),
        height: Val::Px(MAP_NODE_HEIGHT * zoom.clamp(0.8, 1.35)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(10.0)),
        ..default()
    }
}
