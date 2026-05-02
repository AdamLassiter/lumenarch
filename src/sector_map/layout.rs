use bevy::prelude::*;

use crate::{
    UI_BUTTON_RADIUS,
    state::{
        SectorMapDetailText, SectorMapLinkDash, SectorMapStatusText, SectorMapViewState,
        SectorMapNodeBorder, SectorNodeButton, SectorNodeKind, SectorNodeStatus, SectorState,
    },
};

pub(super) const MAP_CENTER_X: f32 = 360.0;
pub(super) const MAP_CENTER_Y: f32 = 230.0;
pub(super) const MAP_NODE_WIDTH: f32 = 170.0;
pub(super) const MAP_NODE_HEIGHT: f32 = 52.0;

pub(crate) fn sync_sector_map_layout(
    sector_state: Res<SectorState>,
    view_state: Res<SectorMapViewState>,
    mut node_query: Query<
        (&SectorNodeButton, &mut Node, &mut BackgroundColor, &mut BorderColor),
        (With<SectorMapNodeBorder>, Without<SectorMapLinkDash>),
    >,
    mut dash_query: Query<
        (&SectorMapLinkDash, &mut Node, &mut Transform, &mut BackgroundColor),
        Without<SectorMapNodeBorder>,
    >,
) {
    if !sector_state.is_changed() && !view_state.is_changed() {
        return;
    }

    for (button, mut node_style, mut background, mut border) in &mut node_query {
        let Some(node) = sector_state.node(button.node_id) else {
            continue;
        };
        let reachable = sector_state.is_reachable(node.id);
        let is_current = sector_state.current_node_id == node.id;
        let is_selected = sector_state.selected_node_id == Some(node.id);
        *node_style = projected_node(node.position, view_state.zoom, view_state.offset);
        *background = BackgroundColor(node_button_color(
            node.kind,
            node.status,
            reachable,
            is_current,
            is_selected,
        ));
        *border = BorderColor::all(node_border_color(is_current, reachable));
    }

    let current_position = sector_state
        .current_node()
        .map(|node| node.position)
        .unwrap_or([0.0, 0.0]);
    for (dash, mut node, mut transform, mut background) in &mut dash_query {
        let reachable = sector_state.is_reachable(dash.target_node_id);
        if let Some(target_node) = sector_state.node(dash.target_node_id) {
            let (next_node, next_transform) = projected_link_dash(
                current_position,
                target_node.position,
                view_state.zoom,
                view_state.offset,
                dash.dash_index,
                dash.dash_count,
            );
            *node = next_node;
            *transform = next_transform;
            *background = BackgroundColor(if reachable {
                Color::srgb(0.88, 0.70, 0.30)
            } else {
                Color::srgba(0.88, 0.70, 0.30, 0.0)
            });
        }
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
        "Current Node: {current}\nSelected Node: {selected}\nSeed: {}\nReachable Nodes: {}",
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

pub(super) fn node_border_color(is_current: bool, reachable: bool) -> Color {
    if is_current {
        Color::srgb(0.98, 0.88, 0.42)
    } else if reachable {
        Color::srgb(0.44, 0.58, 0.74)
    } else {
        Color::srgba(0.10, 0.10, 0.10, 0.0)
    }
}

pub(super) fn projected_node(position: [f32; 2], zoom: f32, offset: Vec2) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(position[0] * zoom + MAP_CENTER_X + offset.x),
        top: Val::Px(position[1] * zoom + MAP_CENTER_Y + offset.y),
        width: Val::Px(MAP_NODE_WIDTH * zoom.clamp(0.8, 1.35)),
        height: Val::Px(MAP_NODE_HEIGHT * zoom.clamp(0.8, 1.35)),
        border: UiRect::all(Val::Px(3.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(8.0)),
        border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
        ..default()
    }
}

pub(super) fn projected_link_dash(
    start: [f32; 2],
    end: [f32; 2],
    zoom: f32,
    offset: Vec2,
    dash_index: u8,
    dash_count: u8,
) -> (Node, Transform) {
    let dash_count = dash_count.max(1);
    let start = Vec2::new(start[0], start[1]) * zoom + Vec2::new(MAP_CENTER_X, MAP_CENTER_Y) + offset;
    let end = Vec2::new(end[0], end[1]) * zoom + Vec2::new(MAP_CENTER_X, MAP_CENTER_Y) + offset;
    let delta = end - start;
    let direction = if delta.length_squared() > 0.0 {
        delta.normalize()
    } else {
        Vec2::X
    };
    let progress = (dash_index as f32 + 0.5) / dash_count as f32;
    let dash_length = (delta.length() / dash_count as f32 * 0.55).max(6.0);
    let center = start + delta * progress;
    let angle = direction.y.atan2(direction.x);

    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(center.x - dash_length * 0.5),
            top: Val::Px(center.y - 1.5),
            width: Val::Px(dash_length),
            height: Val::Px(3.0),
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_z(angle)),
    )
}
