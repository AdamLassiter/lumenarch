use bevy::prelude::*;

use super::{
    super::{
        TOOLBOX_COMPONENTS,
        TOOLBOX_WIDTH,
        state::{
            ComputerProgramButton,
            ComputerProgramEntry,
            ComputerProgramPanel,
            ConnectionStatus,
            DemoProgression,
            EditingCleanup,
            EditorRoot,
            EditorShip,
            EditorStatusText,
            EditorToolState,
            LastMissionReport,
            LaunchButton,
            ProgramButtonAction,
        },
    },
    helpers::{editor_status_line, module_kind_cost},
};
use crate::ship::{
    arch::{ArchProgram, ArchProgramTemplate},
    ModuleKind,
    ShipDefinition,
    storage::{load_default_ship, save_default_ship},
};

pub(crate) fn initialize_editor_ship(
    status: Res<ConnectionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
) {
    match load_default_ship() {
        Ok(Some(saved_ship)) => {
            editor_ship.ship = saved_ship;
        }
        Ok(None) => {
            if let Some(snapshot) = status.active_snapshot.as_ref() {
                editor_ship.ship = snapshot.clone();
            } else if editor_ship.ship.name.is_empty() && editor_ship.ship.modules.is_empty() {
                editor_ship.ship = ShipDefinition::empty("Untitled Knot");
            }
        }
        Err(error) => {
            eprintln!("editor: failed to load saved ship: {error}");
            if let Some(snapshot) = status.active_snapshot.as_ref() {
                editor_ship.ship = snapshot.clone();
            } else if editor_ship.ship.name.is_empty() && editor_ship.ship.modules.is_empty() {
                editor_ship.ship = ShipDefinition::empty("Untitled Knot");
            }
        }
    }

    tool_state.selected_kind = ModuleKind::Hull;
    tool_state.selected_rotation = 0;
}

pub(crate) fn spawn_editor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                    Text::new("Toolbox"),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                toolbox.spawn((
                    Text::new(
                        "Click a component, then place it on the grid.\nLeft click: place/replace\nRight click: erase\nQ/E: rotate\nF5: save ship\nF9: reload saved ship\nL or Launch: runtime scene",
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
                                ..default()
                            },
                            BorderRadius::all(Val::Px(8.0)),
                            BackgroundColor(super::super::NORMAL_BUTTON),
                            super::super::state::ToolboxButton { kind },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format!(
                                    "{} [{}]",
                                    kind.as_str(),
                                    module_kind_cost(kind)
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

                toolbox
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(Color::srgb(0.18, 0.50, 0.30)),
                        LaunchButton,
                    ))
                    .with_child((
                        Text::new("Launch"),
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
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Text::new(editor_status_line(
                        &editor_ship.ship.name,
                        &TOOLBOX_COMPONENTS[0],
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
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
                ComputerProgramPanel,
            ));

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    max_width: Val::Px(320.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Editor Controls\nLeft click: place or replace\nRight click: erase\nQ / E: rotate selected part\nF5: save current ship\nF9: reload saved ship\nL: launch mission\nCosts are shown in [scrap]",
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
            &editor_ship.ship.name,
            &tool_state.selected_kind,
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
        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn sync_computer_program_entries(
    mut commands: Commands,
    editor_ship: Res<EditorShip>,
    asset_server: Res<AssetServer>,
    panel_query: Single<Entity, With<ComputerProgramPanel>>,
    existing_query: Query<Entity, With<ComputerProgramEntry>>,
) {
    if !editor_ship.is_changed() {
        return;
    }

    for entity in &existing_query {
        commands.entity(entity).despawn_recursive();
    }

    let panel = panel_query.into_inner();
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new("ARCH Programs"),
            TextFont {
                font: title_font,
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            ComputerProgramEntry,
        ));

        let computers: Vec<_> = editor_ship
            .ship
            .modules
            .iter()
            .filter(|module| module.kind == ModuleKind::Computer)
            .collect();

        if computers.is_empty() {
            panel.spawn((
                Text::new("No computer modules installed"),
                TextFont {
                    font: mono_font,
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.76, 0.80, 0.86)),
                ComputerProgramEntry,
            ));
            return;
        }

        for module in computers {
            let program = module
                .arch_program
                .clone()
                .unwrap_or_else(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps));
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.92)),
                BorderRadius::all(Val::Px(8.0)),
                ComputerProgramEntry,
            ))
            .with_children(|entry| {
                entry.spawn((
                    Text::new(format!(
                        "Computer #{}\nProgram: {}\nConst A / B: {} / {}",
                        module.id,
                        program.template.as_str(),
                        program.constants[0],
                        program.constants[1]
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                ));
                spawn_program_button(
                    entry,
                    &mono_font,
                    "Cycle Template",
                    module.id,
                    ProgramButtonAction::CycleTemplate,
                );
                spawn_program_button(
                    entry,
                    &mono_font,
                    "Const A +1",
                    module.id,
                    ProgramButtonAction::AdjustConstant { index: 0, delta: 1 },
                );
                spawn_program_button(
                    entry,
                    &mono_font,
                    "Const A -1",
                    module.id,
                    ProgramButtonAction::AdjustConstant { index: 0, delta: -1 },
                );
                spawn_program_button(
                    entry,
                    &mono_font,
                    "Const B +1",
                    module.id,
                    ProgramButtonAction::AdjustConstant { index: 1, delta: 1 },
                );
                spawn_program_button(
                    entry,
                    &mono_font,
                    "Const B -1",
                    module.id,
                    ProgramButtonAction::AdjustConstant { index: 1, delta: -1 },
                );
            });
        }
    });
}

fn spawn_program_button(
    entry: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    module_id: u64,
    action: ProgramButtonAction,
) {
    entry
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.24, 0.47, 0.78)),
            BorderRadius::all(Val::Px(6.0)),
            ComputerProgramButton { module_id, action },
            ComputerProgramEntry,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

#[allow(dead_code)]
pub(crate) fn persist_now(editor_ship: &EditorShip) {
    if let Err(error) = save_default_ship(&editor_ship.ship) {
        eprintln!("editor: failed to save ship: {error}");
    }
}
