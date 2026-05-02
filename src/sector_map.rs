use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use super::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    state::{
        BackToStationButton,
        LaunchEncounterButton,
        SectorMapCanvas,
        SectorMapDetailText,
        SectorMapPanState,
        SectorMapRoot,
        SectorMapStatusText,
        SectorMapViewState,
        SectorNodeButton,
        SectorNodeKind,
        SectorNodeStatus,
        SectorState,
    },
};

const MAP_CENTER_X: f32 = 360.0;
const MAP_CENTER_Y: f32 = 230.0;
const MAP_NODE_WIDTH: f32 = 170.0;
const MAP_NODE_HEIGHT: f32 = 52.0;

pub(crate) fn spawn_sector_map_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sector_state: Res<SectorState>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            SectorMapRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    top: Val::Px(24.0),
                    width: Val::Px(340.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Sector Map"),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                panel.spawn((
                    Text::new(sector_status_text(&sector_state)),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    SectorMapStatusText,
                ));
                panel.spawn((
                    Text::new(sector_detail_text(&sector_state)),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                    SectorMapDetailText,
                ));

                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.50, 0.30)),
                        LaunchEncounterButton,
                    ))
                    .with_child((
                        Text::new("Launch Selected Route"),
                        TextFont {
                            font: title_font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.32, 0.24, 0.18)),
                        BackToStationButton,
                    ))
                    .with_child((
                        Text::new("Back To Station"),
                        TextFont {
                            font: title_font,
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(400.0),
                    right: Val::Px(24.0),
                    top: Val::Px(24.0),
                    bottom: Val::Px(24.0),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.88)),
                SectorMapCanvas,
            ))
            .with_children(|map| {
                for node in &sector_state.nodes {
                    let reachable = sector_state.is_reachable(node.id);
                    let is_current = sector_state.current_node_id == node.id;
                    let is_selected = sector_state.selected_node_id == Some(node.id);
                    let base_color = node_button_color(
                        node.kind,
                        node.status,
                        reachable,
                        is_current,
                        is_selected,
                    );
                    map.spawn((
                        Button,
                        projected_node(node.position, 1.0, Vec2::ZERO),
                        BackgroundColor(base_color),
                        SectorNodeButton { node_id: node.id },
                    ))
                    .with_child((
                        Text::new(format!(
                            "{}\nR{} {}",
                            node.label,
                            node.risk_tier,
                            node.status.as_str()
                        )),
                        TextFont {
                            font: mono_font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        });
}

pub(crate) fn pan_and_zoom_sector_map(
    window: Single<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut pan_state: ResMut<SectorMapPanState>,
    mut view_state: ResMut<SectorMapViewState>,
) {
    let window = window.into_inner();

    for event in mouse_wheel.read() {
        let zoom_step = (1.0 + event.y * 0.08).clamp(0.75, 1.25);
        view_state.zoom = (view_state.zoom * zoom_step).clamp(0.65, 1.8);
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        if let Some(cursor) = window.cursor_position() {
            if let Some(previous_cursor) = pan_state.last_cursor {
                view_state.offset += cursor - previous_cursor;
            }
            pan_state.last_cursor = Some(cursor);
        } else {
            pan_state.last_cursor = None;
        }
    } else {
        pan_state.last_cursor = None;
    }
}

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

pub(crate) fn cleanup_sector_map_ui(
    mut commands: Commands,
    query: Query<Entity, With<SectorMapRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn sector_map_ui_missing(query: Query<Entity, With<SectorMapRoot>>) -> bool {
    query.is_empty()
}

pub(crate) fn sector_map_ui_present(query: Query<Entity, With<SectorMapRoot>>) -> bool {
    !query.is_empty()
}

pub(crate) fn sector_node_button_system(
    mut interaction_query: Query<
        (&Interaction, &SectorNodeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    status: Res<netcode::SessionStatus>,
    sector_state: Res<SectorState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        let Some(node) = sector_state.node(button.node_id).cloned() else {
            continue;
        };
        let reachable = sector_state.is_reachable(button.node_id);
        let is_current = sector_state.current_node_id == button.node_id;
        match *interaction {
            Interaction::Pressed => {
                if reachable && !matches!(node.kind, SectorNodeKind::HubStation) {
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::SelectSectorNode,
                        arg0: button.node_id as i16,
                        ..Default::default()
                    });
                }
                *background = BackgroundColor(PRESSED_BUTTON);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                let is_selected = sector_state.selected_node_id == Some(button.node_id);
                *background = BackgroundColor(node_button_color(
                    node.kind,
                    node.status,
                    reachable,
                    is_current,
                    is_selected,
                ));
            }
        }
    }
}

pub(crate) fn sector_navigation_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&LaunchEncounterButton>,
            Option<&BackToStationButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    sector_state: Res<SectorState>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background, launch, back) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if launch.is_some() {
                    *background = BackgroundColor(Color::srgb(0.12, 0.40, 0.24));
                    if let Some(node_id) = sector_state.selected_node_id
                        && sector_state.is_reachable(node_id)
                        && sector_state
                            .node(node_id)
                            .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
                            .unwrap_or(false)
                    {
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LaunchEncounter,
                            arg0: node_id as i16,
                            ..Default::default()
                        });
                    }
                } else if back.is_some() {
                    *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::ReturnToDock,
                        ..Default::default()
                    });
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if launch.is_some() {
                    Color::srgb(0.18, 0.50, 0.30)
                } else {
                    Color::srgb(0.32, 0.24, 0.18)
                });
            }
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

fn sector_status_text(sector_state: &SectorState) -> String {
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

fn sector_detail_text(sector_state: &SectorState) -> String {
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

fn node_button_color(
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

fn projected_node(position: [f32; 2], zoom: f32, offset: Vec2) -> Node {
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
