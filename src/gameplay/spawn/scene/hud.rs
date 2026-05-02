use bevy::prelude::*;

use crate::{
    gameplay::helpers::gameplay_status_line,
    ship::ShipDefinition,
    state::{
        GameplayBarFill,
        GameplayBarKind,
        GameplayBarLabel,
        GameplayControlsText,
        GameplayControlsPanel,
        GameplayInfoPanelRoot,
        GameplayOverviewBarsPanel,
        GameplayPanelBodyText,
        GameplayPanelTitleText,
        GameplayStationReadoutBarFill,
        GameplayStationReadoutBarTrack,
        GameplayStationReadoutLabel,
        GameplayStationReadoutLight,
        GameplayStationReadoutSlot,
        GameplayStationReadoutValue,
        GameplayStationPanel,
        GameplayStationPanelButton,
        GameplayStationPanelButtonLabel,
        GameplayStationTitleText,
        PlayingCleanup,
        StationPanelButtonAction,
    },
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
};

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
            spawn_info_panel(root, title_font.clone(), mono_font.clone(), ship);
            spawn_compact_status_panel(root, title_font.clone(), mono_font.clone());
            spawn_station_panel(root, title_font.clone(), mono_font.clone());
            spawn_controls_panel(root, title_font, mono_font);
        });
}

fn panel_shell(node: Node) -> impl Bundle {
    (node, BackgroundColor(Color::srgba(0.05, 0.08, 0.13, 0.93)))
}

fn spawn_info_panel(
    root: &mut ChildSpawnerCommands,
    title_font: Handle<Font>,
    mono_font: Handle<Font>,
    ship: &ShipDefinition,
) {
    root.spawn(panel_shell(Node {
        position_type: PositionType::Absolute,
        right: Val::Px(18.0),
        top: Val::Px(18.0),
        width: Val::Px(360.0),
        min_height: Val::Px(220.0),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
        ..default()
    }))
    .insert(GameplayInfoPanelRoot)
    .with_children(|panel| {
        panel.spawn((
            Text::new("Ship Overview"),
            TextFont {
                font: title_font,
                font_size: UI_TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            GameplayPanelTitleText,
        ));
        panel.spawn((
            Text::new(gameplay_status_line(ship)),
            TextFont {
                font: mono_font,
                font_size: UI_BODY_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.91, 0.98)),
            GameplayPanelBodyText,
        ));
    });
}

fn spawn_compact_status_panel(
    root: &mut ChildSpawnerCommands,
    title_font: Handle<Font>,
    mono_font: Handle<Font>,
) {
    root.spawn(panel_shell(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(18.0),
        top: Val::Px(18.0),
        width: Val::Px(360.0),
        padding: UiRect::all(Val::Px(14.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.0),
        border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
        ..default()
    }))
    .insert(GameplayOverviewBarsPanel)
    .with_children(|panel| {
        panel.spawn((
            Text::new("Ship Overview"),
            TextFont {
                font: title_font,
                font_size: UI_TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        spawn_bar(
            panel,
            mono_font.clone(),
            "Hull",
            GameplayBarKind::Hull,
            Color::srgb(0.80, 0.34, 0.34),
        );
        spawn_bar(
            panel,
            mono_font.clone(),
            "Power",
            GameplayBarKind::Power,
            Color::srgb(0.34, 0.74, 0.94),
        );
        spawn_bar(
            panel,
            mono_font.clone(),
            "Battery",
            GameplayBarKind::Battery,
            Color::srgb(0.86, 0.74, 0.30),
        );
        spawn_bar(
            panel,
            mono_font.clone(),
            "Oxygen",
            GameplayBarKind::Oxygen,
            Color::srgb(0.42, 0.86, 0.62),
        );
        spawn_bar(
            panel,
            mono_font.clone(),
            "Heat",
            GameplayBarKind::Heat,
            Color::srgb(0.96, 0.50, 0.24),
        );
        spawn_bar(
            panel,
            mono_font,
            "Electrical",
            GameplayBarKind::Electrical,
            Color::srgb(0.74, 0.56, 0.98),
        );
    });
}

fn spawn_bar(
    panel: &mut ChildSpawnerCommands,
    mono_font: Handle<Font>,
    label: &str,
    kind: GameplayBarKind,
    fill_color: Color,
) {
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                Text::new(format!("{label}: --")),
                TextFont {
                    font: mono_font.clone(),
                    font_size: UI_BODY_FONT_SIZE - 1.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.88, 0.95)),
                GameplayBarLabel { kind },
            ));
            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(10.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.20, 0.24, 0.30, 0.95)),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(999.0)),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    GameplayBarFill { kind },
                ));
            });
        });
}

fn spawn_station_panel(
    root: &mut ChildSpawnerCommands,
    title_font: Handle<Font>,
    mono_font: Handle<Font>,
) {
    let actions = [
        StationPanelButtonAction::HelmThrottle { delta: -0.2 },
        StationPanelButtonAction::HelmThrottle { delta: 0.2 },
        StationPanelButtonAction::HelmTurn { value: -1.0 },
        StationPanelButtonAction::HelmTurn { value: 0.0 },
        StationPanelButtonAction::HelmTurn { value: 1.0 },
        StationPanelButtonAction::TurretAdjustAim { delta: 0.25 },
        StationPanelButtonAction::TurretAdjustAim { delta: -0.25 },
        StationPanelButtonAction::TurretFireToggle,
        StationPanelButtonAction::ReactorAdjustRate { delta: -0.1 },
        StationPanelButtonAction::ReactorAdjustRate { delta: 0.1 },
        StationPanelButtonAction::ReactorAdjustTurbine { delta: -0.1 },
        StationPanelButtonAction::ReactorAdjustTurbine { delta: 0.1 },
        StationPanelButtonAction::LogisticsToggleStorageIntake,
        StationPanelButtonAction::LogisticsToggleAirlock,
        StationPanelButtonAction::LogisticsToggleManipulator,
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction: -1 },
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction: 1 },
        StationPanelButtonAction::LogisticsCycleResource,
        StationPanelButtonAction::LogisticsToggleProcessor,
        StationPanelButtonAction::ComputerToggleEnabled,
        StationPanelButtonAction::ComputerCycleTemplate,
    ];

    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(20.0),
            right: Val::Percent(20.0),
            top: Val::Px(160.0),
            min_height: Val::Px(260.0),
            padding: UiRect::all(Val::Px(18.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.06, 0.11, 0.96)),
        GameplayStationPanel,
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("Station Console"),
            TextFont {
                font: title_font,
                font_size: UI_TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            GameplayStationTitleText,
        ));
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|readouts| {
                for index in 0..6 {
                    readouts
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                            GameplayStationReadoutSlot { index },
                        ))
                        .with_children(|row| {
                            row.spawn(Node {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|header| {
                                header.spawn((
                                    Text::new("--"),
                                    TextFont {
                                        font: mono_font.clone(),
                                        font_size: UI_BODY_FONT_SIZE,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.88, 0.92, 0.98)),
                                    GameplayStationReadoutLabel,
                                ));
                                header.spawn(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                })
                                .with_children(|status| {
                                    status.spawn((
                                        Node {
                                            width: Val::Px(12.0),
                                            height: Val::Px(12.0),
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            display: Display::None,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.22, 0.26, 0.30)),
                                        GameplayStationReadoutLight,
                                    ));
                                    status.spawn((
                                        Text::new("--"),
                                        TextFont {
                                            font: mono_font.clone(),
                                            font_size: UI_BODY_FONT_SIZE - 1.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.76, 0.84, 0.92)),
                                        GameplayStationReadoutValue,
                                    ));
                                });
                            });
                            row.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(10.0),
                                    border_radius: BorderRadius::all(Val::Px(999.0)),
                                    display: Display::None,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.20, 0.24, 0.30, 0.95)),
                                GameplayStationReadoutBarTrack,
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        border_radius: BorderRadius::all(Val::Px(999.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.40, 0.72, 0.94)),
                                    GameplayStationReadoutBarFill,
                                ));
                            });
                        });
                }
            });
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|buttons| {
                for action in actions {
                    buttons
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(34.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.24, 0.38, 0.58)),
                            GameplayStationPanelButton { action },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(station_button_default_label(action)),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: UI_BODY_FONT_SIZE - 1.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                GameplayStationPanelButtonLabel { action },
                            ));
                        });
                }
            });
    });
}

fn spawn_controls_panel(
    root: &mut ChildSpawnerCommands,
    title_font: Handle<Font>,
    mono_font: Handle<Font>,
) {
    root.spawn(panel_shell(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(18.0),
        bottom: Val::Px(22.0),
        width: Val::Px(360.0),
        padding: UiRect::all(Val::Px(14.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(8.0),
        border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
        ..default()
    }))
    .insert(GameplayControlsPanel)
    .with_children(|panel| {
        panel.spawn((
            Text::new("Controls"),
            TextFont {
                font: title_font,
                font_size: UI_TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        panel.spawn((
            Text::new("Controls pending"),
            TextFont {
                font: mono_font.clone(),
                font_size: UI_HELP_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.86, 0.92)),
            GameplayControlsText,
        ));
    });
}

fn station_button_default_label(action: StationPanelButtonAction) -> &'static str {
    match action {
        StationPanelButtonAction::HelmThrottle { delta } if delta < 0.0 => "Throttle Down",
        StationPanelButtonAction::HelmThrottle { .. } => "Throttle Up",
        StationPanelButtonAction::HelmTurn { value } if value < 0.0 => "Turn Port",
        StationPanelButtonAction::HelmTurn { value } if value > 0.0 => "Turn Starboard",
        StationPanelButtonAction::HelmTurn { .. } => "Steady",
        StationPanelButtonAction::TurretAdjustAim { delta } if delta < 0.0 => "Aim Right",
        StationPanelButtonAction::TurretAdjustAim { .. } => "Aim Left",
        StationPanelButtonAction::TurretFireToggle => "Fire Intent",
        StationPanelButtonAction::ReactorAdjustRate { delta } if delta < 0.0 => "Rate -",
        StationPanelButtonAction::ReactorAdjustRate { .. } => "Rate +",
        StationPanelButtonAction::ReactorAdjustTurbine { delta } if delta < 0.0 => "Turbine -",
        StationPanelButtonAction::ReactorAdjustTurbine { .. } => "Turbine +",
        StationPanelButtonAction::LogisticsToggleStorageIntake => "Toggle Intake",
        StationPanelButtonAction::LogisticsToggleAirlock => "Cycle Airlock",
        StationPanelButtonAction::LogisticsToggleManipulator => "Manipulator",
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction }
            if direction < 0 =>
        {
            "Prev Target"
        }
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => "Next Target",
        StationPanelButtonAction::LogisticsCycleResource => "Cycle Resource",
        StationPanelButtonAction::LogisticsToggleProcessor => "Processor",
        StationPanelButtonAction::ComputerToggleEnabled => "Enable/Disable",
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template",
    }
}
