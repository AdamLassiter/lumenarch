use bevy::{ecs::relationship::Relationship, prelude::*};

use super::super::helpers::{editor_mission_report_text, editor_status_line};
use crate::{
    TOOLBOX_COMPONENTS,
    TOOLBOX_WIDTH,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    state::{
        DemoProgression,
        EditingCleanup,
        EditorMissionReportButton,
        EditorMissionReportButtonText,
        EditorMissionReportText,
        EditorMode,
        EditorRoot,
        EditorSessionState,
        EditorShip,
        EditorStatusText,
        EditorToolState,
        EditorToolboxScrollContent,
        EditorToolboxScrollViewport,
        EditorUiState,
        EnemyEditorState,
        EnemyNewButton,
        EnemyNextButton,
        EnemyPrevButton,
        EnemyShipLibraryState,
        GameplayStationPanel,
        GameplayStationPanelButton,
        GameplayStationPanelButtonLabel,
        GameplayStationReadoutBarFill,
        GameplayStationReadoutBarTrack,
        GameplayStationReadoutLabel,
        GameplayStationReadoutLight,
        GameplayStationReadoutSlot,
        GameplayStationReadoutValue,
        GameplayStationTitleText,
        LastMissionReport,
        LeaveEditorButton,
        ProgramEditorAction,
        ProgramEditorActionButton,
        ProgramEditorDiagnosticsText,
        ProgramEditorDraftText,
        ProgramEditorStatusText,
        ProgramEditorTextBox,
        StationPanelButtonAction,
        ToolboxButton,
        ToolboxButtonText,
    },
};

pub(crate) fn editor_ui_missing(query: Query<Entity, With<EditorRoot>>) -> bool {
    query.is_empty()
}

pub(crate) fn editor_ui_present(query: Query<Entity, With<EditorRoot>>) -> bool {
    !query.is_empty()
}

pub(crate) fn spawn_editor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_session: Res<EditorSessionState>,
    enemy_editor_state: Res<EnemyEditorState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ui_state: Res<EditorUiState>,
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
            EditorRoot,
            EditingCleanup,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(TOOLBOX_WIDTH),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|toolbox| {
                toolbox.spawn((
                    Text::new(match editor_session.mode {
                        EditorMode::Player => "Toolbox".to_string(),
                        EditorMode::Enemy => "Enemy Ship Debug".to_string(),
                    }),
                    TextFont {
                        font: title_font.clone(),
                        font_size: UI_TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                toolbox
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(430.0),
                            overflow: Overflow::clip_y(),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        EditorToolboxScrollViewport,
                    ))
                    .with_children(|viewport| {
                        viewport
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    right: Val::Px(0.0),
                                    top: Val::Px(-editor_ui_state.toolbox_scroll),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    ..default()
                                },
                                EditorToolboxScrollContent,
                            ))
                            .with_children(|content| {
                                for kind in TOOLBOX_COMPONENTS {
                                    content
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(38.0),
                                                justify_content: JustifyContent::SpaceBetween,
                                                align_items: AlignItems::Center,
                                                padding: UiRect::horizontal(Val::Px(10.0)),
                                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                                ..default()
                                            },
                                            BackgroundColor(crate::NORMAL_BUTTON),
                                            ToolboxButton { kind },
                                        ))
                                        .with_children(|button| {
                                            button.spawn((
                                                Text::new(toolbox_label(
                                                        kind,
                                                        editor_session.mode,
                                                        &progression,
                                                    ).to_string()
                                                ),
                                                TextFont {
                                                    font: mono_font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                                ToolboxButtonText { kind },
                                            ));
                                        });
                                }

                                if editor_session.mode == EditorMode::Enemy {
                                    if let Some(entry) = enemy_library_state
                                        .library
                                        .selected_or_first(enemy_library_state.selected_index)
                                    {
                                        content.spawn((
                                            Text::new(format!(
                                                "Editing: {} [{}]\nThreat: {}  Behavior: {}",
                                                entry.display_name, entry.id, entry.threat_tier, entry.behavior_tag
                                            )),
                                            TextFont {
                                                font: mono_font.clone(),
                                                font_size: UI_BODY_FONT_SIZE,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.92, 0.94, 0.98)),
                                        ));
                                    }

                                    for (label, marker) in [
                                        ("Previous Enemy", 0usize),
                                        ("Next Enemy", 1usize),
                                        ("New Enemy", 2usize),
                                    ] {
                                        let mut button = content.spawn((
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(34.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.24, 0.32, 0.48)),
                                        ));
                                        let button = match marker {
                                            0 => button.insert(EnemyPrevButton),
                                            1 => button.insert(EnemyNextButton),
                                            _ => button.insert(EnemyNewButton),
                                        };
                                        button.with_child((
                                            Text::new(label),
                                            TextFont {
                                                font: mono_font.clone(),
                                                font_size: 15.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    }
                                }
                            });
                    });

                toolbox
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.46, 0.34, 0.22)),
                        LeaveEditorButton,
                    ))
                    .with_child((
                        Text::new(match editor_session.mode {
                            EditorMode::Player => "Return To Station",
                            EditorMode::Enemy => "Return To Menu",
                        }),
                        TextFont {
                            font: mono_font.clone(),
                            font_size: UI_TITLE_FONT_SIZE - 2.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|hud| {
                hud.spawn((
                        Text::new(editor_status_line(
                            editor_session.mode,
                            &enemy_entry_label(
                                &editor_session,
                                &enemy_editor_state,
                                &enemy_library_state,
                            ),
                            &editor_ship.ship.name,
                        &TOOLBOX_COMPONENTS[0],
                        crate::ship::ModuleVariant::default_for_kind(TOOLBOX_COMPONENTS[0]),
                        0,
                        0,
                        editor_ship.ship.modules.len(),
                        progression.scrap,
                        &progression,
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    EditorStatusText,
                ));
            });

            if editor_session.mode == EditorMode::Player {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(16.0),
                        bottom: Val::Px(16.0),
                        width: Val::Px(360.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        flex_direction: FlexDirection::ColumnReverse,
                        row_gap: Val::Px(10.0),
                        border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                ))
                .with_children(|panel| {
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(38.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.24, 0.32, 0.48)),
                            EditorMissionReportButton,
                        ))
                        .with_child((
                            Text::new(if editor_ui_state.mission_report_expanded {
                                "Hide Last Mission"
                            } else {
                                "Show Last Mission"
                            }),
                            TextFont {
                                font: mono_font.clone(),
                                font_size: UI_BODY_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            EditorMissionReportButtonText,
                        ));

                    panel.spawn((
                        Node {
                            display: if editor_ui_state.mission_report_expanded {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            ..default()
                        },
                        Text::new(editor_mission_report_text(&last_mission_report)),
                        TextFont {
                            font: mono_font.clone(),
                            font_size: UI_BODY_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgb(0.82, 0.86, 0.92)),
                        EditorMissionReportText,
                    ));
                });
            }

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
                    Text::new("Module Console"),
                    TextFont {
                        font: title_font.clone(),
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
                                        header
                                            .spawn(Node {
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
                        for action in station_actions() {
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
                panel.spawn((
                    Text::new("Program Draft"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.76, 0.80, 0.86)),
                ));
                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(160.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.16, 0.24, 0.94)),
                        ProgramEditorTextBox,
                    ))
                    .with_children(|textbox| {
                        textbox.spawn((
                            Text::new(""),
                            TextFont {
                                font: mono_font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            ProgramEditorDraftText,
                        ));
                    });
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        for (label, action) in [
                            ("ARCH", ProgramEditorAction::SwitchArch),
                            ("LUMEN", ProgramEditorAction::SwitchLumen),
                            ("Check", ProgramEditorAction::Check),
                            ("Apply", ProgramEditorAction::Apply),
                            ("Revert", ProgramEditorAction::Revert),
                        ] {
                            buttons
                                .spawn((
                                    Button,
                                    Node {
                                        height: Val::Px(32.0),
                                        padding: UiRect::horizontal(Val::Px(10.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        ..default()
                                    },
                                    BackgroundColor(crate::NORMAL_BUTTON),
                                    ProgramEditorActionButton { action },
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font: mono_font.clone(),
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        }
                    });
                panel.spawn((
                    Text::new("Draft idle"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.84, 0.62)),
                    ProgramEditorStatusText,
                ));
                panel.spawn((
                    Text::new("Diagnostics"),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.76, 0.80, 0.86)),
                    ProgramEditorDiagnosticsText,
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    max_width: Val::Px(320.0),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        match editor_session.mode {
                            EditorMode::Player => "Refit Controls\nLeft click: place or replace\nRight click: erase\nE: inspect hovered component\nQ: close component console\nR: rotate clockwise\nT: repair one damaged selected part\nC / V: channel - / +\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save ship  |  F9 reload ship\nTab: return to station",
                            EditorMode::Enemy => "Enemy Editor Controls\nUnlimited component supply\nLeft click: place or replace\nRight click: erase\nE: inspect hovered component\nQ: close component console\nR: rotate clockwise\nC / V: channel - / +\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save library  |  F9 reload library\n[ / ] cycle entry  |  N new enemy\nTab: return to menu",
                        },
                    ),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_HELP_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
}

pub(crate) fn update_editor_status_text(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    enemy_editor_state: Res<EnemyEditorState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
    tool_state: Res<EditorToolState>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ui_state: Res<EditorUiState>,
    mut ui_queries: ParamSet<(
        Query<'_, '_, &'static mut Text, With<EditorStatusText>>,
        Query<
            '_,
            '_,
            (&'static mut Node, &'static mut Text),
            (With<EditorMissionReportText>, Without<EditorStatusText>),
        >,
        Query<
            '_,
            '_,
            &'static mut Text,
            (
                With<EditorMissionReportButtonText>,
                Without<EditorStatusText>,
                Without<EditorMissionReportText>,
            ),
        >,
    )>,
) {
    if !editor_ship.is_changed()
        && !tool_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
        && !editor_ui_state.is_changed()
        && !enemy_editor_state.is_changed()
    {
        return;
    }

    for mut text in &mut ui_queries.p0() {
        **text = editor_status_line(
            editor_session.mode,
            &enemy_entry_label(&editor_session, &enemy_editor_state, &enemy_library_state),
            &editor_ship.ship.name,
            &tool_state.selected_kind,
            tool_state.selected_variant,
            tool_state.selected_rotation,
            tool_state.selected_channel,
            editor_ship.ship.modules.len(),
            progression.scrap,
            &progression,
        );
    }

    for (mut node, mut text) in &mut ui_queries.p1() {
        node.display = if editor_ui_state.mission_report_expanded {
            Display::Flex
        } else {
            Display::None
        };
        **text = editor_mission_report_text(&last_mission_report);
    }

    for mut text in &mut ui_queries.p2() {
        **text = if editor_ui_state.mission_report_expanded {
            "Hide Last Mission".to_string()
        } else {
            "Show Last Mission".to_string()
        };
    }
}

pub(crate) fn update_editor_module_overlay(
    editor_ship: Res<EditorShip>,
    arch_editor_state: Res<crate::state::ArchEditorState>,
    program_editor_state: Res<crate::state::ProgramTextEditorState>,
    mut panel_query: Query<
        &mut Node,
        (
            With<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut title_query: Query<
        &mut Text,
        (
            With<GameplayStationTitleText>,
            Without<ProgramEditorStatusText>,
            Without<ProgramEditorDiagnosticsText>,
            Without<ProgramEditorDraftText>,
            Without<GameplayStationPanelButtonLabel>,
            Without<GameplayStationReadoutLabel>,
            Without<GameplayStationReadoutValue>,
        ),
    >,
    mut button_query: Query<
        (
            &GameplayStationPanelButton,
            &mut Node,
            &Children,
            &mut BackgroundColor,
        ),
        (
            Without<ProgramEditorActionButton>,
            Without<GameplayStationPanel>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
        ),
    >,
    mut slot_query: Query<
        (Entity, &GameplayStationReadoutSlot, &mut Node, &Children),
        (
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutBarFill>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut button_label_query: Query<
        (&GameplayStationPanelButtonLabel, &mut Text),
        (
            Without<GameplayStationTitleText>,
            Without<ProgramEditorDraftText>,
            Without<ProgramEditorStatusText>,
            Without<ProgramEditorDiagnosticsText>,
            Without<GameplayStationReadoutLabel>,
            Without<GameplayStationReadoutValue>,
        ),
    >,
    mut text_query: ParamSet<(
        Query<'_, '_, &'static mut Text, With<ProgramEditorDraftText>>,
        Query<'_, '_, &'static mut Text, With<ProgramEditorStatusText>>,
        Query<'_, '_, &'static mut Text, With<ProgramEditorDiagnosticsText>>,
        Query<
            '_,
            '_,
            (&'static ChildOf, &'static mut Text),
            (
                With<GameplayStationReadoutLabel>,
                Without<GameplayStationReadoutValue>,
                Without<GameplayStationTitleText>,
                Without<GameplayStationPanelButtonLabel>,
                Without<ProgramEditorDraftText>,
                Without<ProgramEditorStatusText>,
                Without<ProgramEditorDiagnosticsText>,
            ),
        >,
        Query<
            '_,
            '_,
            (&'static ChildOf, &'static mut Text),
            (
                With<GameplayStationReadoutValue>,
                Without<GameplayStationReadoutLabel>,
                Without<GameplayStationTitleText>,
                Without<GameplayStationPanelButtonLabel>,
                Without<ProgramEditorDraftText>,
                Without<ProgramEditorStatusText>,
                Without<ProgramEditorDiagnosticsText>,
            ),
        >,
    )>,
    mut bar_track_query: Query<
        (&ChildOf, &mut Node),
        (
            With<GameplayStationReadoutBarTrack>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarFill>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut bar_fill_query: Query<
        (&ChildOf, &mut Node, &mut BackgroundColor),
        (
            With<GameplayStationReadoutBarFill>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutLight>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut light_query: Query<
        (&ChildOf, &mut Node, &mut BackgroundColor),
        (
            With<GameplayStationReadoutLight>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<GameplayStationReadoutBarTrack>,
            Without<GameplayStationReadoutBarFill>,
            Without<ProgramEditorTextBox>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut program_box_query: Query<
        &mut Node,
        (
            With<ProgramEditorTextBox>,
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorActionButton>,
        ),
    >,
    mut program_action_query: Query<
        (&ProgramEditorActionButton, &mut Node),
        (
            Without<GameplayStationPanel>,
            Without<GameplayStationPanelButton>,
            Without<GameplayStationReadoutSlot>,
            Without<ProgramEditorTextBox>,
        ),
    >,
) {
    for mut panel_node in &mut panel_query {
        panel_node.display = if arch_editor_state.panel_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !arch_editor_state.panel_open {
        return;
    }

    let Some(module_id) = arch_editor_state.selected_module_id else {
        return;
    };
    let Some(module) = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.id == module_id)
    else {
        return;
    };

    let mode = editor_control_mode(module.kind);
    let flags = editor_station_flags(module.kind);
    let title = if module.kind.supports_channel() {
        format!(
            "{} Console  [CH{}]",
            module.kind.as_str(),
            module.effective_channel()
        )
    } else {
        format!("{} Console", module.kind.as_str())
    };
    for mut text in &mut title_query {
        **text = title.clone();
    }

    let readouts = editor_station_readouts(module);
    let show_program_editor = module.kind == crate::ship::ModuleKind::Computer;

    for mut node in &mut program_box_query {
        node.display = if show_program_editor {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (action, mut node) in &mut program_action_query {
        node.display = if show_program_editor
            || !matches!(
                action.action,
                ProgramEditorAction::SwitchArch
                    | ProgramEditorAction::SwitchLumen
                    | ProgramEditorAction::Check
                    | ProgramEditorAction::Apply
                    | ProgramEditorAction::Revert
            ) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut text in &mut text_query.p0() {
        **text = if show_program_editor {
            format_program_textbox(
                &program_editor_state,
                module_id,
                arch_editor_state.selected_language,
                module,
            )
        } else {
            String::new()
        };
    }
    for mut text in &mut text_query.p1() {
        **text = if show_program_editor {
            if program_editor_state.status_line.is_empty() {
                format!(
                    "{} source",
                    match arch_editor_state.selected_language {
                        crate::state::ProgrammingLanguageMode::Arch => "ARCH",
                        crate::state::ProgrammingLanguageMode::Lumen => "LUMEN",
                    }
                )
            } else {
                program_editor_state.status_line.clone()
            }
        } else {
            String::new()
        };
    }
    for mut text in &mut text_query.p2() {
        **text = if show_program_editor && !program_editor_state.diagnostics.is_empty() {
            format!(
                "Diagnostics\n{}",
                program_editor_state.diagnostics.join("\n")
            )
        } else if show_program_editor {
            "Diagnostics\nNo parse diagnostics".to_string()
        } else {
            String::new()
        };
    }

    for (button, mut node, children, mut color) in &mut button_query {
        let visible = editor_station_action_visible(button.action, mode, module.kind, flags);
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *color = BackgroundColor(if visible {
            Color::srgb(0.24, 0.38, 0.58)
        } else {
            Color::srgba(0.24, 0.38, 0.58, 0.18)
        });
        for child in children.iter() {
            if let Ok((label, mut text)) = button_label_query.get_mut(child) {
                **text = editor_station_button_label(
                    label.action,
                    mode,
                    flags,
                    arch_editor_state.selected_language,
                );
            }
        }
    }

    let mut readout_rows = Vec::new();
    for (row_entity, slot, mut row_node, children) in &mut slot_query {
        let readout = readouts.get(slot.index as usize).cloned();
        row_node.display = if readout.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(readout) = readout {
            readout_rows.push((row_entity, children.to_vec(), readout));
        }
    }

    for (row_entity, children, readout) in readout_rows {
        for child in children {
            if let Ok((parent, mut text)) = text_query.p3().get_mut(child)
                && parent.get() == row_entity
            {
                **text = readout.label.clone();
            }
            if let Ok((parent, mut text)) = text_query.p4().get_mut(child)
                && parent.get() == row_entity
            {
                **text = readout.value.clone();
            }
            if let Ok((parent, mut node)) = bar_track_query.get_mut(child)
                && parent.get() == row_entity
            {
                node.display = if matches!(readout.visual, EditorReadoutVisual::Bar { .. }) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Ok((parent, mut node, mut color)) = bar_fill_query.get_mut(child)
                && parent.get() == row_entity
            {
                if let EditorReadoutVisual::Bar {
                    percent,
                    color: fill,
                } = readout.visual
                {
                    node.width = Val::Percent(percent);
                    *color = BackgroundColor(fill);
                    node.display = Display::Flex;
                }
            }
            if let Ok((parent, mut node, mut color)) = light_query.get_mut(child)
                && parent.get() == row_entity
            {
                if let EditorReadoutVisual::Light { color: light } = readout.visual {
                    node.display = Display::Flex;
                    *color = BackgroundColor(light);
                } else {
                    node.display = Display::None;
                }
            }
        }
    }
}

fn toolbox_label(
    kind: crate::ship::ModuleKind,
    mode: EditorMode,
    progression: &DemoProgression,
) -> String {
    let variant = crate::ship::ModuleVariant::default_for_kind(kind);
    match mode {
        EditorMode::Player => format!(
            "{}  [ready {} / damaged {}]",
            kind.as_str(),
            progression.ready_count(kind, variant),
            progression.damaged_count(kind, variant),
        ),
        EditorMode::Enemy => format!("{}  [∞]", kind.as_str(),),
    }
}

#[derive(Clone)]
struct EditorReadout {
    label: String,
    value: String,
    visual: EditorReadoutVisual,
}

#[derive(Clone, Copy)]
enum EditorReadoutVisual {
    Bar { percent: f32, color: Color },
    Light { color: Color },
}

fn editor_control_mode(
    kind: crate::ship::ModuleKind,
) -> crate::gameplay::components::ShipControlMode {
    match kind {
        crate::ship::ModuleKind::Cockpit => crate::gameplay::components::ShipControlMode::Cockpit,
        crate::ship::ModuleKind::Turret => crate::gameplay::components::ShipControlMode::Turret,
        crate::ship::ModuleKind::Reactor => crate::gameplay::components::ShipControlMode::Reactor,
        crate::ship::ModuleKind::Cargo
        | crate::ship::ModuleKind::Airlock
        | crate::ship::ModuleKind::Processor => {
            crate::gameplay::components::ShipControlMode::Logistics
        }
        crate::ship::ModuleKind::Computer => crate::gameplay::components::ShipControlMode::Computer,
        _ => crate::gameplay::components::ShipControlMode::Interior,
    }
}

#[derive(Clone, Copy)]
struct EditorStationFlags {
    storage: bool,
    manipulator: bool,
    processor: bool,
    airlock: bool,
}

fn editor_station_flags(kind: crate::ship::ModuleKind) -> EditorStationFlags {
    EditorStationFlags {
        storage: matches!(
            kind,
            crate::ship::ModuleKind::Cargo | crate::ship::ModuleKind::Airlock
        ),
        manipulator: kind == crate::ship::ModuleKind::Airlock,
        processor: kind == crate::ship::ModuleKind::Processor,
        airlock: kind == crate::ship::ModuleKind::Airlock,
    }
}

fn editor_station_action_visible(
    action: StationPanelButtonAction,
    mode: crate::gameplay::components::ShipControlMode,
    active_kind: crate::ship::ModuleKind,
    flags: EditorStationFlags,
) -> bool {
    match mode {
        crate::gameplay::components::ShipControlMode::Cockpit => matches!(
            action,
            StationPanelButtonAction::HelmThrottle { .. }
                | StationPanelButtonAction::HelmTurn { .. }
        ),
        crate::gameplay::components::ShipControlMode::Turret => matches!(
            action,
            StationPanelButtonAction::TurretAdjustAim { .. }
                | StationPanelButtonAction::TurretFireToggle
        ),
        crate::gameplay::components::ShipControlMode::Reactor => matches!(
            action,
            StationPanelButtonAction::ReactorAdjustRate { .. }
                | StationPanelButtonAction::ReactorAdjustTurbine { .. }
        ),
        crate::gameplay::components::ShipControlMode::Logistics => match action {
            StationPanelButtonAction::LogisticsToggleStorageIntake => flags.storage,
            StationPanelButtonAction::LogisticsToggleAirlock => flags.airlock,
            StationPanelButtonAction::LogisticsToggleManipulator
            | StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. }
            | StationPanelButtonAction::LogisticsCycleResource => flags.manipulator,
            StationPanelButtonAction::LogisticsToggleProcessor => {
                flags.processor || active_kind == crate::ship::ModuleKind::Airlock
            }
            _ => false,
        },
        crate::gameplay::components::ShipControlMode::Computer => matches!(
            action,
            StationPanelButtonAction::ComputerToggleEnabled
                | StationPanelButtonAction::ComputerCycleTemplate
        ),
        crate::gameplay::components::ShipControlMode::Interior => false,
    }
}

fn editor_station_button_label(
    action: StationPanelButtonAction,
    mode: crate::gameplay::components::ShipControlMode,
    flags: EditorStationFlags,
    language: crate::state::ProgrammingLanguageMode,
) -> String {
    match action {
        StationPanelButtonAction::ComputerCycleTemplate => match language {
            crate::state::ProgrammingLanguageMode::Arch => "Cycle ARCH Template".to_string(),
            crate::state::ProgrammingLanguageMode::Lumen => "Cycle LUMEN Template".to_string(),
        },
        StationPanelButtonAction::LogisticsToggleProcessor
            if mode == crate::gameplay::components::ShipControlMode::Logistics
                && !flags.processor =>
        {
            "Cycle Recipe".to_string()
        }
        _ => station_button_default_label(action).to_string(),
    }
}

fn editor_station_readouts(module: &crate::ship::ShipModule) -> Vec<EditorReadout> {
    match module.kind {
        crate::ship::ModuleKind::Reactor => vec![
            editor_bar(
                module,
                "RRF",
                "Reaction",
                module.defaults.reaction_rate_milli as f32 / 10.0,
                Color::srgb(0.94, 0.42, 0.24),
            ),
            editor_bar(
                module,
                "RRT",
                "Turbine",
                module.defaults.turbine_load_milli as f32 / 10.0,
                Color::srgb(0.34, 0.74, 0.94),
            ),
            editor_light(
                module,
                "RRS",
                "Default Stability",
                "Nominal",
                Color::srgb(0.34, 0.78, 0.46),
            ),
            editor_light(
                module,
                "RRP",
                "Startup Output",
                "Derived in encounter",
                Color::srgb(0.86, 0.74, 0.30),
            ),
        ],
        crate::ship::ModuleKind::Turret => vec![
            editor_light(
                module,
                "WTF",
                "Fire Gate",
                if module.defaults.turret_fire_intent {
                    "Open"
                } else {
                    "Hold"
                },
                if module.defaults.turret_fire_intent {
                    Color::srgb(0.34, 0.78, 0.46)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            editor_light(
                module,
                "WTR",
                "Default Arc",
                "Forward",
                Color::srgb(0.64, 0.60, 0.96),
            ),
            editor_light(
                module,
                "WTC",
                "Cooldown",
                "Variant derived",
                Color::srgb(0.96, 0.56, 0.24),
            ),
        ],
        crate::ship::ModuleKind::Cargo => vec![
            editor_light(
                module,
                "LSI",
                "Allow Intake",
                bool_word(module.defaults.storage_allow_intake),
                bool_color(module.defaults.storage_allow_intake),
            ),
            editor_light(
                module,
                "LSC",
                "Capacity",
                &crate::ship::ModuleSpec::for_module(module.kind, module.variant)
                    .storage_capacity
                    .to_string(),
                Color::srgb(0.36, 0.72, 0.96),
            ),
        ],
        crate::ship::ModuleKind::Airlock => vec![
            editor_light(
                module,
                "LAC",
                "Airlock",
                if module.defaults.airlock_open {
                    "Open"
                } else {
                    "Closed"
                },
                bool_color(!module.defaults.airlock_open),
            ),
            editor_light(
                module,
                "LSI",
                "Allow Intake",
                bool_word(module.defaults.storage_allow_intake),
                bool_color(module.defaults.storage_allow_intake),
            ),
            editor_light(
                module,
                "LMM",
                "Manual Mode",
                bool_word(module.defaults.manipulator_manual_mode),
                bool_color(module.defaults.manipulator_manual_mode),
            ),
            editor_light(
                module,
                "LME",
                "Transfer",
                bool_word(module.defaults.manipulator_transfer_enabled),
                bool_color(module.defaults.manipulator_transfer_enabled),
            ),
            editor_light(
                module,
                "LMT",
                "Resource",
                module.defaults.manipulator_resource_kind.as_str(),
                Color::srgb(0.86, 0.74, 0.30),
            ),
        ],
        crate::ship::ModuleKind::Processor => vec![
            editor_light(
                module,
                "LPY",
                "Recipe",
                module.defaults.processor_recipe.as_str(),
                Color::srgb(0.86, 0.74, 0.30),
            ),
            editor_light(
                module,
                "LPS",
                "Enabled",
                bool_word(module.defaults.processor_enabled),
                bool_color(module.defaults.processor_enabled),
            ),
        ],
        crate::ship::ModuleKind::Computer => vec![
            editor_light(
                module,
                "CCA",
                "Enabled",
                bool_word(module.defaults.computer_enabled),
                bool_color(module.defaults.computer_enabled),
            ),
            editor_light(
                module,
                "CCP",
                "ARCH Program",
                module
                    .arch_program
                    .as_ref()
                    .map(|program| program.name.as_str())
                    .unwrap_or("Balanced Ops"),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "CLP",
                "LUMEN Program",
                module
                    .lumen_program
                    .as_ref()
                    .map(|program| program.name.as_str())
                    .unwrap_or("Balanced Supervision"),
                Color::srgb(0.62, 0.90, 0.80),
            ),
        ],
        crate::ship::ModuleKind::Cockpit => vec![
            editor_light(
                module,
                "HLC",
                "Channel",
                &module.effective_channel().to_string(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "HLM",
                "Default Helm",
                "Manual crew control",
                Color::srgb(0.34, 0.78, 0.46),
            ),
        ],
        _ => vec![
            editor_light(
                module,
                "MOD",
                "Variant",
                module.variant.display_name(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "CHN",
                "Channel",
                &module.effective_channel().to_string(),
                Color::srgb(0.62, 0.90, 0.80),
            ),
        ],
    }
}

fn editor_bar(
    module: &crate::ship::ShipModule,
    register: &str,
    label: &str,
    percent: f32,
    color: Color,
) -> EditorReadout {
    EditorReadout {
        label: format!("{register}{}  {label}", module.effective_channel()),
        value: format!("{:.0}%", percent),
        visual: EditorReadoutVisual::Bar {
            percent: percent.clamp(0.0, 100.0),
            color,
        },
    }
}

fn editor_light(
    module: &crate::ship::ShipModule,
    register: &str,
    label: &str,
    value: &str,
    color: Color,
) -> EditorReadout {
    EditorReadout {
        label: if module.kind.supports_channel() {
            format!("{register}{}  {label}", module.effective_channel())
        } else {
            format!("{register}  {label}")
        },
        value: value.to_string(),
        visual: EditorReadoutVisual::Light { color },
    }
}

fn bool_word(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

fn bool_color(value: bool) -> Color {
    if value {
        Color::srgb(0.34, 0.78, 0.46)
    } else {
        Color::srgb(0.84, 0.28, 0.28)
    }
}

fn format_program_textbox(
    program_editor_state: &crate::state::ProgramTextEditorState,
    module_id: u64,
    language: crate::state::ProgrammingLanguageMode,
    module: &crate::ship::ShipModule,
) -> String {
    let draft_text = if program_editor_state.module_id == Some(module_id)
        && program_editor_state.language == language
    {
        program_editor_state.draft_text.clone()
    } else {
        match language {
            crate::state::ProgrammingLanguageMode::Arch => module
                .arch_program
                .as_ref()
                .map(|program| program.source_text.clone())
                .unwrap_or_default(),
            crate::state::ProgrammingLanguageMode::Lumen => module
                .lumen_program
                .as_ref()
                .map(|program| program.source_text.clone())
                .unwrap_or_default(),
        }
    };
    if program_editor_state.module_id != Some(module_id)
        || program_editor_state.language != language
        || !program_editor_state.focused
    {
        return draft_text;
    }
    let mut display = if program_editor_state.select_all {
        format!("[{draft_text}]")
    } else {
        draft_text
    };
    let cursor_index = program_editor_state
        .cursor_index
        .min(display.chars().count());
    let insert_at = display
        .char_indices()
        .nth(cursor_index)
        .map(|(index, _)| index)
        .unwrap_or(display.len());
    display.insert(insert_at, '|');
    display
}

pub(crate) fn cleanup_editor_entities(
    mut commands: Commands,
    query: Query<Entity, With<EditingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn enemy_entry_label(
    editor_session: &EditorSessionState,
    enemy_editor_state: &EnemyEditorState,
    enemy_library_state: &EnemyShipLibraryState,
) -> String {
    if editor_session.mode == EditorMode::Enemy {
        let Some(entry) = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
        else {
            return "No Enemy Entry".to_string();
        };
        let status_suffix = match enemy_library_state.entry_statuses.get(&entry.id).copied() {
            Some(crate::ship::enemy::EnemyShipEntryValidationStatus::RepairedInMemory) => {
                " [repaired in memory]"
            }
            Some(crate::ship::enemy::EnemyShipEntryValidationStatus::Invalid) => " [invalid]",
            _ => "",
        };
        let dirty_suffix = if enemy_editor_state.dirty {
            " [unsaved]"
        } else {
            ""
        };
        format!("{}{}{}", entry.display_name, status_suffix, dirty_suffix)
    } else {
        "Player Ship".to_string()
    }
}

fn station_actions() -> [StationPanelButtonAction; 21] {
    [
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
    ]
}

fn station_button_default_label(action: StationPanelButtonAction) -> &'static str {
    match action {
        StationPanelButtonAction::HelmThrottle { delta } if delta < 0.0 => "Throttle Down",
        StationPanelButtonAction::HelmThrottle { .. } => "Throttle Up",
        StationPanelButtonAction::HelmTurn { value } if value < 0.0 => "Turn Port",
        StationPanelButtonAction::HelmTurn { value } if value > 0.0 => "Turn Starboard",
        StationPanelButtonAction::HelmTurn { .. } => "Steady",
        StationPanelButtonAction::TurretAdjustAim { delta } if delta < 0.0 => "Trim Right",
        StationPanelButtonAction::TurretAdjustAim { .. } => "Trim Left",
        StationPanelButtonAction::TurretFireToggle => "Toggle Fire Gate",
        StationPanelButtonAction::ReactorAdjustRate { delta } if delta < 0.0 => "Reaction -",
        StationPanelButtonAction::ReactorAdjustRate { .. } => "Reaction +",
        StationPanelButtonAction::ReactorAdjustTurbine { delta } if delta < 0.0 => "Turbine -",
        StationPanelButtonAction::ReactorAdjustTurbine { .. } => "Turbine +",
        StationPanelButtonAction::LogisticsToggleStorageIntake => "Toggle Intake",
        StationPanelButtonAction::LogisticsToggleAirlock => "Cycle Airlock",
        StationPanelButtonAction::LogisticsToggleManipulator => "Toggle Manipulator",
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction }
            if direction < 0 =>
        {
            "Prev Target"
        }
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => "Next Target",
        StationPanelButtonAction::LogisticsCycleResource => "Cycle Resource",
        StationPanelButtonAction::LogisticsToggleProcessor => "Toggle Processor",
        StationPanelButtonAction::ComputerToggleEnabled => "Enable / Disable",
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template",
    }
}
