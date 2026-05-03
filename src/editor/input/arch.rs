use bevy::prelude::*;

use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    ship::{
        ModuleKind,
        arch::{ArchInstruction, ArchProgram, ArchProgramTemplate, ArchRegister, ArchValueRef},
        lumen::{
            LumenAspect,
            LumenInstruction,
            LumenOp,
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
        ProgrammingLanguageMode,
    },
};

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
                arch_editor_state.selected_line = line - 1;
            }
        }
        ArchEditorButtonAction::MoveLumenLineDown { module_id, line } => {
            let Some(program) = module_lumen_program_mut(editor_ship, module_id) else {
                return;
            };
            if line + 1 < program.instructions.len() {
                program.instructions.swap(line, line + 1);
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
