use bevy::prelude::*;

use crate::{
    ship::{
        ModuleKind,
        arch::{ArchProgram, ArchProgramTemplate},
        lumen::{LumenProgram, LumenProgramTemplate},
        storage::save_default_ship,
    },
    state::{
        ArchEditorButton,
        ArchEditorButtonAction,
        ArchEditorState,
        ComputerProgramButton,
        ComputerProgramEntry,
        ComputerProgramPanel,
        EditorShip,
        ProgramButtonAction,
        ProgramEditorAction,
        ProgramEditorActionButton,
        ProgramEditorDiagnosticsText,
        ProgramEditorDraftText,
        ProgramEditorStatusText,
        ProgramEditorTextBox,
        ProgramTextEditorState,
        ProgrammingLanguageMode,
    },
};

pub(crate) fn sync_computer_program_entries(
    mut commands: Commands,
    editor_ship: Res<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    program_editor_state: Res<ProgramTextEditorState>,
    asset_server: Res<AssetServer>,
    panel_query: Single<Entity, With<ComputerProgramPanel>>,
    existing_query: Query<Entity, With<ComputerProgramEntry>>,
) {
    if !editor_ship.is_changed()
        && !arch_editor_state.is_changed()
        && !program_editor_state.is_changed()
    {
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
        let draft_text = if program_editor_state.module_id == Some(selected_module.id)
            && program_editor_state.language == arch_editor_state.selected_language
        {
            program_editor_state.draft_text.clone()
        } else {
            match arch_editor_state.selected_language {
                ProgrammingLanguageMode::Arch => program.source_text.clone(),
                ProgrammingLanguageMode::Lumen => lumen_program.source_text.clone(),
            }
        };
        let diagnostics_text = if program_editor_state.diagnostics.is_empty() {
            "No parse diagnostics".to_string()
        } else {
            program_editor_state.diagnostics.join("\n")
        };

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
                    Text::new(
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => format!(
                                "Computer #{}\nProgram: {}\nTemplate: {}\nChannel: {}\nValidation: {}",
                                selected_module.id,
                                program.name,
                                program.template.as_str(),
                                selected_module.effective_channel(),
                                program_validation_summary(&program),
                            ),
                            ProgrammingLanguageMode::Lumen => format!(
                                "Computer #{}\nProgram: {}\nTemplate: {}\nChannel: {}\nState: {}\nValidation: {}",
                                selected_module.id,
                                lumen_program.name,
                                lumen_program.template.as_str(),
                                selected_module.effective_channel(),
                                if lumen_program.enabled { "online" } else { "disabled" },
                                lumen_validation_summary(&lumen_program),
                            ),
                        }.to_string()
                    ),
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
                            }
                        }
                        for (label, action) in [
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
                                    ComputerProgramEntry,
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

                entry.spawn((
                    Text::new(match arch_editor_state.selected_language {
                        ProgrammingLanguageMode::Arch => {
                            "ARCH Source\nClick to focus. Enter newline. Ctrl-A/C/X/V supported."
                        }
                        ProgrammingLanguageMode::Lumen => {
                            "LUMEN Source\nClick to focus. Enter newline. Ctrl-A/C/X/V supported."
                        }
                    }),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.76, 0.80, 0.86)),
                    ComputerProgramEntry,
                ));

                entry
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(220.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if program_editor_state.focused {
                            Color::srgba(0.18, 0.24, 0.34, 0.98)
                        } else {
                            Color::srgba(0.12, 0.16, 0.24, 0.94)
                        }),
                        ProgramEditorTextBox,
                        ComputerProgramEntry,
                    ))
                    .with_children(|textbox| {
                        textbox.spawn((
                            Text::new(format_program_draft_text(
                                &draft_text,
                                &program_editor_state,
                                selected_module.id,
                                arch_editor_state.selected_language,
                            )),
                            TextFont {
                                font: mono_font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            ProgramEditorDraftText,
                        ));
                    });

                entry.spawn((
                    Text::new(if program_editor_state.status_line.is_empty() {
                        "Draft idle".to_string()
                    } else {
                        program_editor_state.status_line.clone()
                    }),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.84, 0.62)),
                    ProgramEditorStatusText,
                    ComputerProgramEntry,
                ));

                entry.spawn((
                    Text::new(format!("Diagnostics\n{diagnostics_text}")),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(if program_editor_state.diagnostics.is_empty() {
                        Color::srgb(0.76, 0.80, 0.86)
                    } else {
                        Color::srgb(0.96, 0.58, 0.46)
                    }),
                    ProgramEditorDiagnosticsText,
                    ComputerProgramEntry,
                ));
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

fn format_program_draft_text(
    draft_text: &str,
    program_editor_state: &ProgramTextEditorState,
    module_id: u64,
    language: ProgrammingLanguageMode,
) -> String {
    if program_editor_state.module_id != Some(module_id)
        || program_editor_state.language != language
        || !program_editor_state.focused
    {
        return draft_text.to_string();
    }

    let mut display = if program_editor_state.select_all {
        format!("[{draft_text}]")
    } else {
        draft_text.to_string()
    };
    let cursor_index = program_editor_state
        .cursor_index
        .min(draft_text.chars().count());
    let insert_at = char_to_byte_index(
        &display,
        if program_editor_state.select_all {
            display.chars().count()
        } else {
            cursor_index
        },
    );
    display.insert(insert_at, '|');
    display
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[allow(dead_code)]
pub(crate) fn persist_now(editor_ship: &EditorShip) {
    if let Err(error) = save_default_ship(&editor_ship.ship) {
        eprintln!("editor: failed to save ship: {error}");
    }
}
