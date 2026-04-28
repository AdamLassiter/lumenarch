use bevy::prelude::*;

use crate::client::gameplay::helpers::gameplay_status_line;
use crate::client::state::{
    GameplayAlertsText,
    GameplayControlsText,
    GameplayInspectionText,
    GameplayStatusText,
    PlayingCleanup,
    ReturnButton,
};
use crate::ship::ShipDefinition;

pub(super) fn spawn_runtime_hud(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
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
            PlayingCleanup,
        ))
        .with_children(|root| {
            spawn_status_panel(root, &asset_server, title_font.clone(), mono_font.clone(), ship);
            spawn_inspection_panel(root, mono_font.clone());
            spawn_alerts_panel(root, mono_font.clone());
            spawn_controls_panel(root, mono_font);
        });
}

fn spawn_status_panel(
    root: &mut ChildBuilder,
    asset_server: &AssetServer,
    title_font: Handle<Font>,
    mono_font: Handle<Font>,
    ship: &ShipDefinition,
) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            padding: UiRect::all(Val::Px(14.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
        BorderRadius::all(Val::Px(10.0)),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Runtime Slice"),
            TextFont {
                font: title_font,
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        panel.spawn((
            Text::new(gameplay_status_line(ship)),
            TextFont {
                font: mono_font.clone(),
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.92, 0.94, 0.98)),
            GameplayStatusText,
        ));

        panel
            .spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(44.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.52, 0.27, 0.18)),
                BorderRadius::all(Val::Px(8.0)),
                ReturnButton,
            ))
            .with_child((
                Text::new("Return To Editor"),
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
    });
}

fn spawn_inspection_panel(root: &mut ChildBuilder, mono_font: Handle<Font>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            padding: UiRect::all(Val::Px(14.0)),
            max_width: Val::Px(340.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
        BorderRadius::all(Val::Px(10.0)),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Current Station"),
            TextFont {
                font: mono_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        panel.spawn((
            Text::new("Station data pending"),
            TextFont {
                font: mono_font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            GameplayInspectionText,
        ));
    });
}

fn spawn_alerts_panel(root: &mut ChildBuilder, mono_font: Handle<Font>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(220.0),
            padding: UiRect::all(Val::Px(14.0)),
            max_width: Val::Px(340.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
        BorderRadius::all(Val::Px(10.0)),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Alerts"),
            TextFont {
                font: mono_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        panel.spawn((
            Text::new("No alerts"),
            TextFont {
                font: mono_font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            GameplayAlertsText,
        ));
    });
}

fn spawn_controls_panel(root: &mut ChildBuilder, mono_font: Handle<Font>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            padding: UiRect::all(Val::Px(14.0)),
            max_width: Val::Px(320.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
        BorderRadius::all(Val::Px(10.0)),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new(
                "Controls pending",
            ),
            TextFont {
                font: mono_font,
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.86, 0.92)),
            GameplayControlsText,
        ));
    });
}
