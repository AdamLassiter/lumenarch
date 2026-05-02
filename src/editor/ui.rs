use bevy::prelude::*;

use super::{
    super::{
        TOOLBOX_COMPONENTS,
        TOOLBOX_WIDTH,
        netcode,
        state::{
            ArchEditorButton,
            ArchEditorButtonAction,
            ArchEditorState,
            ComputerProgramButton,
            ComputerProgramEntry,
            ComputerProgramPanel,
            DemoProgression,
            EditingCleanup,
            EditorMode,
            EditorRoot,
            EditorSessionState,
            EditorShip,
            EditorStatusText,
            EditorToolState,
            EnemyNewButton,
            EnemyNextButton,
            EnemyPrevButton,
            EnemyShipLibraryState,
            LastMissionReport,
            LeaveEditorButton,
            ProgramButtonAction,
        },
    },
    helpers::{editor_status_line, module_kind_cost},
};
use crate::ship::{
    ModuleKind,
    ShipDefinition,
    arch::{ArchProgram, ArchProgramTemplate},
    enemy::load_default_enemy_library,
    storage::{load_default_ship, save_default_ship},
};

pub(crate) fn initialize_editor_ship(
    status: Res<netcode::SessionStatus>,
    rollback_state: Res<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
) {
    match editor_session.mode {
        EditorMode::Player => {
            if matches!(status.phase, netcode::SessionPhase::Connected) {
                editor_ship.ship = rollback_state.editor_ship.clone();
            } else {
                match load_default_ship() {
                    Ok(Some(saved_ship)) => {
                        editor_ship.ship = saved_ship;
                    }
                    Ok(None) => {
                        if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
                            editor_ship.ship = snapshot.clone();
                        } else if editor_ship.ship.name.is_empty()
                            && editor_ship.ship.modules.is_empty()
                        {
                            editor_ship.ship = ShipDefinition::empty("Untitled Knot");
                        }
                    }
                    Err(error) => {
                        eprintln!("editor: failed to load saved ship: {error}");
                        if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
                            editor_ship.ship = snapshot.clone();
                        } else if editor_ship.ship.name.is_empty()
                            && editor_ship.ship.modules.is_empty()
                        {
                            editor_ship.ship = ShipDefinition::empty("Untitled Knot");
                        }
                    }
                }
            }
        }
        EditorMode::Enemy => {
            match load_default_enemy_library() {
                Ok(Some(library)) => {
                    enemy_library_state.library = library;
                }
                Ok(None) => {
                    enemy_library_state.library = crate::ship::enemy::EnemyShipLibrary::seeded();
                }
                Err(error) => {
                    eprintln!("editor: failed to load enemy ship library: {error}");
                    enemy_library_state.library = crate::ship::enemy::EnemyShipLibrary::seeded();
                }
            }
            enemy_library_state.library.ensure_seeded();
            enemy_library_state.selected_index = enemy_library_state
                .selected_index
                .min(enemy_library_state.library.entries.len().saturating_sub(1));
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first(enemy_library_state.selected_index)
            {
                editor_ship.ship = entry.ship.clone();
            }
        }
    }

    tool_state.selected_kind = ModuleKind::Hull;
    tool_state.selected_variant = crate::ship::ModuleVariant::default_for_kind(ModuleKind::Hull);
    tool_state.selected_rotation = 0;
    arch_editor_state.selected_module_id = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Computer)
        .map(|module| module.id);
    arch_editor_state.selected_line = 0;
}

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
                            BackgroundColor(super::super::NORMAL_BUTTON),
                            super::super::state::ToolboxButton { kind },
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
                ComputerProgramPanel,
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

pub(crate) fn cleanup_editor_entities(
    mut commands: Commands,
    query: Query<Entity, With<EditingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn sync_computer_program_entries(
    mut commands: Commands,
    editor_ship: Res<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    asset_server: Res<AssetServer>,
    panel_query: Single<Entity, With<ComputerProgramPanel>>,
    existing_query: Query<Entity, With<ComputerProgramEntry>>,
) {
    if !editor_ship.is_changed() && !arch_editor_state.is_changed() {
        return;
    }

    for entity in &existing_query {
        commands.entity(entity).despawn();
    }

    let panel = panel_query.into_inner();
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new("ARCH Workshop"),
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

        panel.spawn((
            Text::new("Select a computer, then edit its instruction list with buttons. This is a first-pass workshop: opcode cycling, register cycling, immediate/jump tweaks, line add/remove, plus template and constant controls."),
            TextFont {
                font: mono_font.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.76, 0.80, 0.86)),
            ComputerProgramEntry,
        ));

        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                ComputerProgramEntry,
            ))
            .with_children(|buttons| {
                for module in &computers {
                    spawn_arch_editor_button(
                        buttons,
                        &mono_font,
                        &format!(
                            "{}CPU #{}",
                            if arch_editor_state.selected_module_id == Some(module.id) {
                                "Selected "
                            } else {
                                ""
                            },
                            module.id
                        ),
                        ArchEditorButtonAction::SelectModule(module.id),
                    );
                }
            });

        let selected_module = arch_editor_state
            .selected_module_id
            .and_then(|module_id| computers.iter().find(|module| module.id == module_id).copied())
            .unwrap_or(computers[0]);
        let program = selected_module
            .arch_program
            .clone()
            .unwrap_or_else(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps));

        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.92)),
                ComputerProgramEntry,
            ))
            .with_children(|entry| {
                entry.spawn((
                    Text::new(format!(
                        "Computer #{}\nProgram: {}\nTemplate: {}\nConst A / B: {} / {}\nValidation: {}",
                        selected_module.id,
                        program.name,
                        program.template.as_str(),
                        program.constants[0],
                        program.constants[1],
                        program_validation_summary(&program),
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                ));
                entry
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            "Cycle Template",
                            selected_module.id,
                            ProgramButtonAction::CycleTemplate,
                        );
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            "Const A +1",
                            selected_module.id,
                            ProgramButtonAction::AdjustConstant { index: 0, delta: 1 },
                        );
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            "Const A -1",
                            selected_module.id,
                            ProgramButtonAction::AdjustConstant { index: 0, delta: -1 },
                        );
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            "Const B +1",
                            selected_module.id,
                            ProgramButtonAction::AdjustConstant { index: 1, delta: 1 },
                        );
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            "Const B -1",
                            selected_module.id,
                            ProgramButtonAction::AdjustConstant { index: 1, delta: -1 },
                        );
                        spawn_arch_editor_button(
                            buttons,
                            &mono_font,
                            "Add Line",
                            ArchEditorButtonAction::AddLine(selected_module.id),
                        );
                    });

                for (line_index, instruction) in program.instructions.iter().enumerate() {
                    let selected = arch_editor_state.selected_line == line_index;
                    entry
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(6.0),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(if selected {
                                Color::srgba(0.22, 0.28, 0.37, 0.98)
                            } else {
                                Color::srgba(0.15, 0.18, 0.24, 0.92)
                            }),
                        ))
                        .with_children(|line| {
                            line.spawn((
                                Text::new(format!(
                                    "L{:02}  {}",
                                    line_index,
                                    instruction_summary(instruction)
                                )),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            line.spawn(Node {
                                width: Val::Percent(100.0),
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|buttons| {
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    if selected { "Selected" } else { "Select" },
                                    ArchEditorButtonAction::SelectLine {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Opcode",
                                    ArchEditorButtonAction::CycleOpcode {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Dst",
                                    ArchEditorButtonAction::CycleDst {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Src A",
                                    ArchEditorButtonAction::CycleSrcA {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Src B",
                                    ArchEditorButtonAction::CycleSrcB {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Imm A +",
                                    ArchEditorButtonAction::AdjustImmediateA {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: 1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Imm A -",
                                    ArchEditorButtonAction::AdjustImmediateA {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: -1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Imm B +",
                                    ArchEditorButtonAction::AdjustImmediateB {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: 1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Imm B -",
                                    ArchEditorButtonAction::AdjustImmediateB {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: -1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Jump +",
                                    ArchEditorButtonAction::AdjustJump {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: 1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Jump -",
                                    ArchEditorButtonAction::AdjustJump {
                                        module_id: selected_module.id,
                                        line: line_index,
                                        delta: -1,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Insert",
                                    ArchEditorButtonAction::InsertLineAfter {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Up",
                                    ArchEditorButtonAction::MoveLineUp {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Down",
                                    ArchEditorButtonAction::MoveLineDown {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Remove",
                                    ArchEditorButtonAction::RemoveLine {
                                        module_id: selected_module.id,
                                        line: line_index,
                                    },
                                );
                            });
                        });
                }
            });
    });
}

fn spawn_program_button(
    entry: &mut ChildSpawnerCommands,
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
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.24, 0.47, 0.78)),
            ComputerProgramButton { module_id, action },
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

fn spawn_arch_editor_button(
    entry: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    action: ArchEditorButtonAction,
) {
    entry
        .spawn((
            Button,
            Node {
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.40, 0.62)),
            ArchEditorButton { action },
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn instruction_summary(instruction: &crate::ship::arch::ArchInstruction) -> String {
    use crate::ship::arch::{ArchInstruction, ArchValueRef};

    let value_ref = |value: &ArchValueRef| match value {
        ArchValueRef::Register(register) => register.as_str().to_string(),
        ArchValueRef::Immediate(value) => format!("#{value}"),
    };

    match instruction {
        ArchInstruction::Nop => "NOP".to_string(),
        ArchInstruction::Mov { dst, src } => format!("MOV {} <- {}", dst.as_str(), value_ref(src)),
        ArchInstruction::Add { dst, lhs, rhs } => {
            format!(
                "ADD {} <- {} + {}",
                dst.as_str(),
                value_ref(lhs),
                value_ref(rhs)
            )
        }
        ArchInstruction::Sub { dst, lhs, rhs } => {
            format!(
                "SUB {} <- {} - {}",
                dst.as_str(),
                value_ref(lhs),
                value_ref(rhs)
            )
        }
        ArchInstruction::Gt { dst, lhs, rhs } => {
            format!(
                "GT {} <- {} > {}",
                dst.as_str(),
                value_ref(lhs),
                value_ref(rhs)
            )
        }
        ArchInstruction::Lt { dst, lhs, rhs } => {
            format!(
                "LT {} <- {} < {}",
                dst.as_str(),
                value_ref(lhs),
                value_ref(rhs)
            )
        }
        ArchInstruction::Jnz { cond, target } => format!("JNZ {} -> {}", value_ref(cond), target),
        ArchInstruction::Jmp { target } => format!("JMP {}", target),
        other => format!("{other:?}"),
    }
}

fn program_validation_summary(program: &ArchProgram) -> String {
    let mut warnings = Vec::new();

    for (index, instruction) in program.instructions.iter().enumerate() {
        if let Some(dst) = instruction_destination(instruction)
            && !dst.is_writable()
        {
            warnings.push(format!("L{index:02} writes read-only {}", dst.as_str()));
        }
        if let Some(target) = instruction_jump_target(instruction)
            && target >= program.instructions.len()
        {
            warnings.push(format!("L{index:02} jumps past end"));
        }
    }

    if warnings.is_empty() {
        "ready".to_string()
    } else {
        warnings.join(" | ")
    }
}

fn instruction_destination(
    instruction: &crate::ship::arch::ArchInstruction,
) -> Option<crate::ship::arch::ArchRegister> {
    use crate::ship::arch::ArchInstruction;

    match instruction {
        ArchInstruction::Mov { dst, .. }
        | ArchInstruction::Clp { dst, .. }
        | ArchInstruction::Abs { dst, .. }
        | ArchInstruction::Neg { dst, .. }
        | ArchInstruction::Add { dst, .. }
        | ArchInstruction::Sub { dst, .. }
        | ArchInstruction::Mul { dst, .. }
        | ArchInstruction::Div { dst, .. }
        | ArchInstruction::Mod { dst, .. }
        | ArchInstruction::Pow { dst, .. }
        | ArchInstruction::Sqrt { dst, .. }
        | ArchInstruction::Log { dst, .. }
        | ArchInstruction::Sin { dst, .. }
        | ArchInstruction::Cos { dst, .. }
        | ArchInstruction::Tan { dst, .. }
        | ArchInstruction::Asin { dst, .. }
        | ArchInstruction::Acos { dst, .. }
        | ArchInstruction::Atan { dst, .. }
        | ArchInstruction::Atn2 { dst, .. }
        | ArchInstruction::Gt { dst, .. }
        | ArchInstruction::Gte { dst, .. }
        | ArchInstruction::Lt { dst, .. }
        | ArchInstruction::Lte { dst, .. }
        | ArchInstruction::Eq { dst, .. }
        | ArchInstruction::Neq { dst, .. }
        | ArchInstruction::And { dst, .. }
        | ArchInstruction::Or { dst, .. }
        | ArchInstruction::Xor { dst, .. }
        | ArchInstruction::Not { dst, .. }
        | ArchInstruction::Min { dst, .. }
        | ArchInstruction::Max { dst, .. }
        | ArchInstruction::Avg { dst, .. }
        | ArchInstruction::Sgn { dst, .. }
        | ArchInstruction::Lerp { dst, .. } => Some(*dst),
        _ => None,
    }
}

fn instruction_jump_target(instruction: &crate::ship::arch::ArchInstruction) -> Option<usize> {
    use crate::ship::arch::ArchInstruction;

    match instruction {
        ArchInstruction::Jmp { target }
        | ArchInstruction::Jeq { target, .. }
        | ArchInstruction::Jne { target, .. }
        | ArchInstruction::Jgt { target, .. }
        | ArchInstruction::Jge { target, .. }
        | ArchInstruction::Jlt { target, .. }
        | ArchInstruction::Jle { target, .. }
        | ArchInstruction::Jnz { target, .. } => Some(*target),
        _ => None,
    }
}

#[allow(dead_code)]
pub(crate) fn persist_now(editor_ship: &EditorShip) {
    if let Err(error) = save_default_ship(&editor_ship.ship) {
        eprintln!("editor: failed to save ship: {error}");
    }
}
