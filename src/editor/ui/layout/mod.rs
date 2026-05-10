mod station_panel;
mod status_updates;
mod toolbox;

use bevy::prelude::*;
pub(crate) use station_panel::cleanup_editor_entities;
use station_panel::{enemy_entry_label, station_actions, station_button_default_label};
pub(crate) use status_updates::{
    sync_editor_toolbox_layer_sections,
    update_editor_module_overlay,
    update_editor_status_text,
};
use toolbox::{
    foundation_toolbox_groups,
    spawn_foundation_button_grid,
    spawn_layer_button,
    spawn_select_action_button,
    spawn_tool_mode_button,
    spawn_variant_button_grid,
    toolbox_groups,
};

use super::super::helpers::{
    editor_mission_report_text,
    editor_status_line,
    selection_summary,
    variant_tooltip_text,
};
use crate::{
    NORMAL_BUTTON,
    TOOLBOX_WIDTH,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    state::{
        ControlsHelpPanel,
        EditingCleanup,
        EditorAutoHullButton,
        EditorBuildSection,
        EditorCopySelectionButton,
        EditorDeleteSelectionButton,
        EditorLayer,
        EditorMissionReportButton,
        EditorMissionReportButtonText,
        EditorMissionReportText,
        EditorMode,
        EditorOverlayBuildSection,
        EditorPasteSelectionButton,
        EditorPlacementBlocker,
        EditorRoot,
        EditorSelectSection,
        EditorSelectionState,
        EditorSelectionSummaryText,
        EditorSessionState,
        EditorShip,
        EditorStatusText,
        EditorToolMode,
        EditorToolState,
        EditorToolboxPanel,
        EditorToolboxScrollContent,
        EditorToolboxScrollViewport,
        EditorToolboxTooltipText,
        EditorUiState,
        EditorUnderlayBuildSection,
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
        Progression,
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
    tool_state: Res<EditorToolState>,
    selection_state: Res<EditorSelectionState>,
    progression: Res<Progression>,
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
                EditorPlacementBlocker,
                EditorToolboxPanel,
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
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_tool_mode_button(
                            row,
                            "Build",
                            EditorToolMode::Build,
                            tool_state.tool_mode,
                            &mono_font,
                        );
                        spawn_tool_mode_button(
                            row,
                            "Select",
                            EditorToolMode::Select,
                            tool_state.tool_mode,
                            &mono_font,
                        );
                    });

                toolbox
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_layer_button(
                            row,
                            "Underlay",
                            EditorLayer::Underlay,
                            tool_state.active_layer,
                            &mono_font,
                        );
                        spawn_layer_button(
                            row,
                            "Overlay",
                            EditorLayer::Overlay,
                            tool_state.active_layer,
                            &mono_font,
                        );
                    });

                toolbox
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(470.0),
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
                                content
                                    .spawn((
                                        Node {
                                            display: if tool_state.tool_mode == EditorToolMode::Build {
                                                Display::Flex
                                            } else {
                                                Display::None
                                            },
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(14.0),
                                            ..default()
                                        },
                                        EditorBuildSection,
                                    ))
                                    .with_children(|build| {
                                        build
                                            .spawn((
                                                Node {
                                                    display: if tool_state.active_layer
                                                        == EditorLayer::Underlay
                                                    {
                                                        Display::Flex
                                                    } else {
                                                        Display::None
                                                    },
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: Val::Px(14.0),
                                                    ..default()
                                                },
                                                EditorUnderlayBuildSection,
                                            ))
                                            .with_children(|underlay| {
                                                for (title, entries) in foundation_toolbox_groups() {
                                                    underlay.spawn((
                                                        Text::new(title),
                                                        TextFont {
                                                            font: mono_font.clone(),
                                                            font_size: 15.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::srgb(0.74, 0.80, 0.88)),
                                                    ));
                                                    spawn_foundation_button_grid(
                                                        underlay,
                                                        &asset_server,
                                                        &mono_font,
                                                        tool_state.selected_foundation_kind,
                                                        entries,
                                                    );
                                                }
                                            });

                                        build
                                            .spawn((
                                                Node {
                                                    display: if tool_state.active_layer
                                                        == EditorLayer::Overlay
                                                    {
                                                        Display::Flex
                                                    } else {
                                                        Display::None
                                                    },
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: Val::Px(14.0),
                                                    ..default()
                                                },
                                                EditorOverlayBuildSection,
                                            ))
                                            .with_children(|overlay| {
                                                for (title, entries) in toolbox_groups() {
                                                    overlay.spawn((
                                                        Text::new(title),
                                                        TextFont {
                                                            font: mono_font.clone(),
                                                            font_size: 15.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::srgb(0.74, 0.80, 0.88)),
                                                    ));
                                                    spawn_variant_button_grid(
                                                        overlay,
                                                        &asset_server,
                                                        &mono_font,
                                                        editor_session.mode,
                                                        &progression,
                                                        tool_state.ignore_component_limits,
                                                        tool_state.selected_kind,
                                                        tool_state.selected_variant,
                                                        entries,
                                                    );
                                                }
                                            });
                                    });

                                content
                                    .spawn((
                                        Node {
                                            display: if tool_state.tool_mode == EditorToolMode::Select {
                                                Display::Flex
                                            } else {
                                                Display::None
                                            },
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(10.0),
                                            ..default()
                                        },
                                        EditorSelectSection,
                                    ))
                                    .with_children(|select| {
                                        select.spawn((
                                            Text::new(format!(
                                                "Selection\n{}",
                                                selection_summary(&editor_ship.ship, &selection_state)
                                            )),
                                            TextFont {
                                                font: mono_font.clone(),
                                                font_size: UI_BODY_FONT_SIZE,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.90, 0.93, 0.98)),
                                            EditorSelectionSummaryText,
                                        ));
                                        spawn_select_action_button(
                                            select,
                                            "Auto Hull",
                                            Color::srgb(0.46, 0.36, 0.18),
                                            EditorAutoHullButton,
                                            &mono_font,
                                        );
                                        spawn_select_action_button(
                                            select,
                                            "Copy Selection",
                                            Color::srgb(0.26, 0.42, 0.62),
                                            EditorCopySelectionButton,
                                            &mono_font,
                                        );
                                        spawn_select_action_button(
                                            select,
                                            "Paste Clipboard",
                                            Color::srgb(0.22, 0.52, 0.34),
                                            EditorPasteSelectionButton,
                                            &mono_font,
                                        );
                                        spawn_select_action_button(
                                            select,
                                            "Delete Selection",
                                            Color::srgb(0.58, 0.26, 0.18),
                                            EditorDeleteSelectionButton,
                                            &mono_font,
                                        );
                                    });

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

                toolbox.spawn((
                    Text::new(if editor_ui_state.toolbox_tooltip.title.is_empty() {
                        variant_tooltip_text(
                            editor_session.mode,
                            &progression,
                            tool_state.selected_kind,
                            tool_state.selected_variant,
                        )
                    } else {
                        format!(
                            "{}\n{}",
                            editor_ui_state.toolbox_tooltip.title,
                            editor_ui_state.toolbox_tooltip.detail
                        )
                    }),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.74, 0.80, 0.88)),
                    EditorToolboxTooltipText,
                ));

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
                EditorPlacementBlocker,
            ))
            .with_children(|hud| {
                hud.spawn((
                        Text::new(editor_status_line(
                            editor_session.mode,
                            tool_state.tool_mode,
                            tool_state.active_layer,
                            &enemy_entry_label(
                                &editor_session,
                                &enemy_editor_state,
                                &enemy_library_state,
                            ),
                            &editor_ship.ship.name,
                            &tool_state.selected_kind,
                            tool_state.selected_foundation_kind,
                            tool_state.selected_variant,
                            tool_state.selected_rotation,
                            tool_state.selected_channel,
                            tool_state.ignore_component_limits,
                            editor_ship.ship.modules.len(),
                            progression.scrap,
                            &progression,
                            &editor_ship.ship,
                            &selection_state,
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
                    EditorPlacementBlocker,
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
                EditorPlacementBlocker,
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
                                    BackgroundColor(NORMAL_BUTTON),
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
                    display: Display::None,
                    padding: UiRect::all(Val::Px(12.0)),
                    max_width: Val::Px(320.0),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                EditorPlacementBlocker,
                ControlsHelpPanel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        match editor_session.mode {
                            EditorMode::Player => "Refit Controls\nBuild mode: left drag place, right drag erase\nSelect mode: drag marquee, arrows move group, Ctrl+C / Ctrl+V copy and paste, Delete removes, H runs Auto Hull\nE: inspect hovered component\nQ: close component console\nR: rotate build variant clockwise\nZ / X: previous / next variant\nT: repair one damaged selected build variant\nC / V: channel - / + in build mode\nF10: ignore component limits\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save ship  |  F9 reload ship\nTab: return to station",
                            EditorMode::Enemy => "Enemy Editor Controls\nUnlimited component supply across all variants\nBuild mode: left drag place, right drag erase\nSelect mode: drag marquee, arrows move group, Ctrl+C / Ctrl+V copy and paste, Delete removes, H runs Auto Hull\nE: inspect hovered component\nQ: close component console\nR: rotate build variant clockwise\nZ / X: previous / next variant\nC / V: channel - / + in build mode\nF10: ignore component limits\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save library  |  F9 reload library\n[ / ] cycle entry  |  N new enemy\nTab: return to menu",
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
