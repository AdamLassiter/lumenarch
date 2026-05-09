use bevy::prelude::*;

use crate::{
    gameplay::components::ShipControlMode,
    ship::{
        ModuleKind,
        ModuleSpec,
        ModuleVariant,
        ShipModule,
        enemy::EnemyShipEntryValidationStatus,
    },
    state::{
        EditingCleanup,
        EditorMode,
        EditorSessionState,
        EnemyEditorState,
        EnemyShipLibraryState,
        ProgramTextEditorState,
        ProgrammingLanguageMode,
        StationPanelButtonAction,
    },
};

#[derive(Clone)]
pub(super) struct EditorReadout {
    pub(super) label: String,
    pub(super) value: String,
    pub(super) visual: EditorReadoutVisual,
}

#[derive(Clone, Copy)]
pub(super) enum EditorReadoutVisual {
    Bar { percent: f32, color: Color },
    Light { color: Color },
}

pub(super) fn editor_control_mode(kind: ModuleKind) -> ShipControlMode {
    match kind {
        ModuleKind::Cockpit => ShipControlMode::Cockpit,
        ModuleKind::Turret => ShipControlMode::Turret,
        ModuleKind::Reactor => ShipControlMode::Reactor,
        ModuleKind::Cargo | ModuleKind::Airlock | ModuleKind::Processor => {
            ShipControlMode::Logistics
        }
        ModuleKind::Computer => ShipControlMode::Computer,
        _ => ShipControlMode::Interior,
    }
}

#[derive(Clone, Copy)]
pub(super) struct EditorStationFlags {
    storage: bool,
    manipulator: bool,
    processor: bool,
    airlock: bool,
    drone: bool,
}

pub(super) fn editor_station_flags(kind: ModuleKind) -> EditorStationFlags {
    EditorStationFlags {
        storage: matches!(kind, ModuleKind::Cargo | ModuleKind::Airlock),
        manipulator: kind == ModuleKind::Airlock,
        processor: kind == ModuleKind::Processor,
        airlock: kind == ModuleKind::Airlock,
        drone: kind == ModuleKind::Airlock,
    }
}

pub(super) fn editor_station_action_visible(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    active_kind: ModuleKind,
    flags: EditorStationFlags,
) -> bool {
    match mode {
        ShipControlMode::Cockpit => matches!(
            action,
            StationPanelButtonAction::HelmThrottle { .. }
                | StationPanelButtonAction::HelmTurn { .. }
        ),
        ShipControlMode::Turret => matches!(
            action,
            StationPanelButtonAction::TurretAdjustAim { .. }
                | StationPanelButtonAction::TurretFireToggle
        ),
        ShipControlMode::Reactor => matches!(
            action,
            StationPanelButtonAction::ReactorAdjustRate { .. }
                | StationPanelButtonAction::ReactorAdjustTurbine { .. }
        ),
        ShipControlMode::Logistics => match action {
            StationPanelButtonAction::LogisticsToggleStorageIntake => flags.storage,
            StationPanelButtonAction::LogisticsToggleAirlock => flags.airlock,
            StationPanelButtonAction::LogisticsToggleManipulator
            | StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. }
            | StationPanelButtonAction::LogisticsCycleResource => flags.manipulator,
            StationPanelButtonAction::LogisticsToggleProcessor => {
                flags.processor || flags.drone || active_kind == ModuleKind::Airlock
            }
            _ => false,
        },
        ShipControlMode::Computer => matches!(
            action,
            StationPanelButtonAction::ComputerToggleEnabled
                | StationPanelButtonAction::ComputerCycleTemplate
        ),
        ShipControlMode::Interior => false,
    }
}

pub(super) fn editor_station_button_label(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    flags: EditorStationFlags,
    language: ProgrammingLanguageMode,
) -> String {
    match action {
        StationPanelButtonAction::ComputerCycleTemplate => match language {
            ProgrammingLanguageMode::Arch => "Cycle ARCH Template".to_string(),
            ProgrammingLanguageMode::Lumen => "Cycle LUMEN Template".to_string(),
        },
        StationPanelButtonAction::LogisticsToggleProcessor
            if mode == ShipControlMode::Logistics && flags.drone =>
        {
            "Cycle Drone Mode".to_string()
        }
        StationPanelButtonAction::LogisticsToggleProcessor
            if mode == ShipControlMode::Logistics && !flags.processor =>
        {
            "Cycle Recipe".to_string()
        }
        _ => station_button_default_label(action).to_string(),
    }
}

pub(super) fn editor_station_readouts(module: &ShipModule) -> Vec<EditorReadout> {
    match module.kind {
        ModuleKind::Reactor => vec![
            editor_bar(
                module,
                "RRF",
                "Reaction",
                module.defaults.reaction_rate_milli as f32 / 10.0,
                Color::srgb(0.94, 0.42, 0.24),
            ),
            editor_bar(
                module,
                "RRT",
                "Turbine",
                module.defaults.turbine_load_milli as f32 / 10.0,
                Color::srgb(0.34, 0.74, 0.94),
            ),
            editor_light(
                module,
                "RRS",
                "Default Stability",
                "Nominal",
                Color::srgb(0.34, 0.78, 0.46),
            ),
            editor_light(
                module,
                "RRP",
                "Startup Output",
                "Derived in encounter",
                Color::srgb(0.86, 0.74, 0.30),
            ),
        ],
        ModuleKind::Turret => vec![
            editor_light(
                module,
                "WTF",
                "Fire Gate",
                if module.defaults.turret_fire_intent {
                    "Open"
                } else {
                    "Hold"
                },
                if module.defaults.turret_fire_intent {
                    Color::srgb(0.34, 0.78, 0.46)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            editor_light(
                module,
                "WTR",
                "Default Arc",
                "Forward",
                Color::srgb(0.64, 0.60, 0.96),
            ),
            editor_light(
                module,
                "WTC",
                "Cooldown",
                "Variant derived",
                Color::srgb(0.96, 0.56, 0.24),
            ),
        ],
        ModuleKind::Cargo => vec![
            editor_light(
                module,
                "LSI",
                "Allow Intake",
                bool_word(module.defaults.storage_allow_intake),
                bool_color(module.defaults.storage_allow_intake),
            ),
            editor_light(
                module,
                "LSC",
                "Capacity",
                &ModuleSpec::for_module(module.kind, module.variant)
                    .storage_capacity
                    .to_string(),
                Color::srgb(0.36, 0.72, 0.96),
            ),
        ],
        ModuleKind::Airlock => vec![
            editor_light(
                module,
                "LAC",
                "Airlock",
                if module.defaults.airlock_open {
                    "Open"
                } else {
                    "Closed"
                },
                bool_color(!module.defaults.airlock_open),
            ),
            editor_light(
                module,
                "LSI",
                "Allow Intake",
                bool_word(module.defaults.storage_allow_intake),
                bool_color(module.defaults.storage_allow_intake),
            ),
            editor_light(
                module,
                "LMM",
                "Manual Mode",
                bool_word(module.defaults.manipulator_manual_mode),
                bool_color(module.defaults.manipulator_manual_mode),
            ),
            editor_light(
                module,
                "LME",
                "Transfer",
                bool_word(module.defaults.manipulator_transfer_enabled),
                bool_color(module.defaults.manipulator_transfer_enabled),
            ),
            editor_light(
                module,
                "LMT",
                "Resource",
                module.defaults.manipulator_resource_kind.as_str(),
                Color::srgb(0.86, 0.74, 0.30),
            ),
            editor_light(
                module,
                "DRM",
                "Drone Frame",
                if module.variant == ModuleVariant::DroneBay {
                    "Installed"
                } else {
                    "None"
                },
                if module.variant == ModuleVariant::DroneBay {
                    Color::srgb(0.54, 0.86, 1.0)
                } else {
                    Color::srgb(0.42, 0.46, 0.52)
                },
            ),
        ],
        ModuleKind::Processor => vec![
            editor_light(
                module,
                "LPY",
                "Recipe",
                module.defaults.processor_recipe.as_str(),
                Color::srgb(0.86, 0.74, 0.30),
            ),
            editor_light(
                module,
                "LPS",
                "Enabled",
                bool_word(module.defaults.processor_enabled),
                bool_color(module.defaults.processor_enabled),
            ),
        ],
        ModuleKind::Computer => vec![
            editor_light(
                module,
                "CCA",
                "Enabled",
                bool_word(module.defaults.computer_enabled),
                bool_color(module.defaults.computer_enabled),
            ),
            editor_light(
                module,
                "CCP",
                "ARCH Program",
                module
                    .arch_program
                    .as_ref()
                    .map(|program| program.name.as_str())
                    .unwrap_or("Balanced Ops"),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "CLP",
                "LUMEN Program",
                module
                    .lumen_program
                    .as_ref()
                    .map(|program| program.name.as_str())
                    .unwrap_or("Balanced Supervision"),
                Color::srgb(0.62, 0.90, 0.80),
            ),
        ],
        ModuleKind::Cockpit => vec![
            editor_light(
                module,
                "HLC",
                "Channel",
                &module.effective_channel().to_string(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "HLM",
                "Default Helm",
                "Manual crew control",
                Color::srgb(0.34, 0.78, 0.46),
            ),
        ],
        _ => vec![
            editor_light(
                module,
                "MOD",
                "Variant",
                module.variant.display_name(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            editor_light(
                module,
                "CHN",
                "Channel",
                &module.effective_channel().to_string(),
                Color::srgb(0.62, 0.90, 0.80),
            ),
        ],
    }
}

fn editor_bar(
    module: &ShipModule,
    register: &str,
    label: &str,
    percent: f32,
    color: Color,
) -> EditorReadout {
    EditorReadout {
        label: format!("{register}{}  {label}", module.effective_channel()),
        value: format!("{:.0}%", percent),
        visual: EditorReadoutVisual::Bar {
            percent: percent.clamp(0.0, 100.0),
            color,
        },
    }
}

fn editor_light(
    module: &ShipModule,
    register: &str,
    label: &str,
    value: &str,
    color: Color,
) -> EditorReadout {
    EditorReadout {
        label: if module.kind.supports_channel() {
            format!("{register}{}  {label}", module.effective_channel())
        } else {
            format!("{register}  {label}")
        },
        value: value.to_string(),
        visual: EditorReadoutVisual::Light { color },
    }
}

fn bool_word(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

fn bool_color(value: bool) -> Color {
    if value {
        Color::srgb(0.34, 0.78, 0.46)
    } else {
        Color::srgb(0.84, 0.28, 0.28)
    }
}

pub(super) fn format_program_textbox(
    program_editor_state: &ProgramTextEditorState,
    module_id: u64,
    language: ProgrammingLanguageMode,
    module: &ShipModule,
) -> String {
    let draft_text = if program_editor_state.module_id == Some(module_id)
        && program_editor_state.language == language
    {
        program_editor_state.draft_text.clone()
    } else {
        match language {
            ProgrammingLanguageMode::Arch => module
                .arch_program
                .as_ref()
                .map(|program| program.source_text.clone())
                .unwrap_or_default(),
            ProgrammingLanguageMode::Lumen => module
                .lumen_program
                .as_ref()
                .map(|program| program.source_text.clone())
                .unwrap_or_default(),
        }
    };
    if program_editor_state.module_id != Some(module_id)
        || program_editor_state.language != language
        || !program_editor_state.focused
    {
        return draft_text;
    }
    let mut display = if program_editor_state.select_all {
        format!("[{draft_text}]")
    } else {
        draft_text
    };
    let cursor_index = program_editor_state
        .cursor_index
        .min(display.chars().count());
    let insert_at = display
        .char_indices()
        .nth(cursor_index)
        .map(|(index, _)| index)
        .unwrap_or(display.len());
    display.insert(insert_at, '|');
    display
}

pub(crate) fn cleanup_editor_entities(
    mut commands: Commands,
    query: Query<Entity, With<EditingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(super) fn enemy_entry_label(
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
            Some(EnemyShipEntryValidationStatus::RepairedInMemory) => " [repaired in memory]",
            Some(EnemyShipEntryValidationStatus::Invalid) => " [invalid]",
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

pub(super) fn station_actions() -> [StationPanelButtonAction; 21] {
    [
        StationPanelButtonAction::HelmThrottle { delta: -0.2 },
        StationPanelButtonAction::HelmThrottle { delta: 0.2 },
        StationPanelButtonAction::HelmTurn { value: -1.0 },
        StationPanelButtonAction::HelmTurn { value: 0.0 },
        StationPanelButtonAction::HelmTurn { value: 1.0 },
        StationPanelButtonAction::TurretAdjustAim { delta: 0.25 },
        StationPanelButtonAction::TurretAdjustAim { delta: -0.25 },
        StationPanelButtonAction::TurretFireToggle,
        StationPanelButtonAction::ReactorAdjustRate { delta: -0.1 },
        StationPanelButtonAction::ReactorAdjustRate { delta: 0.1 },
        StationPanelButtonAction::ReactorAdjustTurbine { delta: -0.1 },
        StationPanelButtonAction::ReactorAdjustTurbine { delta: 0.1 },
        StationPanelButtonAction::LogisticsToggleStorageIntake,
        StationPanelButtonAction::LogisticsToggleAirlock,
        StationPanelButtonAction::LogisticsToggleManipulator,
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction: -1 },
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction: 1 },
        StationPanelButtonAction::LogisticsCycleResource,
        StationPanelButtonAction::LogisticsToggleProcessor,
        StationPanelButtonAction::ComputerToggleEnabled,
        StationPanelButtonAction::ComputerCycleTemplate,
    ]
}

pub(super) fn station_button_default_label(action: StationPanelButtonAction) -> &'static str {
    match action {
        StationPanelButtonAction::HelmThrottle { delta } if delta < 0.0 => "Throttle Down",
        StationPanelButtonAction::HelmThrottle { .. } => "Throttle Up",
        StationPanelButtonAction::HelmTurn { value } if value < 0.0 => "Turn Port",
        StationPanelButtonAction::HelmTurn { value } if value > 0.0 => "Turn Starboard",
        StationPanelButtonAction::HelmTurn { .. } => "Steady",
        StationPanelButtonAction::TurretAdjustAim { delta } if delta < 0.0 => "Trim Right",
        StationPanelButtonAction::TurretAdjustAim { .. } => "Trim Left",
        StationPanelButtonAction::TurretFireToggle => "Toggle Fire Gate",
        StationPanelButtonAction::ReactorAdjustRate { delta } if delta < 0.0 => "Reaction -",
        StationPanelButtonAction::ReactorAdjustRate { .. } => "Reaction +",
        StationPanelButtonAction::ReactorAdjustTurbine { delta } if delta < 0.0 => "Turbine -",
        StationPanelButtonAction::ReactorAdjustTurbine { .. } => "Turbine +",
        StationPanelButtonAction::LogisticsToggleStorageIntake => "Toggle Intake",
        StationPanelButtonAction::LogisticsToggleAirlock => "Cycle Airlock",
        StationPanelButtonAction::LogisticsToggleManipulator => "Toggle Manipulator",
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction }
            if direction < 0 =>
        {
            "Prev Target"
        }
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => "Next Target",
        StationPanelButtonAction::LogisticsCycleResource => "Cycle Resource",
        StationPanelButtonAction::LogisticsToggleProcessor => "Toggle Processor",
        StationPanelButtonAction::ComputerToggleEnabled => "Enable / Disable",
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template",
    }
}
