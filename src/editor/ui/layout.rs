use bevy::prelude::*;

use super::super::helpers::{editor_status_line, module_kind_cost};
use crate::{
    TOOLBOX_COMPONENTS,
    TOOLBOX_WIDTH,
    state::{
        DemoProgression, EditingCleanup, EditorMode, EditorRoot, EditorSessionState, EditorShip,
        EditorStatusText, EditorToolState, EnemyNewButton, EnemyNextButton, EnemyPrevButton,
        EnemyShipLibraryState, LastMissionReport, LeaveEditorButton, ToolboxButton,
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
    enemy_library_state: Res<EnemyShipLibraryState>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
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
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                toolbox.spawn((
                    Text::new(
                        match editor_session.mode {
                            EditorMode::Player => "Click a component, then place it on the grid.\nLeft click: place/replace\nRight click: erase\nQ/E: rotate\nScroll: zoom\nMiddle drag: pan\nF5: save ship\nF9: reload saved ship\nTab or Return: station hub".to_string(),
                            EditorMode::Enemy => "Debug authoring for hostile ship layouts.\nLeft click: place/replace\nRight click: erase\nQ/E: rotate\nScroll: zoom\nMiddle drag: pan\nF5: save enemy library\nF9: reload enemy library\n[ / ]: cycle enemy entry\nN: new enemy entry\nTab or Return: main menu".to_string(),
                        },
                    ),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.74, 0.78, 0.86)),
                ));

                for kind in TOOLBOX_COMPONENTS {
                    toolbox
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(38.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(crate::NORMAL_BUTTON),
                            ToolboxButton { kind },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format!(
                                    "{} [{}]",
                                    kind.as_str(),
                                    module_kind_cost(
                                        kind,
                                        crate::ship::ModuleVariant::default_for_kind(kind),
                                    )
                                )),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }

                if editor_session.mode == EditorMode::Enemy {
                    if let Some(entry) = enemy_library_state
                        .library
                        .selected_or_first(enemy_library_state.selected_index)
                    {
                        toolbox.spawn((
                            Text::new(format!(
                                "Editing: {} [{}]\nThreat: {}  Behavior: {}",
                                entry.display_name, entry.id, entry.threat_tier, entry.behavior_tag
                            )),
                            TextFont {
                                font: mono_font.clone(),
                                font_size: 14.0,
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
                        let mut button = toolbox.spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(34.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
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

                toolbox
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(8.0)),
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
                            font_size: 18.0,
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
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Text::new(editor_status_line(
                        editor_session.mode,
                        enemy_entry_label(&editor_session, &enemy_library_state),
                        &editor_ship.ship.name,
                        &TOOLBOX_COMPONENTS[0],
                        crate::ship::ModuleVariant::default_for_kind(TOOLBOX_COMPONENTS[0]),
                        0,
                        editor_ship.ship.modules.len(),
                        progression.scrap,
                        &last_mission_report,
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    EditorStatusText,
                ));
            });

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
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                crate::state::ComputerProgramPanel,
            ));

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    max_width: Val::Px(320.0),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Refit Controls\nLeft click: place or replace\nRight click: erase\nQ / E: rotate selected part\nScroll: zoom view\nMiddle drag: pan view\nF5: save current ship\nF9: reload saved ship\nTab: return to station\nCosts are shown in [scrap]",
                    ),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
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
    enemy_library_state: Res<EnemyShipLibraryState>,
    tool_state: Res<EditorToolState>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    mut query: Query<&mut Text, With<EditorStatusText>>,
) {
    if !editor_ship.is_changed()
        && !tool_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
    {
        return;
    }

    for mut text in &mut query {
        **text = editor_status_line(
            editor_session.mode,
            enemy_entry_label(&editor_session, &enemy_library_state),
            &editor_ship.ship.name,
            &tool_state.selected_kind,
            tool_state.selected_variant,
            tool_state.selected_rotation,
            editor_ship.ship.modules.len(),
            progression.scrap,
            &last_mission_report,
        );
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

fn enemy_entry_label<'a>(
    editor_session: &'a EditorSessionState,
    enemy_library_state: &'a EnemyShipLibraryState,
) -> &'a str {
    if editor_session.mode == EditorMode::Enemy {
        enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
            .map(|entry| entry.display_name.as_str())
            .unwrap_or("No Enemy Entry")
    } else {
        "Player Ship"
    }
}
