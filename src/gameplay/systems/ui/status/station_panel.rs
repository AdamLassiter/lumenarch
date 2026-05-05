use bevy::log;

use super::*;

mod actions;
mod readouts;

use actions::pending_station_command;
pub(crate) use actions::{station_action_visible, station_button_label};
use readouts::*;

#[derive(Clone)]
pub(crate) struct StationPanelReadout {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) visual: StationReadoutVisual,
}

#[derive(Clone, Copy)]
pub(crate) enum StationReadoutVisual {
    Bar { percent: f32, color: Color },
    Light { color: Color },
}

pub(crate) struct StationPanelDisplay {
    pub(crate) title: String,
    pub(crate) active_station_kind: Option<crate::ship::ModuleKind>,
    pub(crate) flags: summary::StationFlags,
    pub(crate) readouts: Vec<StationPanelReadout>,
}

impl StationPanelDisplay {
    fn empty(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            active_station_kind: None,
            flags: summary::StationFlags::default(),
            readouts: Vec::new(),
        }
    }
}

pub(crate) fn station_panel_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &GameplayStationPanelButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    player_query: Single<&ShipboardControlState, With<ObservedLocalPlayerMarker>>,
    mut pending_station: ResMut<netcode::PendingLocalStationCommand>,
) {
    let control_state = player_query.into_inner();

    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.20, 0.31, 0.48));
                let Some(command) = pending_station_command(button.action, control_state) else {
                    log::debug!(
                        "Ignoring station panel action {:?} because the local player is not in a matching control state",
                        button.action
                    );
                    continue;
                };
                pending_station.0 = Some(command);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.30, 0.46, 0.68));
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn station_panel_display(
    control_mode: &ShipboardControlState,
    mission_state: &MissionState,
    weapon_state: &ShipWeaponState,
    _player_fields: &PlayerFieldState,
    module_query: &Query<
        (
            Entity,
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    drone_query: &Query<
        (
            Option<&DroneStationModule>,
            Option<&DroneStationCommandState>,
        ),
        With<RuntimeShipModule>,
    >,
    focused_station_context: &str,
    arch_summary: &summary::ArchSummary,
) -> StationPanelDisplay {
    let Some(focused_entity) = control_mode.focused_entity else {
        return StationPanelDisplay::empty("Station Console");
    };

    let Ok((
        _,
        runtime_module,
        integrity,
        runtime_state,
        computer,
        storage,
        storage_command,
        manipulator,
        manipulator_command,
        processor,
        processor_command,
        reactor,
        turret,
        airlock_state,
        destroyed,
    )) = module_query.get(focused_entity)
    else {
        return StationPanelDisplay::empty("Station Console");
    };
    let (drone_station, drone_station_command) =
        drone_query.get(focused_entity).ok().unwrap_or((None, None));

    let flags = summary::StationFlags {
        storage: storage.is_some(),
        manipulator: manipulator.is_some(),
        processor: processor.is_some(),
        airlock: airlock_state.is_some(),
        computer: computer.is_some(),
        reactor: reactor.is_some(),
        turret: turret.is_some(),
        drone: drone_station.is_some(),
    };

    let title = if runtime_module.kind.supports_channel() {
        format!(
            "{} Console  [CH{}]",
            module_display_name(runtime_module.kind),
            runtime_module.channel
        )
    } else {
        format!("{} Console", module_display_name(runtime_module.kind))
    };

    if destroyed.is_some() {
        return StationPanelDisplay {
            title,
            active_station_kind: Some(runtime_module.kind),
            flags,
            readouts: vec![
                readout_bar(
                    register_readout_label(runtime_module, "MII", "Integrity"),
                    format!("{} / {}", integrity.current, integrity.max),
                    percent_from_ratio(integrity.current as f32, integrity.max.max(1) as f32),
                    Color::srgb(0.80, 0.34, 0.34),
                ),
                readout_bar(
                    register_readout_label(runtime_module, "MPH", "Heat"),
                    format_fx1(runtime_state.current_heat),
                    percent_from_fx(runtime_state.current_heat, Fx::from_num(16)),
                    Color::srgb(0.96, 0.50, 0.24),
                ),
                readout_bar(
                    register_readout_label(runtime_module, "MPE", "Electrical"),
                    format_fx1(runtime_state.electrical_instability),
                    percent_from_fx(runtime_state.electrical_instability, Fx::from_num(12)),
                    Color::srgb(0.74, 0.56, 0.98),
                ),
                readout_light("Status", "Destroyed", Color::srgb(0.90, 0.28, 0.28)),
            ],
        };
    }

    let readouts = if let Some(reactor) = reactor {
        vec![
            readout_bar(
                register_readout_label(runtime_module, "RRF", "Reaction"),
                format!("{}%", format_fx0(reactor.reaction_rate * Fx::from_num(100))),
                percent_from_fx(reactor.reaction_rate, Fx::from_num(1)),
                Color::srgb(0.94, 0.42, 0.24),
            ),
            readout_bar(
                register_readout_label(runtime_module, "RRT", "Turbine"),
                format!("{}%", format_fx0(reactor.turbine_load * Fx::from_num(100))),
                percent_from_fx(reactor.turbine_load, Fx::from_num(1)),
                Color::srgb(0.34, 0.74, 0.94),
            ),
            readout_bar(
                register_readout_label(runtime_module, "RRS", "Confinement"),
                format!("{}%", format_fx0(reactor.confinement * Fx::from_num(100))),
                percent_from_fx(reactor.confinement, Fx::from_num(1)),
                Color::srgb(0.70, 0.48, 0.94),
            ),
            readout_bar(
                register_readout_label(runtime_module, "RRP", "Output"),
                format_fx1(reactor.power_output),
                percent_from_fx(reactor.power_output, Fx::from_num(20)),
                Color::srgb(0.86, 0.74, 0.30),
            ),
            readout_bar(
                register_readout_label(runtime_module, "RRE", "Fuel"),
                format_fx0(reactor.fuel_remaining),
                percent_from_fx(reactor.fuel_remaining, Fx::from_num(100)),
                Color::srgb(0.42, 0.86, 0.62),
            ),
            readout_light(
                "Core Status",
                if runtime_state.needs_attention {
                    "Attention"
                } else {
                    "Stable"
                },
                if runtime_state.needs_attention {
                    Color::srgb(0.90, 0.48, 0.18)
                } else {
                    Color::srgb(0.34, 0.78, 0.46)
                },
            ),
        ]
    } else if let Some(turret) = turret {
        vec![
            readout_bar(
                register_readout_label(runtime_module, "WTA", "Aim Error"),
                format_fx1(angle_distance(turret.desired_angle, turret.actual_angle)),
                percent_from_fx(
                    angle_distance(turret.desired_angle, turret.actual_angle),
                    Fx::PI,
                ),
                Color::srgb(0.36, 0.72, 0.96),
            ),
            readout_bar(
                register_readout_label(runtime_module, "WTR", "Desired Arc"),
                format_fx1(turret.desired_angle),
                percent_from_wrapped_angle(turret.desired_angle),
                Color::srgb(0.64, 0.60, 0.96),
            ),
            readout_bar(
                register_readout_label(runtime_module, "WTC", "Cooldown"),
                format_fx2(weapon_state.cooldown_remaining),
                percent_from_fx(weapon_state.cooldown_remaining, Fx::from_num(3)),
                Color::srgb(0.96, 0.56, 0.24),
            ),
            readout_bar(
                register_readout_label(runtime_module, "WTH", "Heat"),
                format_fx1(runtime_state.current_heat),
                percent_from_fx(runtime_state.current_heat, Fx::from_num(16)),
                Color::srgb(0.96, 0.50, 0.24),
            ),
            readout_light(
                "Fire Gate",
                if turret.fire_intent { "Open" } else { "Hold" },
                if turret.fire_intent {
                    Color::srgb(0.34, 0.78, 0.46)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            readout_light(
                "Threat State",
                mission_status_line(mission_state),
                if mission_state.failed {
                    Color::srgb(0.84, 0.28, 0.28)
                } else {
                    Color::srgb(0.34, 0.78, 0.46)
                },
            ),
        ]
    } else if storage.is_some()
        || manipulator.is_some()
        || processor.is_some()
        || airlock_state.is_some()
    {
        logistics_readouts(
            storage,
            storage_command,
            manipulator,
            manipulator_command,
            processor,
            processor_command,
            airlock_state,
            drone_station,
            drone_station_command,
            focused_station_context,
        )
    } else if let Some(computer) = computer {
        let budget_pct = if computer.last_result.budget == 0 {
            0.0
        } else {
            percent_from_ratio(
                computer.last_result.executed as f32,
                computer.last_result.budget as f32,
            )
        };
        vec![
            readout_light(
                register_readout_label(runtime_module, "CCA", "Online"),
                if computer.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                if computer.enabled {
                    Color::srgb(0.34, 0.78, 0.46)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            readout_bar(
                register_readout_label(runtime_module, "CCB", "Exec Budget"),
                format!(
                    "{}/{}",
                    computer.last_result.executed, computer.last_result.budget
                ),
                budget_pct,
                Color::srgb(0.36, 0.72, 0.96),
            ),
            readout_light(
                register_readout_label(runtime_module, "CCW", "Writes"),
                arch_summary.recent_writes.as_str(),
                Color::srgb(0.90, 0.64, 0.20),
            ),
            readout_light(
                register_readout_label(runtime_module, "CLT", "LUMEN"),
                if computer.lumen_program.enabled {
                    computer
                        .last_lumen_result
                        .recent_effects
                        .first()
                        .map(|effect| effect.as_str())
                        .unwrap_or("Ready")
                } else {
                    "Disabled"
                },
                if computer.lumen_program.enabled {
                    Color::srgb(0.44, 0.88, 0.68)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            readout_light(
                register_readout_label(runtime_module, "CCH", "Halt State"),
                computer
                    .last_result
                    .halted_reason
                    .as_deref()
                    .unwrap_or("Running"),
                if computer.last_result.halted_reason.is_some() {
                    Color::srgb(0.90, 0.48, 0.18)
                } else {
                    Color::srgb(0.34, 0.78, 0.46)
                },
            ),
            readout_light(
                register_readout_label(runtime_module, "CCP", "Program"),
                computer.program.name.as_str(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
            readout_light(
                "ARCH Source",
                first_program_line(&computer.program.source_text),
                Color::srgb(0.72, 0.82, 0.96),
            ),
            readout_light(
                "LUMEN Source",
                first_program_line(&computer.lumen_program.source_text),
                Color::srgb(0.62, 0.90, 0.80),
            ),
        ]
    } else {
        vec![
            readout_bar(
                register_readout_label(runtime_module, "MII", "Integrity"),
                format!("{} / {}", integrity.current, integrity.max),
                percent_from_ratio(integrity.current as f32, integrity.max.max(1) as f32),
                Color::srgb(0.80, 0.34, 0.34),
            ),
            readout_bar(
                register_readout_label(runtime_module, "MPH", "Heat"),
                format_fx1(runtime_state.current_heat),
                percent_from_fx(runtime_state.current_heat, Fx::from_num(16)),
                Color::srgb(0.96, 0.50, 0.24),
            ),
            readout_bar(
                register_readout_label(runtime_module, "MPE", "Electrical"),
                format_fx1(runtime_state.electrical_instability),
                percent_from_fx(runtime_state.electrical_instability, Fx::from_num(12)),
                Color::srgb(0.74, 0.56, 0.98),
            ),
            readout_light(
                "Context",
                focused_station_context,
                Color::srgb(0.52, 0.76, 0.96),
            ),
            readout_light(
                "Attention",
                if runtime_state.needs_attention {
                    "Required"
                } else {
                    "Nominal"
                },
                if runtime_state.needs_attention {
                    Color::srgb(0.90, 0.48, 0.18)
                } else {
                    Color::srgb(0.34, 0.78, 0.46)
                },
            ),
        ]
    };

    StationPanelDisplay {
        title,
        active_station_kind: Some(runtime_module.kind),
        flags,
        readouts,
    }
}

pub(crate) fn station_panel_content(
    control_mode: &ShipboardControlState,
    mission_state: &MissionState,
    weapon_state: &ShipWeaponState,
    player_fields: &PlayerFieldState,
    player_motion: &PlayerMotionState,
    module_query: &Query<
        (
            Entity,
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    drone_query: &Query<
        (
            Option<&DroneStationModule>,
            Option<&DroneStationCommandState>,
        ),
        With<RuntimeShipModule>,
    >,
    focused_station_context: &str,
    arch_summary: &summary::ArchSummary,
) -> (
    String,
    String,
    Option<crate::ship::ModuleKind>,
    summary::StationFlags,
) {
    let Some(focused_entity) = control_mode.focused_entity else {
        return (
            "Station Console".to_string(),
            format!(
                "Walk the ship or EVA, then press E at a station.\nCurrent frame: {}\nLocal oxygen: {}  heat: {}  electrical: {}\nRecent action: {}",
                match player_motion.frame {
                    PlayerReferenceFrame::World => "EVA / World",
                    PlayerReferenceFrame::Ship(_) => "Ship Local",
                },
                format_fx1(player_fields.local_oxygen),
                format_fx1(player_fields.local_heat),
                format_fx1(player_fields.local_electrical),
                mission_state.recent_action.as_deref().unwrap_or("none"),
            ),
            None,
            summary::StationFlags::default(),
        );
    };

    let Ok((
        _,
        runtime_module,
        integrity,
        runtime_state,
        computer,
        storage,
        storage_command,
        manipulator,
        _manipulator_command,
        processor,
        processor_command,
        reactor,
        turret,
        airlock_state,
        destroyed,
    )) = module_query.get(focused_entity)
    else {
        return (
            "Station Console".to_string(),
            "Focused module unavailable".to_string(),
            None,
            summary::StationFlags::default(),
        );
    };
    let (drone_station, drone_station_command) =
        drone_query.get(focused_entity).ok().unwrap_or((None, None));

    let flags = summary::StationFlags {
        storage: storage.is_some(),
        manipulator: manipulator.is_some(),
        processor: processor.is_some(),
        airlock: airlock_state.is_some(),
        computer: computer.is_some(),
        reactor: reactor.is_some(),
        turret: turret.is_some(),
        drone: drone_station.is_some(),
    };

    if destroyed.is_some() {
        return (
            format!("{} Console", module_display_name(runtime_module.kind)),
            format!(
                "Context: {}\nModule destroyed.\nIntegrity: {} / {}\nHeat: {}  Electrical: {}\nLeave with Q or Esc.",
                focused_station_context,
                integrity.current,
                integrity.max,
                format_fx1(runtime_state.current_heat),
                format_fx1(runtime_state.electrical_instability),
            ),
            Some(runtime_module.kind),
            flags,
        );
    }

    let title = match control_mode.mode {
        ShipControlMode::Interior => "Vessel Console".to_string(),
        _ => format!("{} Console", module_display_name(runtime_module.kind)),
    };

    let body = if let Some(reactor) = reactor {
        format!(
            "Context: {}\nReaction Rate: {}%\nTurbine Load: {}%\nPower Output: {}\nFuel Remaining: {}\nCore Heat: {}\nInstability: {}\nUse the buttons below or W/S + A/D.\nRecent: {}",
            focused_station_context,
            format_fx0(reactor.reaction_rate * Fx::from_num(100)),
            format_fx0(reactor.turbine_load * Fx::from_num(100)),
            format_fx1(reactor.power_output),
            format_fx0(reactor.fuel_remaining),
            format_fx1(runtime_state.current_heat),
            format_fx1(runtime_state.electrical_instability),
            mission_state.recent_action.as_deref().unwrap_or("none"),
        )
    } else if let Some(turret) = turret {
        format!(
            "Context: {}\nDesired Angle: {}\nActual Angle: {}\nFire Gate: {}\nCooldown: {}\nThreat State: {}\nUse buttons or mouse / A,D while manned.",
            focused_station_context,
            format_fx1(turret.desired_angle),
            format_fx1(turret.actual_angle),
            if turret.fire_intent { "open" } else { "hold" },
            format_fx2(weapon_state.cooldown_remaining),
            mission_status_line(mission_state),
        )
    } else if let Some(storage) = storage {
        format!(
            "Context: {}\nStorage Fill: {}/{}\nRaw Salvage: {}\nRepair Charge: {}\nIntake: {}\nAirlock Seal: {}\nManipulator: {}\nCargo can be moved with panel buttons or F/G.",
            focused_station_context,
            storage.inventory.total_units(),
            storage.capacity,
            storage.inventory.raw_salvage,
            storage.inventory.repair_charge,
            if storage_command.is_some_and(|command| command.allow_intake) {
                "open"
            } else {
                "closed"
            },
            if airlock_state.is_some_and(|state| state.open) {
                "open"
            } else {
                "sealed"
            },
            manipulator
                .and_then(|manipulator| manipulator.blocked_reason.clone())
                .unwrap_or_else(|| "ready".to_string()),
        )
    } else if let Some(processor) = processor {
        let progress_pct = if processor.duration > Fx::from_num(0) {
            processor.progress / processor.duration * Fx::from_num(100)
        } else {
            Fx::from_num(0)
        };
        format!(
            "Context: {}\nRecipe: {}\nEnabled: {}\nProgress: {}%\nHeld Raw: {}\nHeld Charge: {}\nState: {}\nFabrication output feeds the ship through logistics.",
            focused_station_context,
            processor_command
                .map(|command| command.selected_recipe.as_str())
                .unwrap_or("Repair Charge"),
            if processor_command.is_none_or(|command| command.enabled) {
                "yes"
            } else {
                "no"
            },
            format_fx0(progress_pct),
            processor.inventory.raw_salvage,
            processor.inventory.repair_charge,
            processor.blocked_reason.as_deref().unwrap_or("running"),
        )
    } else if let Some(drone_station) = drone_station {
        format!(
            "Context: {}\nDrone mode: {}\nActive drones: {} / {}\nIdle drones: {}\nQueued tasks: {}\nPower draw: {}\nStatus: {}\nDrones extend logistics beyond adjacency and return to station when idle.",
            focused_station_context,
            drone_station_command
                .map(|command| command.selected_task.as_str())
                .unwrap_or("Logistics"),
            drone_station.active_drones,
            drone_station.max_drones,
            drone_station.idle_drones,
            drone_station.queued_tasks,
            format_fx1(drone_station.power_draw),
            drone_station.last_status,
        )
    } else if let Some(computer) = computer {
        format!(
            "Context: {}\nProgram: {}\nOnline: {}\nExec: {}\nWrites: {}\nInvalid/Halt: {}\nUse buttons here, then refine logic in the refit editor.",
            focused_station_context,
            computer.program.name,
            if computer.enabled { "yes" } else { "no" },
            arch_summary.exec_summary,
            arch_summary.recent_writes,
            arch_summary.invalid_count,
        )
    } else {
        format!(
            "Context: {}\nModule: {}\nCondition: {:?}\nHeat: {}\nElectrical: {}\nNeeds Attention: {}",
            focused_station_context,
            module_display_name(runtime_module.kind),
            module_condition(integrity, runtime_state, false, &BalanceConfig::default()),
            format_fx1(runtime_state.current_heat),
            format_fx1(runtime_state.electrical_instability),
            if runtime_state.needs_attention {
                "yes"
            } else {
                "no"
            },
        )
    };

    (title, body, Some(runtime_module.kind), flags)
}
