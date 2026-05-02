use bevy::prelude::*;

use crate::{
    ship::{
        arch::{ArchProgram, ArchProgramTemplate},
        lumen::{LumenProgram, LumenProgramTemplate},
        storage::save_default_ship,
        ModuleKind,
    },
    state::{
        ArchEditorButton, ArchEditorButtonAction, ArchEditorState, ComputerProgramButton,
        ComputerProgramEntry, ComputerProgramPanel, EditorShip, ProgramButtonAction,
        ProgrammingLanguageMode,
    },
};

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
            Text::new(match arch_editor_state.selected_language {
                ProgrammingLanguageMode::Arch => "ARCH Workshop",
                ProgrammingLanguageMode::Lumen => "LUMEN Workshop",
            }),
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
            Text::new("Select a computer, then edit either the direct ARCH program or the optimization-layer LUMEN program. ARCH writes explicit command registers; LUMEN applies bounded BUFF / NERF directives to target groups."),
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
        let lumen_program = selected_module
            .lumen_program
            .clone()
            .unwrap_or_else(|| LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision));

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
                entry
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
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            if arch_editor_state.selected_language == ProgrammingLanguageMode::Arch {
                                "ARCH Active"
                            } else {
                                "Open ARCH"
                            },
                            selected_module.id,
                            ProgramButtonAction::SwitchLanguage(ProgrammingLanguageMode::Arch),
                        );
                        spawn_program_button(
                            buttons,
                            &mono_font,
                            if arch_editor_state.selected_language == ProgrammingLanguageMode::Lumen {
                                "LUMEN Active"
                            } else {
                                "Open LUMEN"
                            },
                            selected_module.id,
                            ProgramButtonAction::SwitchLanguage(ProgrammingLanguageMode::Lumen),
                        );
                    });
                entry.spawn((
                    Text::new(format!(
                        "{}",
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => format!(
                                "Computer #{}\nProgram: {}\nTemplate: {}\nConst A / B: {} / {}\nValidation: {}",
                                selected_module.id,
                                program.name,
                                program.template.as_str(),
                                program.constants[0],
                                program.constants[1],
                                program_validation_summary(&program),
                            ),
                            ProgrammingLanguageMode::Lumen => format!(
                                "Computer #{}\nProgram: {}\nTemplate: {}\nState: {}\nValidation: {}",
                                selected_module.id,
                                lumen_program.name,
                                lumen_program.template.as_str(),
                                if lumen_program.enabled { "online" } else { "disabled" },
                                lumen_validation_summary(&lumen_program),
                            ),
                        }
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
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => {
                                spawn_program_button(
                                    buttons,
                                    &mono_font,
                                    "Cycle Template",
                                    selected_module.id,
                                    ProgramButtonAction::CycleArchTemplate,
                                );
                                for (label, index, delta) in [
                                    ("Const A +1", 0, 1),
                                    ("Const A -1", 0, -1),
                                    ("Const B +1", 1, 1),
                                    ("Const B -1", 1, -1),
                                ] {
                                    spawn_program_button(
                                        buttons,
                                        &mono_font,
                                        label,
                                        selected_module.id,
                                        ProgramButtonAction::AdjustArchConstant { index, delta },
                                    );
                                }
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Add Line",
                                    ArchEditorButtonAction::AddLine(selected_module.id),
                                );
                            }
                            ProgrammingLanguageMode::Lumen => {
                                spawn_program_button(
                                    buttons,
                                    &mono_font,
                                    "Cycle Template",
                                    selected_module.id,
                                    ProgramButtonAction::CycleLumenTemplate,
                                );
                                spawn_program_button(
                                    buttons,
                                    &mono_font,
                                    if lumen_program.enabled {
                                        "Disable Program"
                                    } else {
                                        "Enable Program"
                                    },
                                    selected_module.id,
                                    ProgramButtonAction::ToggleLumenEnabled,
                                );
                                spawn_arch_editor_button(
                                    buttons,
                                    &mono_font,
                                    "Add Directive",
                                    ArchEditorButtonAction::AddLumenLine(selected_module.id),
                                );
                            }
                        }
                    });

                let line_count = match arch_editor_state.selected_language {
                    ProgrammingLanguageMode::Arch => program.instructions.len(),
                    ProgrammingLanguageMode::Lumen => lumen_program.instructions.len(),
                };
                for line_index in 0..line_count {
                    let selected = arch_editor_state.selected_line == line_index;
                    let title = match arch_editor_state.selected_language {
                        ProgrammingLanguageMode::Arch => format!(
                            "L{:02}  {}",
                            line_index,
                            instruction_summary(&program.instructions[line_index])
                        ),
                        ProgrammingLanguageMode::Lumen => format!(
                            "L{:02}  {}",
                            line_index,
                            lumen_instruction_summary(&lumen_program.instructions[line_index])
                        ),
                    };
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
                                Text::new(title),
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
                                let actions = match arch_editor_state.selected_language {
                                    ProgrammingLanguageMode::Arch => vec![
                                        (
                                            if selected { "Selected" } else { "Select" },
                                            ArchEditorButtonAction::SelectLine {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Opcode",
                                            ArchEditorButtonAction::CycleOpcode {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Dst",
                                            ArchEditorButtonAction::CycleDst {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Src A",
                                            ArchEditorButtonAction::CycleSrcA {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Src B",
                                            ArchEditorButtonAction::CycleSrcB {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Imm A +",
                                            ArchEditorButtonAction::AdjustImmediateA {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: 1,
                                            },
                                        ),
                                        (
                                            "Imm A -",
                                            ArchEditorButtonAction::AdjustImmediateA {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: -1,
                                            },
                                        ),
                                        (
                                            "Imm B +",
                                            ArchEditorButtonAction::AdjustImmediateB {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: 1,
                                            },
                                        ),
                                        (
                                            "Imm B -",
                                            ArchEditorButtonAction::AdjustImmediateB {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: -1,
                                            },
                                        ),
                                        (
                                            "Jump +",
                                            ArchEditorButtonAction::AdjustJump {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: 1,
                                            },
                                        ),
                                        (
                                            "Jump -",
                                            ArchEditorButtonAction::AdjustJump {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: -1,
                                            },
                                        ),
                                        (
                                            "Insert",
                                            ArchEditorButtonAction::InsertLineAfter {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Up",
                                            ArchEditorButtonAction::MoveLineUp {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Down",
                                            ArchEditorButtonAction::MoveLineDown {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Remove",
                                            ArchEditorButtonAction::RemoveLine {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                    ],
                                    ProgrammingLanguageMode::Lumen => vec![
                                        (
                                            if selected { "Selected" } else { "Select" },
                                            ArchEditorButtonAction::SelectLumenLine {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "BUFF/NERF",
                                            ArchEditorButtonAction::CycleLumenOp {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Target",
                                            ArchEditorButtonAction::CycleLumenTarget {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Aspect",
                                            ArchEditorButtonAction::CycleLumenAspect {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Weight +",
                                            ArchEditorButtonAction::AdjustLumenWeight {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: 1,
                                            },
                                        ),
                                        (
                                            "Weight -",
                                            ArchEditorButtonAction::AdjustLumenWeight {
                                                module_id: selected_module.id,
                                                line: line_index,
                                                delta: -1,
                                            },
                                        ),
                                        (
                                            "Insert",
                                            ArchEditorButtonAction::InsertLumenLineAfter {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Up",
                                            ArchEditorButtonAction::MoveLumenLineUp {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Down",
                                            ArchEditorButtonAction::MoveLumenLineDown {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                        (
                                            "Remove",
                                            ArchEditorButtonAction::RemoveLumenLine {
                                                module_id: selected_module.id,
                                                line: line_index,
                                            },
                                        ),
                                    ],
                                };
                                for (label, action) in actions {
                                    spawn_arch_editor_button(buttons, &mono_font, label, action);
                                }
                            });
                        });
                }
            });
    });
}

fn lumen_instruction_summary(instruction: &crate::ship::lumen::LumenInstruction) -> String {
    instruction.syntax()
}

fn lumen_validation_summary(program: &LumenProgram) -> String {
    if program.instructions.is_empty() {
        "no directives".to_string()
    } else if !program.enabled {
        "disabled".to_string()
    } else {
        format!("{} directives ready", program.instructions.len())
    }
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
            format!("ADD {} <- {} + {}", dst.as_str(), value_ref(lhs), value_ref(rhs))
        }
        ArchInstruction::Sub { dst, lhs, rhs } => {
            format!("SUB {} <- {} - {}", dst.as_str(), value_ref(lhs), value_ref(rhs))
        }
        ArchInstruction::Gt { dst, lhs, rhs } => {
            format!("GT {} <- {} > {}", dst.as_str(), value_ref(lhs), value_ref(rhs))
        }
        ArchInstruction::Lt { dst, lhs, rhs } => {
            format!("LT {} <- {} < {}", dst.as_str(), value_ref(lhs), value_ref(rhs))
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
