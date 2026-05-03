use bevy::prelude::*;

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
        LastMissionReport,
        LeaveEditorButton,
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
                                                Text::new(format!(
                                                    "{}",
                                                    toolbox_label(
                                                        kind,
                                                        editor_session.mode,
                                                        &progression,
                                                    )
                                                )),
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
                        top: Val::Px(176.0),
                        width: Val::Px(360.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        flex_direction: FlexDirection::Column,
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
                    left: Val::Px(TOOLBOX_WIDTH + 16.0),
                    bottom: Val::Px(16.0),
                    width: Val::Px(640.0),
                    max_height: Val::Px(340.0),
                    overflow: Overflow::clip_y(),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                crate::state::ComputerProgramPanel,
            ));

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
                            EditorMode::Player => "Refit Controls\nLeft click: place or replace\nRight click: erase\nR: repair one damaged selected component\nQ / E: rotate part\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save ship  |  F9 reload ship\nTab: return to station",
                            EditorMode::Enemy => "Enemy Editor Controls\nUnlimited component supply\nLeft click: place or replace\nRight click: erase\nQ / E: rotate part\nScroll over toolbox: browse parts\nScroll over grid: zoom\nMiddle drag: pan view\nF5 save library  |  F9 reload library\n[ / ] cycle entry  |  N new enemy\nTab: return to menu",
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
