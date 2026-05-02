use bevy::prelude::*;

use crate::{
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    state::{
        BackToStationButton, LaunchEncounterButton, SectorMapCanvas, SectorMapDetailText,
        SectorMapRoot, SectorMapStatusText, SectorNodeButton, SectorState,
    },
};

use super::layout::{node_button_color, projected_node, sector_detail_text, sector_status_text};

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
                    left: Val::Px(400.0),
                    right: Val::Px(24.0),
                    top: Val::Px(24.0),
                    bottom: Val::Px(24.0),
                    overflow: Overflow::clip(),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
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
                            font_size: UI_BODY_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    top: Val::Px(24.0),
                    width: Val::Px(340.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Sector Map"),
                    TextFont {
                        font: title_font.clone(),
                        font_size: UI_TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                panel.spawn((
                    Text::new(sector_status_text(&sector_state)),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    SectorMapStatusText,
                ));
                panel.spawn((
                    Text::new(sector_detail_text(&sector_state)),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
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
                            border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.50, 0.30)),
                        LaunchEncounterButton,
                    ))
                    .with_child((
                        Text::new("Launch Selected Route"),
                            TextFont {
                                font: title_font.clone(),
                                font_size: UI_TITLE_FONT_SIZE - 2.0,
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
                            border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.32, 0.24, 0.18)),
                        BackToStationButton,
                    ))
                    .with_child((
                        Text::new("Back To Station"),
                            TextFont {
                                font: title_font,
                                font_size: UI_TITLE_FONT_SIZE - 2.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                    ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    bottom: Val::Px(24.0),
                    width: Val::Px(340.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Sector Controls\nLeft click: select reachable route\nScroll: zoom map\nMiddle drag: pan map\nEnter host mission from left action panel\nTab: return to station",
                    ),
                    TextFont {
                        font: mono_font,
                        font_size: UI_HELP_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
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
