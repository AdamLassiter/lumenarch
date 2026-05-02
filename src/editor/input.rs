use std::ops::DerefMut;

use bevy::{
    input::mouse::{MouseButton, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use super::{
    super::{
        HOVERED_BUTTON,
        PRESSED_BUTTON,
        netcode,
        state::{
            ArchEditorButton,
            ArchEditorButtonAction,
            ArchEditorState,
            ComputerProgramButton,
            DemoProgression,
            EditorMode,
            FrontendMode,
            EditorPanState,
            EditorSessionState,
            EditorShip,
            EditorToolState,
            EditorViewState,
            EnemyNewButton,
            EnemyNextButton,
            EnemyPrevButton,
            EnemyShipLibraryState,
            LeaveEditorButton,
            ProgramButtonAction,
            ToolboxButton,
        },
    },
    helpers::{cursor_grid_position, is_cursor_over_editor_ui, module_kind_cost},
};
use crate::ship::{
    ModuleVariant,
    ShipModule,
    arch::{ArchInstruction, ArchProgram, ArchProgramTemplate, ArchRegister, ArchValueRef},
    enemy::{load_default_enemy_library, save_default_enemy_library},
    storage::{load_default_ship, save_default_ship},
};

pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (&Interaction, &ToolboxButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    progression: Res<DemoProgression>,
    mut tool_state: ResMut<EditorToolState>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        let affordable = progression.scrap
            >= module_kind_cost(button.kind, ModuleVariant::default_for_kind(button.kind));
        match *interaction {
            Interaction::Pressed => {
                tool_state.selected_kind = button.kind;
                tool_state.selected_variant = ModuleVariant::default_for_kind(button.kind);
                *background = BackgroundColor(if affordable {
                    PRESSED_BUTTON
                } else {
                    super::SELECTED_UNAFFORDABLE_BUTTON
                });
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if affordable {
                    HOVERED_BUTTON
                } else {
                    super::UNAFFORDABLE_BUTTON
                });
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn leave_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LeaveEditorButton>),
    >,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                match editor_session.mode {
                    EditorMode::Player => {
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LeaveEditor,
                            ..Default::default()
                        });
                    }
                    EditorMode::Enemy => {
                        rollback_state.phase = netcode::RollbackPhase::Docked;
                        next_mode.set(FrontendMode::Menu);
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.56, 0.40, 0.26));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.46, 0.34, 0.22));
            }
        }
    }
}

pub(crate) fn leave_editor_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        match editor_session.mode {
            EditorMode::Player => {
                pending_meta.0 = Some(netcode::PendingMetaCommand {
                    op: netcode::RollbackMetaOp::LeaveEditor,
                    ..Default::default()
                });
            }
            EditorMode::Enemy => {
                rollback_state.phase = netcode::RollbackPhase::Docked;
                next_mode.set(FrontendMode::Menu);
            }
        }
    }
}

pub(crate) fn computer_program_button_system(
    mut interaction_query: Query<
        (&Interaction, &ComputerProgramButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let Some(module) = editor_ship
                    .ship
                    .modules
                    .iter_mut()
                    .find(|module| module.id == button.module_id)
                else {
                    continue;
                };
                let program = module.arch_program.get_or_insert_with(|| {
                    ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                });
                match button.action {
                    ProgramButtonAction::CycleTemplate => {
                        *program = ArchProgram::from_template(program.template.next());
                    }
                    ProgramButtonAction::AdjustConstant { index, delta } => {
                        if let Some(constant) = program.constants.get_mut(index) {
                            *constant = (*constant + delta).clamp(0, 20);
                        }
                    }
                }
                rollback_state.editor_ship = editor_ship.ship.clone();
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn arch_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &ArchEditorButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut arch_editor_state: ResMut<ArchEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                apply_arch_editor_action(&mut arch_editor_state, &mut editor_ship, button.action);
                rollback_state.editor_ship = editor_ship.ship.clone();
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn rotate_selected_tool(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool_state: ResMut<EditorToolState>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 3) % 4;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 1) % 4;
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, -1);
    }

    if keys.just_pressed(KeyCode::KeyX) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, 1);
    }
}

pub(crate) fn place_or_remove_tile(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<DemoProgression>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        let selected_cost = if editor_session.mode == EditorMode::Player {
            module_kind_cost(tool_state.selected_kind, tool_state.selected_variant)
        } else {
            0
        };
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            if existing.kind == tool_state.selected_kind
                && existing.variant == tool_state.selected_variant
            {
                existing.rotation_quadrants = tool_state.selected_rotation;
                return;
            }

            let existing_cost = if editor_session.mode == EditorMode::Player {
                module_kind_cost(existing.kind, existing.variant)
            } else {
                0
            };
            let additional_cost = selected_cost.saturating_sub(existing_cost);
            if progression.scrap < additional_cost {
                return;
            }

            progression.scrap -= additional_cost;
            existing.kind = tool_state.selected_kind;
            existing.variant = tool_state
                .selected_variant
                .normalize_for_kind(tool_state.selected_kind);
            existing.rotation_quadrants = tool_state.selected_rotation;
        } else {
            if progression.scrap < selected_cost {
                return;
            }
            progression.scrap -= selected_cost;
            let next_id = editor_ship.ship.next_module_id();
            editor_ship.ship.replace_module(ShipModule::new(
                next_id,
                tool_state.selected_kind,
                grid_x,
                grid_y,
                tool_state.selected_rotation,
            ));
            if let Some(module) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
                module.variant = tool_state
                    .selected_variant
                    .normalize_for_kind(tool_state.selected_kind);
            }
        }
        rollback_state.editor_ship = editor_ship.ship.clone();
        rollback_state.progression = progression.clone();
    }

    if buttons.just_pressed(MouseButton::Right) {
        editor_ship.ship.remove_module_at(grid_x, grid_y);
        rollback_state.editor_ship = editor_ship.ship.clone();
    }
}

pub(crate) fn save_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => save_default_enemy_library(&enemy_library_state.library),
    };
    if let Err(error) = result {
        eprintln!("editor: failed to save ship data: {error}");
    }
}

pub(crate) fn pan_and_zoom_editor_view(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut pan_state: ResMut<EditorPanState>,
    mut view_state: ResMut<EditorViewState>,
    camera_query: Single<
        (&mut Transform, &mut Projection),
        (With<Camera2d>, With<crate::state::MainCamera>),
    >,
) {
    let window = window.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();
    let Projection::Orthographic(projection) = projection.deref_mut() else {
        return;
    };

    for event in mouse_wheel.read() {
        let zoom_step = (1.0 - event.y * 0.08).clamp(0.75, 1.25);
        view_state.zoom = (view_state.zoom * zoom_step).clamp(0.35, 2.75);
    }

    let cursor_position = window.cursor_position();
    if mouse_buttons.pressed(MouseButton::Middle) && !is_cursor_over_editor_ui(window) {
        if let Some(cursor) = cursor_position {
            if let Some(previous_cursor) = pan_state.last_cursor {
                let delta = cursor - previous_cursor;
                view_state.center.x -= delta.x * view_state.zoom;
                view_state.center.y += delta.y * view_state.zoom;
            }
            pan_state.last_cursor = Some(cursor);
        } else {
            pan_state.last_cursor = None;
        }
    } else {
        pan_state.last_cursor = None;
    }

    camera_transform.translation.x = view_state.center.x;
    camera_transform.translation.y = view_state.center.y;
    projection.scale = view_state.zoom;
}

pub(crate) fn load_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    status: Res<netcode::SessionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }

    match editor_session.mode {
        EditorMode::Player => match load_default_ship() {
            Ok(Some(saved_ship)) => {
                editor_ship.ship = saved_ship;
                rollback_state.editor_ship = editor_ship.ship.clone();
            }
            Ok(None) => {
                eprintln!("editor: no saved ship file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load ship: {error}");
            }
        },
        EditorMode::Enemy => match load_default_enemy_library() {
            Ok(Some(library)) => {
                enemy_library_state.library = library;
                enemy_library_state.library.ensure_seeded();
                enemy_library_state.selected_index = enemy_library_state
                    .selected_index
                    .min(enemy_library_state.library.entries.len().saturating_sub(1));
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                    rollback_state.editor_ship = editor_ship.ship.clone();
                }
            }
            Ok(None) => {
                eprintln!("editor: no enemy ship library file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load enemy ship library: {error}");
            }
        },
    }
}

pub(crate) fn persist_editor_ship(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if !editor_ship.is_changed() {
        return;
    }
    rollback_state.editor_ship = editor_ship.ship.clone();

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => {
            enemy_library_state.library.ensure_seeded();
            let selected_index = enemy_library_state.selected_index;
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first_mut(selected_index)
            {
                entry.ship = editor_ship.ship.clone();
                entry.display_name = editor_ship.ship.name.clone();
            }
            save_default_enemy_library(&enemy_library_state.library)
        }
    };

    if let Err(error) = result {
        eprintln!("editor: failed to autosave ship: {error}");
    }
}

pub(crate) fn enemy_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&EnemyPrevButton>,
            Option<&EnemyNextButton>,
            Option<&EnemyNewButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    for (interaction, mut background, prev, next, new_entry) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                enemy_library_state.library.ensure_seeded();
                if prev.is_some() && !enemy_library_state.library.entries.is_empty() {
                    let len = enemy_library_state.library.entries.len();
                    enemy_library_state.selected_index =
                        (enemy_library_state.selected_index + len - 1) % len;
                } else if next.is_some() && !enemy_library_state.library.entries.is_empty() {
                    enemy_library_state.selected_index = (enemy_library_state.selected_index + 1)
                        % enemy_library_state.library.entries.len();
                } else if new_entry.is_some() {
                    enemy_library_state.selected_index =
                        enemy_library_state.library.add_blank_entry();
                }
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                    rollback_state.editor_ship = editor_ship.ship.clone();
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.24, 0.32, 0.48));
            }
        }
    }
}

pub(crate) fn enemy_library_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    let mut changed = false;
    enemy_library_state.library.ensure_seeded();
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketLeft) {
        let len = enemy_library_state.library.entries.len();
        enemy_library_state.selected_index = (enemy_library_state.selected_index + len - 1) % len;
        changed = true;
    }
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketRight) {
        enemy_library_state.selected_index =
            (enemy_library_state.selected_index + 1) % enemy_library_state.library.entries.len();
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        enemy_library_state.selected_index = enemy_library_state.library.add_blank_entry();
        changed = true;
    }

    if changed
        && let Some(entry) = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
    {
        editor_ship.ship = entry.ship.clone();
        rollback_state.editor_ship = editor_ship.ship.clone();
    }
}

fn apply_arch_editor_action(
    arch_editor_state: &mut ArchEditorState,
    editor_ship: &mut EditorShip,
    action: ArchEditorButtonAction,
) {
    match action {
        ArchEditorButtonAction::SelectModule(module_id) => {
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = 0;
        }
        ArchEditorButtonAction::SelectLine { module_id, line } => {
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = line;
        }
        ArchEditorButtonAction::AddLine(module_id) => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            program.instructions.push(ArchInstruction::Nop);
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = program.instructions.len().saturating_sub(1);
        }
        ArchEditorButtonAction::InsertLineAfter { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            let insert_at = (line + 1).min(program.instructions.len());
            program.instructions.insert(insert_at, ArchInstruction::Nop);
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = insert_at;
        }
        ArchEditorButtonAction::RemoveLine { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            if program.instructions.len() > 1 && line < program.instructions.len() {
                program.instructions.remove(line);
                arch_editor_state.selected_line = arch_editor_state
                    .selected_line
                    .min(program.instructions.len().saturating_sub(1));
            }
        }
        ArchEditorButtonAction::MoveLineUp { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            if line > 0 && line < program.instructions.len() {
                program.instructions.swap(line, line - 1);
                arch_editor_state.selected_module_id = Some(module_id);
                arch_editor_state.selected_line = line - 1;
            }
        }
        ArchEditorButtonAction::MoveLineDown { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            if line + 1 < program.instructions.len() {
                program.instructions.swap(line, line + 1);
                arch_editor_state.selected_module_id = Some(module_id);
                arch_editor_state.selected_line = line + 1;
            }
        }
        ArchEditorButtonAction::CycleOpcode { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                cycle_instruction_opcode(instruction);
            });
        }
        ArchEditorButtonAction::CycleDst { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                cycle_dst_register(instruction);
            });
        }
        ArchEditorButtonAction::CycleSrcA { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                cycle_value_operand(instruction, OperandSlot::A);
            });
        }
        ArchEditorButtonAction::CycleSrcB { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                cycle_value_operand(instruction, OperandSlot::B);
            });
        }
        ArchEditorButtonAction::AdjustImmediateA {
            module_id,
            line,
            delta,
        } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                adjust_immediate_operand(instruction, OperandSlot::A, delta);
            });
        }
        ArchEditorButtonAction::AdjustImmediateB {
            module_id,
            line,
            delta,
        } => {
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                adjust_immediate_operand(instruction, OperandSlot::B, delta);
            });
        }
        ArchEditorButtonAction::AdjustJump {
            module_id,
            line,
            delta,
        } => {
            let max_len = editor_ship
                .ship
                .modules
                .iter()
                .find(|module| module.id == module_id)
                .and_then(|module| module.arch_program.as_ref())
                .map(|program| program.instructions.len())
                .unwrap_or(1);
            mutate_instruction(editor_ship, module_id, line, |instruction| {
                adjust_jump_target(instruction, delta, max_len);
            });
        }
        ArchEditorButtonAction::RenameModuleProgram(module_id) => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            program.name = format!("ARCH-{}", module_id);
        }
    }
}

fn module_program_mut(editor_ship: &mut EditorShip, module_id: u64) -> Option<&mut ArchProgram> {
    editor_ship
        .ship
        .modules
        .iter_mut()
        .find(|module| module.id == module_id)
        .and_then(|module| {
            if module.kind != crate::ship::ModuleKind::Computer {
                return None;
            }
            Some(module.arch_program.get_or_insert_with(|| {
                ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
            }))
        })
}

fn mutate_instruction(
    editor_ship: &mut EditorShip,
    module_id: u64,
    line: usize,
    mutate: impl FnOnce(&mut ArchInstruction),
) {
    let Some(program) = module_program_mut(editor_ship, module_id) else {
        return;
    };
    let Some(instruction) = program.instructions.get_mut(line) else {
        return;
    };
    mutate(instruction);
}

#[derive(Clone, Copy)]
enum EditableOpcode {
    Nop,
    Mov,
    Add,
    Sub,
    Gt,
    Lt,
    Jnz,
    Jmp,
}

#[derive(Clone, Copy)]
enum OperandSlot {
    A,
    B,
}

fn cycle_instruction_opcode(instruction: &mut ArchInstruction) {
    let editable = editable_instruction_from(instruction);
    let next_kind = match editable.kind {
        EditableOpcode::Nop => EditableOpcode::Mov,
        EditableOpcode::Mov => EditableOpcode::Add,
        EditableOpcode::Add => EditableOpcode::Sub,
        EditableOpcode::Sub => EditableOpcode::Gt,
        EditableOpcode::Gt => EditableOpcode::Lt,
        EditableOpcode::Lt => EditableOpcode::Jnz,
        EditableOpcode::Jnz => EditableOpcode::Jmp,
        EditableOpcode::Jmp => EditableOpcode::Nop,
    };
    editable.write_back(instruction, next_kind);
}

fn cycle_dst_register(instruction: &mut ArchInstruction) {
    let mut editable = editable_instruction_from(instruction);
    if let Some(dst) = editable.dst {
        editable.dst = Some(next_writable_register(dst));
        let kind = editable.kind;
        editable.write_back(instruction, kind);
    }
}

fn cycle_value_operand(instruction: &mut ArchInstruction, slot: OperandSlot) {
    let mut editable = editable_instruction_from(instruction);
    let operand = match slot {
        OperandSlot::A => &mut editable.a,
        OperandSlot::B => &mut editable.b,
    };
    *operand = Some(match operand.take().unwrap_or(ArchValueRef::Immediate(0)) {
        ArchValueRef::Register(_) => ArchValueRef::Immediate(0),
        ArchValueRef::Immediate(_) => ArchValueRef::Register(ArchRegister::Gp0),
    });
    let kind = editable.kind;
    editable.write_back(instruction, kind);
}

fn adjust_immediate_operand(instruction: &mut ArchInstruction, slot: OperandSlot, delta: i32) {
    let mut editable = editable_instruction_from(instruction);
    let operand = match slot {
        OperandSlot::A => &mut editable.a,
        OperandSlot::B => &mut editable.b,
    };
    let value = match operand.take().unwrap_or(ArchValueRef::Immediate(0)) {
        ArchValueRef::Register(_) => delta,
        ArchValueRef::Immediate(value) => value + delta,
    };
    *operand = Some(ArchValueRef::Immediate(value.clamp(-20, 20)));
    let kind = editable.kind;
    editable.write_back(instruction, kind);
}

fn adjust_jump_target(instruction: &mut ArchInstruction, delta: i32, max_len: usize) {
    let mut editable = editable_instruction_from(instruction);
    let current = editable.jump.unwrap_or(0) as i32;
    editable.jump = Some((current + delta).clamp(0, max_len.saturating_sub(1) as i32) as usize);
    let kind = editable.kind;
    editable.write_back(instruction, kind);
}

#[derive(Clone)]
struct EditableInstruction {
    kind: EditableOpcode,
    dst: Option<ArchRegister>,
    a: Option<ArchValueRef>,
    b: Option<ArchValueRef>,
    jump: Option<usize>,
}

impl EditableInstruction {
    fn write_back(self, instruction: &mut ArchInstruction, force_kind: EditableOpcode) {
        let dst = self.dst.unwrap_or(ArchRegister::Gp0);
        let a = self.a.unwrap_or(ArchValueRef::Immediate(0));
        let b = self.b.unwrap_or(ArchValueRef::Immediate(0));
        let jump = self.jump.unwrap_or(0);
        *instruction = match force_kind {
            EditableOpcode::Nop => ArchInstruction::Nop,
            EditableOpcode::Mov => ArchInstruction::Mov { dst, src: a },
            EditableOpcode::Add => ArchInstruction::Add {
                dst,
                lhs: a,
                rhs: b,
            },
            EditableOpcode::Sub => ArchInstruction::Sub {
                dst,
                lhs: a,
                rhs: b,
            },
            EditableOpcode::Gt => ArchInstruction::Gt {
                dst,
                lhs: a,
                rhs: b,
            },
            EditableOpcode::Lt => ArchInstruction::Lt {
                dst,
                lhs: a,
                rhs: b,
            },
            EditableOpcode::Jnz => ArchInstruction::Jnz {
                cond: a,
                target: jump,
            },
            EditableOpcode::Jmp => ArchInstruction::Jmp { target: jump },
        };
    }
}

fn editable_instruction_from(instruction: &ArchInstruction) -> EditableInstruction {
    match instruction {
        ArchInstruction::Nop => EditableInstruction {
            kind: EditableOpcode::Nop,
            dst: Some(ArchRegister::Gp0),
            a: Some(ArchValueRef::Immediate(0)),
            b: Some(ArchValueRef::Immediate(0)),
            jump: Some(0),
        },
        ArchInstruction::Mov { dst, src } => EditableInstruction {
            kind: EditableOpcode::Mov,
            dst: Some(*dst),
            a: Some(src.clone()),
            b: Some(ArchValueRef::Immediate(0)),
            jump: Some(0),
        },
        ArchInstruction::Add { dst, lhs, rhs } => EditableInstruction {
            kind: EditableOpcode::Add,
            dst: Some(*dst),
            a: Some(lhs.clone()),
            b: Some(rhs.clone()),
            jump: Some(0),
        },
        ArchInstruction::Sub { dst, lhs, rhs } => EditableInstruction {
            kind: EditableOpcode::Sub,
            dst: Some(*dst),
            a: Some(lhs.clone()),
            b: Some(rhs.clone()),
            jump: Some(0),
        },
        ArchInstruction::Gt { dst, lhs, rhs } => EditableInstruction {
            kind: EditableOpcode::Gt,
            dst: Some(*dst),
            a: Some(lhs.clone()),
            b: Some(rhs.clone()),
            jump: Some(0),
        },
        ArchInstruction::Lt { dst, lhs, rhs } => EditableInstruction {
            kind: EditableOpcode::Lt,
            dst: Some(*dst),
            a: Some(lhs.clone()),
            b: Some(rhs.clone()),
            jump: Some(0),
        },
        ArchInstruction::Jnz { cond, target } => EditableInstruction {
            kind: EditableOpcode::Jnz,
            dst: Some(ArchRegister::Gp0),
            a: Some(cond.clone()),
            b: Some(ArchValueRef::Immediate(0)),
            jump: Some(*target),
        },
        ArchInstruction::Jmp { target } => EditableInstruction {
            kind: EditableOpcode::Jmp,
            dst: Some(ArchRegister::Gp0),
            a: Some(ArchValueRef::Immediate(0)),
            b: Some(ArchValueRef::Immediate(0)),
            jump: Some(*target),
        },
        _ => EditableInstruction {
            kind: EditableOpcode::Nop,
            dst: Some(ArchRegister::Gp0),
            a: Some(ArchValueRef::Immediate(0)),
            b: Some(ArchValueRef::Immediate(0)),
            jump: Some(0),
        },
    }
}

fn next_register(current: ArchRegister) -> ArchRegister {
    use ArchRegister::*;
    const REGISTERS: [ArchRegister; 19] = [
        Gp0,
        Gp1,
        Gp2,
        Gp3,
        ShipPowerReserve,
        ShipAverageHeat,
        MissionThreat,
        ReactorHeat,
        ReactorInstability,
        StorageRawSalvage,
        StorageRepairCharge,
        ProcessorRawSalvage,
        ProcessorRepairCharge,
        TurretReady,
        TurretCooldown,
        CmdReactorBias,
        CmdLogisticsEnable,
        CmdLogisticsPreference,
        CmdTurretAssist,
    ];
    let index = REGISTERS
        .iter()
        .position(|register| *register == current)
        .unwrap_or(0);
    REGISTERS[(index + 1) % REGISTERS.len()]
}

fn next_writable_register(current: ArchRegister) -> ArchRegister {
    let mut next = current;
    for _ in 0..32 {
        next = next_register(next);
        if next.is_writable() {
            return next;
        }
    }
    ArchRegister::Gp0
}
