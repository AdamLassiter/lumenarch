use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    ship::{
        ModuleKind,
        arch::{
            ArchInstruction,
            ArchParseDiagnostic,
            ArchProgram,
            ArchProgramTemplate,
            ArchRegister,
            ArchValueRef,
        },
        lumen::{
            LumenAspect,
            LumenInstruction,
            LumenOp,
            LumenParseDiagnostic,
            LumenProgram,
            LumenProgramTemplate,
            LumenTarget,
        },
    },
    state::{
        ArchEditorButton,
        ArchEditorButtonAction,
        ArchEditorState,
        ComputerProgramButton,
        EditorMode,
        EditorSessionState,
        EditorShip,
        EnemyEditorState,
        ProgramButtonAction,
        ProgramEditorAction,
        ProgramEditorActionButton,
        ProgramEditorTextBox,
        ProgramTextEditorState,
        ProgrammingLanguageMode,
        TextBoxClipboard,
    },
};

pub(crate) fn sync_program_text_editor_state(
    editor_ship: Res<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    mut program_editor_state: ResMut<ProgramTextEditorState>,
) {
    let Some(module_id) = arch_editor_state.selected_module_id else {
        if program_editor_state.module_id.is_some() {
            *program_editor_state = ProgramTextEditorState::default();
        }
        return;
    };
    let Some(module) = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.id == module_id && module.kind == ModuleKind::Computer)
    else {
        return;
    };

    let desired_source = match arch_editor_state.selected_language {
        ProgrammingLanguageMode::Arch => module
            .arch_program
            .as_ref()
            .map(|program| program.source_text.clone())
            .unwrap_or_else(|| {
                ArchProgram::from_template(ArchProgramTemplate::BalancedOps).source_text
            }),
        ProgrammingLanguageMode::Lumen => module
            .lumen_program
            .as_ref()
            .map(|program| program.source_text.clone())
            .unwrap_or_else(|| {
                LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision).source_text
            }),
    };

    let switched_target = program_editor_state.module_id != Some(module_id)
        || program_editor_state.language != arch_editor_state.selected_language;
    let external_update =
        !program_editor_state.dirty && program_editor_state.draft_text != desired_source;

    if switched_target || external_update {
        program_editor_state.module_id = Some(module_id);
        program_editor_state.language = arch_editor_state.selected_language;
        program_editor_state.draft_text = desired_source;
        program_editor_state.cursor_index = program_editor_state.draft_text.chars().count();
        program_editor_state.select_all = false;
        program_editor_state.dirty = false;
        program_editor_state.diagnostics.clear();
        program_editor_state.status_line = "Draft matches committed source".to_string();
    }
}

pub(crate) fn focus_program_text_editor_on_click(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ProgramEditorTextBox>,
            Option<&ProgramEditorActionButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut program_editor_state: ResMut<ProgramTextEditorState>,
) {
    for (interaction, textbox, action_button) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if textbox.is_some() {
            program_editor_state.focused = true;
            if program_editor_state.cursor_index == 0 && !program_editor_state.draft_text.is_empty()
            {
                program_editor_state.cursor_index = program_editor_state.draft_text.chars().count();
            }
        } else if action_button.is_some() {
            program_editor_state.focused = false;
        }
    }
}

pub(crate) fn edit_program_text_editor(
    mut keyboard_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut program_editor_state: ResMut<ProgramTextEditorState>,
    mut clipboard: ResMut<TextBoxClipboard>,
) {
    if !program_editor_state.focused {
        return;
    }

    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    normalize_program_cursor(&mut program_editor_state);

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::ArrowLeft => move_program_cursor_left(&mut program_editor_state),
            Key::ArrowRight => move_program_cursor_right(&mut program_editor_state),
            Key::ArrowUp => move_program_cursor_vertical(&mut program_editor_state, -1),
            Key::ArrowDown => move_program_cursor_vertical(&mut program_editor_state, 1),
            Key::Home => move_program_home(&mut program_editor_state),
            Key::End => move_program_end(&mut program_editor_state),
            Key::Enter => insert_program_text(&mut program_editor_state, "\n"),
            Key::Backspace => backspace_program_text(&mut program_editor_state),
            Key::Delete => delete_program_text(&mut program_editor_state),
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("a") => {
                program_editor_state.cursor_index = program_editor_state.draft_text.chars().count();
                program_editor_state.select_all = true;
            }
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("c") => {
                if program_editor_state.select_all {
                    clipboard.contents = program_editor_state.draft_text.clone();
                }
            }
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("x") => {
                if program_editor_state.select_all {
                    clipboard.contents = program_editor_state.draft_text.clone();
                    program_editor_state.draft_text.clear();
                    program_editor_state.cursor_index = 0;
                    program_editor_state.select_all = false;
                    program_editor_state.dirty = true;
                    program_editor_state.status_line = "Draft edited".to_string();
                }
            }
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("v") => {
                if !clipboard.contents.is_empty() {
                    let contents = clipboard.contents.clone();
                    insert_program_text(&mut program_editor_state, &contents);
                }
            }
            Key::Character(chars) if !ctrl_pressed => {
                let sanitized: String = chars
                    .chars()
                    .filter(|character| {
                        *character == '\t' || (!character.is_control() && character.is_ascii())
                    })
                    .collect();
                if !sanitized.is_empty() {
                    insert_program_text(&mut program_editor_state, &sanitized);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn program_editor_action_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &ProgramEditorActionButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_ship: ResMut<EditorShip>,
    mut program_editor_state: ResMut<ProgramTextEditorState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                match button.action {
                    ProgramEditorAction::SwitchArch => {
                        arch_editor_state.selected_language = ProgrammingLanguageMode::Arch;
                    }
                    ProgramEditorAction::SwitchLumen => {
                        arch_editor_state.selected_language = ProgrammingLanguageMode::Lumen;
                    }
                    ProgramEditorAction::Check => {
                        validate_program_draft(&mut program_editor_state);
                    }
                    ProgramEditorAction::Revert => {
                        revert_program_draft(&editor_ship, &mut program_editor_state);
                    }
                    ProgramEditorAction::Apply => {
                        if apply_program_draft(
                            &mut editor_ship,
                            &mut program_editor_state,
                            editor_session.mode,
                            &mut rollback_state,
                            &mut enemy_editor_state,
                        ) {
                            program_editor_state.focused = false;
                        }
                    }
                    ProgramEditorAction::Focus => {
                        program_editor_state.focused = true;
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn computer_program_button_system(
    mut interaction_query: Query<
        (&Interaction, &ComputerProgramButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut arch_editor_state: ResMut<ArchEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
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
                match button.action {
                    ProgramButtonAction::SwitchLanguage(language) => {
                        arch_editor_state.selected_language = language;
                        arch_editor_state.selected_module_id = Some(button.module_id);
                        arch_editor_state.selected_line = 0;
                    }
                    ProgramButtonAction::CycleArchTemplate => {
                        let program = module.arch_program.get_or_insert_with(|| {
                            ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                        });
                        *program = ArchProgram::from_template(program.template.next());
                    }
                    ProgramButtonAction::AdjustArchConstant { index, delta } => {
                        let program = module.arch_program.get_or_insert_with(|| {
                            ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                        });
                        if let Some(constant) = program.constants.get_mut(index) {
                            *constant = (*constant + delta).clamp(0, 20);
                        }
                    }
                    ProgramButtonAction::CycleLumenTemplate => {
                        let lumen_program = module.lumen_program.get_or_insert_with(|| {
                            LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)
                        });
                        *lumen_program = LumenProgram::from_template(lumen_program.template.next());
                    }
                    ProgramButtonAction::ToggleLumenEnabled => {
                        let lumen_program = module.lumen_program.get_or_insert_with(|| {
                            LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)
                        });
                        lumen_program.enabled = !lumen_program.enabled;
                    }
                }
                if editor_session.mode == EditorMode::Player {
                    rollback_state.editor_ship = editor_ship.ship.clone();
                } else {
                    enemy_editor_state.dirty = true;
                }
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
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                apply_arch_editor_action(&mut arch_editor_state, &mut editor_ship, button.action);
                if editor_session.mode == EditorMode::Player {
                    rollback_state.editor_ship = editor_ship.ship.clone();
                } else {
                    enemy_editor_state.dirty = true;
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
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
            arch_editor_state.selected_language = ProgrammingLanguageMode::Arch;
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = line;
        }
        ArchEditorButtonAction::SelectLumenLine { module_id, line } => {
            arch_editor_state.selected_language = ProgrammingLanguageMode::Lumen;
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = line;
        }
        ArchEditorButtonAction::AddLine(module_id) => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            program.instructions.push(ArchInstruction::Nop);
            program.refresh_source_text();
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = program.instructions.len().saturating_sub(1);
        }
        ArchEditorButtonAction::InsertLineAfter { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            let insert_at = (line + 1).min(program.instructions.len());
            program.instructions.insert(insert_at, ArchInstruction::Nop);
            program.refresh_source_text();
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = insert_at;
        }
        ArchEditorButtonAction::RemoveLine { module_id, line } => {
            let Some(program) = module_program_mut(editor_ship, module_id) else {
                return;
            };
            if program.instructions.len() > 1 && line < program.instructions.len() {
                program.instructions.remove(line);
                program.refresh_source_text();
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
                program.refresh_source_text();
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
                program.refresh_source_text();
                arch_editor_state.selected_module_id = Some(module_id);
                arch_editor_state.selected_line = line + 1;
            }
        }
        ArchEditorButtonAction::CycleOpcode { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, cycle_instruction_opcode);
        }
        ArchEditorButtonAction::CycleDst { module_id, line } => {
            mutate_instruction(editor_ship, module_id, line, cycle_dst_register);
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
        ArchEditorButtonAction::AddLumenLine(module_id) => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            program.instructions.push(default_lumen_instruction());
            program.refresh_source_text();
            arch_editor_state.selected_language = ProgrammingLanguageMode::Lumen;
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = program.instructions.len().saturating_sub(1);
        }
        ArchEditorButtonAction::InsertLumenLineAfter { module_id, line } => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            let insert_at = (line + 1).min(program.instructions.len());
            program
                .instructions
                .insert(insert_at, default_lumen_instruction());
            program.refresh_source_text();
            arch_editor_state.selected_language = ProgrammingLanguageMode::Lumen;
            arch_editor_state.selected_module_id = Some(module_id);
            arch_editor_state.selected_line = insert_at;
        }
        ArchEditorButtonAction::RemoveLumenLine { module_id, line } => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            if program.instructions.len() > 1 && line < program.instructions.len() {
                program.instructions.remove(line);
                program.refresh_source_text();
                arch_editor_state.selected_line = arch_editor_state
                    .selected_line
                    .min(program.instructions.len().saturating_sub(1));
            }
        }
        ArchEditorButtonAction::MoveLumenLineUp { module_id, line } => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            if line > 0 && line < program.instructions.len() {
                program.instructions.swap(line, line - 1);
                program.refresh_source_text();
                arch_editor_state.selected_line = line - 1;
            }
        }
        ArchEditorButtonAction::MoveLumenLineDown { module_id, line } => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            if line + 1 < program.instructions.len() {
                program.instructions.swap(line, line + 1);
                program.refresh_source_text();
                arch_editor_state.selected_line = line + 1;
            }
        }
        ArchEditorButtonAction::CycleLumenOp { module_id, line } => {
            mutate_lumen_instruction(editor_ship, module_id, line, |instruction| {
                instruction.op = instruction.op.next();
            });
        }
        ArchEditorButtonAction::CycleLumenTarget { module_id, line } => {
            mutate_lumen_instruction(editor_ship, module_id, line, |instruction| {
                instruction.target = instruction.target.next();
            });
        }
        ArchEditorButtonAction::CycleLumenAspect { module_id, line } => {
            mutate_lumen_instruction(editor_ship, module_id, line, |instruction| {
                instruction.aspect = instruction.aspect.next();
            });
        }
        ArchEditorButtonAction::AdjustLumenWeight {
            module_id,
            line,
            delta,
        } => {
            mutate_lumen_instruction(editor_ship, module_id, line, |instruction| {
                instruction.weight = ((instruction.weight as i32 + delta).clamp(0, 9)) as u8;
            });
        }
        ArchEditorButtonAction::RenameLumenProgram(module_id) => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            program.name = format!("LUMEN-{}", module_id);
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

fn current_program_source(
    editor_ship: &EditorShip,
    module_id: u64,
    language: ProgrammingLanguageMode,
) -> Option<String> {
    let module = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.id == module_id && module.kind == ModuleKind::Computer)?;
    Some(match language {
        ProgrammingLanguageMode::Arch => module
            .arch_program
            .as_ref()
            .map(|program| program.source_text.clone())
            .unwrap_or_else(|| {
                ArchProgram::from_template(ArchProgramTemplate::BalancedOps).source_text
            }),
        ProgrammingLanguageMode::Lumen => module
            .lumen_program
            .as_ref()
            .map(|program| program.source_text.clone())
            .unwrap_or_else(|| {
                LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision).source_text
            }),
    })
}

fn module_lumen_program_mut(
    editor_ship: &mut EditorShip,
    module_id: u64,
) -> Option<&mut LumenProgram> {
    editor_ship
        .ship
        .modules
        .iter_mut()
        .find(|module| module.id == module_id)
        .and_then(|module| {
            if module.kind != ModuleKind::Computer {
                return None;
            }
            Some(module.lumen_program.get_or_insert_with(|| {
                LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)
            }))
        })
}

fn normalize_program_cursor(state: &mut ProgramTextEditorState) {
    state.cursor_index = state.cursor_index.min(state.draft_text.chars().count());
}

fn move_program_cursor_left(state: &mut ProgramTextEditorState) {
    if state.select_all {
        state.cursor_index = 0;
        state.select_all = false;
    } else {
        state.cursor_index = state.cursor_index.saturating_sub(1);
    }
}

fn move_program_cursor_right(state: &mut ProgramTextEditorState) {
    if state.select_all {
        state.cursor_index = state.draft_text.chars().count();
        state.select_all = false;
    } else {
        state.cursor_index = (state.cursor_index + 1).min(state.draft_text.chars().count());
    }
}

fn move_program_home(state: &mut ProgramTextEditorState) {
    let (line_start, _) = current_line_bounds(&state.draft_text, state.cursor_index);
    state.cursor_index = line_start;
    state.select_all = false;
}

fn move_program_end(state: &mut ProgramTextEditorState) {
    let (_, line_end) = current_line_bounds(&state.draft_text, state.cursor_index);
    state.cursor_index = line_end;
    state.select_all = false;
}

fn move_program_cursor_vertical(state: &mut ProgramTextEditorState, direction: i32) {
    let lines: Vec<&str> = state.draft_text.split('\n').collect();
    let (line_index, column) = line_column_for_cursor(&state.draft_text, state.cursor_index);
    let next_line =
        ((line_index as i32 + direction).clamp(0, lines.len().saturating_sub(1) as i32)) as usize;
    let next_column = column.min(lines[next_line].chars().count());
    state.cursor_index = cursor_index_from_line_column(&lines, next_line, next_column);
    state.select_all = false;
}

fn backspace_program_text(state: &mut ProgramTextEditorState) {
    if state.select_all {
        state.draft_text.clear();
        state.cursor_index = 0;
        state.select_all = false;
        state.dirty = true;
        state.status_line = "Draft edited".to_string();
        return;
    }
    if state.cursor_index == 0 {
        return;
    }
    let start = char_to_byte_index(&state.draft_text, state.cursor_index - 1);
    let end = char_to_byte_index(&state.draft_text, state.cursor_index);
    state.draft_text.replace_range(start..end, "");
    state.cursor_index -= 1;
    state.dirty = true;
    state.status_line = "Draft edited".to_string();
}

fn delete_program_text(state: &mut ProgramTextEditorState) {
    if state.select_all {
        state.draft_text.clear();
        state.cursor_index = 0;
        state.select_all = false;
        state.dirty = true;
        state.status_line = "Draft edited".to_string();
        return;
    }
    let len = state.draft_text.chars().count();
    if state.cursor_index >= len {
        return;
    }
    let start = char_to_byte_index(&state.draft_text, state.cursor_index);
    let end = char_to_byte_index(&state.draft_text, state.cursor_index + 1);
    state.draft_text.replace_range(start..end, "");
    state.dirty = true;
    state.status_line = "Draft edited".to_string();
}

fn insert_program_text(state: &mut ProgramTextEditorState, inserted_text: &str) {
    let mut sanitized = inserted_text.replace('\r', "");
    if state.select_all {
        state.draft_text.clear();
        state.cursor_index = 0;
        state.select_all = false;
    }
    let current_lines = state.draft_text.lines().count().max(1);
    let inserted_lines = sanitized.matches('\n').count();
    if current_lines + inserted_lines > 16 {
        let allowed_newlines = 16usize.saturating_sub(current_lines);
        let mut kept_newlines = 0usize;
        sanitized = sanitized
            .chars()
            .filter(|character| {
                if *character != '\n' {
                    return true;
                }
                if kept_newlines < allowed_newlines {
                    kept_newlines += 1;
                    true
                } else {
                    false
                }
            })
            .collect();
    }
    let cursor = state.cursor_index.min(state.draft_text.chars().count());
    let byte_index = char_to_byte_index(&state.draft_text, cursor);
    state.draft_text.insert_str(byte_index, &sanitized);
    state.cursor_index = cursor + sanitized.chars().count();
    state.dirty = true;
    state.status_line = "Draft edited".to_string();
}

fn validate_program_draft(state: &mut ProgramTextEditorState) {
    state.diagnostics = match state.language {
        ProgrammingLanguageMode::Arch => ArchProgram::validate_source_text(&state.draft_text)
            .map(|instructions| {
                state.status_line = format!("ARCH valid: {} instructions", instructions.len());
                Vec::new()
            })
            .unwrap_or_else(|diagnostics| {
                state.status_line = format!("ARCH invalid: {} issue(s)", diagnostics.len());
                diagnostics_to_strings(&diagnostics)
            }),
        ProgrammingLanguageMode::Lumen => LumenProgram::validate_source_text(&state.draft_text)
            .map(|instructions| {
                state.status_line = format!("LUMEN valid: {} directives", instructions.len());
                Vec::new()
            })
            .unwrap_or_else(|diagnostics| {
                state.status_line = format!("LUMEN invalid: {} issue(s)", diagnostics.len());
                lumen_diagnostics_to_strings(&diagnostics)
            }),
    };
}

fn revert_program_draft(editor_ship: &EditorShip, state: &mut ProgramTextEditorState) {
    let Some(module_id) = state.module_id else {
        return;
    };
    if let Some(source) = current_program_source(editor_ship, module_id, state.language) {
        state.draft_text = source;
        state.cursor_index = state.draft_text.chars().count();
        state.select_all = false;
        state.dirty = false;
        state.diagnostics.clear();
        state.status_line = "Reverted to committed source".to_string();
    }
}

fn apply_program_draft(
    editor_ship: &mut EditorShip,
    state: &mut ProgramTextEditorState,
    editor_mode: EditorMode,
    rollback_state: &mut netcode::RollbackGameState,
    enemy_editor_state: &mut EnemyEditorState,
) -> bool {
    let Some(module_id) = state.module_id else {
        return false;
    };
    let Some(module) = editor_ship
        .ship
        .modules
        .iter_mut()
        .find(|module| module.id == module_id && module.kind == ModuleKind::Computer)
    else {
        return false;
    };

    let apply_result = match state.language {
        ProgrammingLanguageMode::Arch => {
            let program = module.arch_program.get_or_insert_with(|| {
                ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
            });
            program
                .compile_source_text(&state.draft_text)
                .map_err(|diagnostics| {
                    state.diagnostics = diagnostics_to_strings(&diagnostics);
                    state.status_line =
                        format!("ARCH apply failed: {} issue(s)", state.diagnostics.len());
                })
        }
        ProgrammingLanguageMode::Lumen => {
            let program = module.lumen_program.get_or_insert_with(|| {
                LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)
            });
            program
                .compile_source_text(&state.draft_text)
                .map_err(|diagnostics| {
                    state.diagnostics = lumen_diagnostics_to_strings(&diagnostics);
                    state.status_line =
                        format!("LUMEN apply failed: {} issue(s)", state.diagnostics.len());
                })
        }
    };

    if apply_result.is_err() {
        return false;
    }

    state.dirty = false;
    state.diagnostics.clear();
    state.status_line = "Committed program source".to_string();
    if editor_mode == EditorMode::Player {
        rollback_state.editor_ship = editor_ship.ship.clone();
    } else {
        enemy_editor_state.dirty = true;
    }
    true
}

fn diagnostics_to_strings(diagnostics: &[ArchParseDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| format!("L{:02}: {}", diagnostic.line + 1, diagnostic.message))
        .collect()
}

fn lumen_diagnostics_to_strings(diagnostics: &[LumenParseDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| format!("L{:02}: {}", diagnostic.line + 1, diagnostic.message))
        .collect()
}

fn current_line_bounds(text: &str, cursor_index: usize) -> (usize, usize) {
    let lines: Vec<&str> = text.split('\n').collect();
    let (line_index, _) = line_column_for_cursor(text, cursor_index);
    let line_start = cursor_index_from_line_column(&lines, line_index, 0);
    let line_end =
        cursor_index_from_line_column(&lines, line_index, lines[line_index].chars().count());
    (line_start, line_end)
}

fn line_column_for_cursor(text: &str, cursor_index: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut column = 0usize;
    for (index, character) in text.chars().enumerate() {
        if index == cursor_index {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn cursor_index_from_line_column(
    lines: &[&str],
    target_line: usize,
    target_column: usize,
) -> usize {
    let mut index = 0usize;
    for (line_index, line) in lines.iter().enumerate() {
        if line_index == target_line {
            return index + target_column.min(line.chars().count());
        }
        index += line.chars().count() + 1;
    }
    index
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
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
    program.refresh_source_text();
}

fn mutate_lumen_instruction(
    editor_ship: &mut EditorShip,
    module_id: u64,
    line: usize,
    mutate: impl FnOnce(&mut LumenInstruction),
) {
    let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
        return;
    };
    let Some(instruction) = program.instructions.get_mut(line) else {
        return;
    };
    mutate(instruction);
    program.refresh_source_text();
}

fn default_lumen_instruction() -> LumenInstruction {
    LumenInstruction {
        op: LumenOp::Buff,
        target: LumenTarget::Reactors,
        aspect: LumenAspect::HeatCooling,
        weight: 1,
    }
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
