use bevy::log;

use super::*;
use crate::gameplay::helpers::wrap_radians;

#[derive(Clone)]
pub(super) struct StationPanelReadout {
    pub(super) label: String,
    pub(super) value: String,
    pub(super) visual: StationReadoutVisual,
}

#[derive(Clone, Copy)]
pub(super) enum StationReadoutVisual {
    Bar { percent: f32, color: Color },
    Light { color: Color },
}

pub(super) struct StationPanelDisplay {
    pub(super) title: String,
    pub(super) active_station_kind: Option<crate::ship::ModuleKind>,
    pub(super) flags: summary::StationFlags,
    pub(super) readouts: Vec<StationPanelReadout>,
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

pub(super) fn station_panel_display(
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

    let flags = summary::StationFlags {
        storage: storage.is_some(),
        manipulator: manipulator.is_some(),
        processor: processor.is_some(),
        airlock: airlock_state.is_some(),
        computer: computer.is_some(),
        reactor: reactor.is_some(),
        turret: turret.is_some(),
    };

    let title = format!("{} Console", module_display_name(runtime_module.kind));

    if destroyed.is_some() {
        return StationPanelDisplay {
            title,
            active_station_kind: Some(runtime_module.kind),
            flags,
            readouts: vec![
                readout_bar(
                    "Integrity",
                    format!("{} / {}", integrity.current, integrity.max),
                    percent_from_ratio(integrity.current as f32, integrity.max.max(1) as f32),
                    Color::srgb(0.80, 0.34, 0.34),
                ),
                readout_bar(
                    "Heat",
                    format_fx1(runtime_state.current_heat),
                    percent_from_fx(runtime_state.current_heat, Fx::from_num(16)),
                    Color::srgb(0.96, 0.50, 0.24),
                ),
                readout_bar(
                    "Electrical",
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
                "Reaction",
                format!("{}%", format_fx0(reactor.reaction_rate * Fx::from_num(100))),
                percent_from_fx(reactor.reaction_rate, Fx::from_num(1)),
                Color::srgb(0.94, 0.42, 0.24),
            ),
            readout_bar(
                "Turbine",
                format!("{}%", format_fx0(reactor.turbine_load * Fx::from_num(100))),
                percent_from_fx(reactor.turbine_load, Fx::from_num(1)),
                Color::srgb(0.34, 0.74, 0.94),
            ),
            readout_bar(
                "Confinement",
                format!("{}%", format_fx0(reactor.confinement * Fx::from_num(100))),
                percent_from_fx(reactor.confinement, Fx::from_num(1)),
                Color::srgb(0.70, 0.48, 0.94),
            ),
            readout_bar(
                "Output",
                format_fx1(reactor.power_output),
                percent_from_fx(reactor.power_output, Fx::from_num(20)),
                Color::srgb(0.86, 0.74, 0.30),
            ),
            readout_bar(
                "Fuel",
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
                "Aim Error",
                format_fx1(angle_distance(turret.desired_angle, turret.actual_angle)),
                percent_from_fx(angle_distance(turret.desired_angle, turret.actual_angle), Fx::PI),
                Color::srgb(0.36, 0.72, 0.96),
            ),
            readout_bar(
                "Desired Arc",
                format_fx1(turret.desired_angle),
                percent_from_wrapped_angle(turret.desired_angle),
                Color::srgb(0.64, 0.60, 0.96),
            ),
            readout_bar(
                "Cooldown",
                format_fx2(weapon_state.cooldown_remaining),
                percent_from_fx(weapon_state.cooldown_remaining, Fx::from_num(3)),
                Color::srgb(0.96, 0.56, 0.24),
            ),
            readout_bar(
                "Heat",
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
    } else if storage.is_some() || manipulator.is_some() || processor.is_some() || airlock_state.is_some() {
        logistics_readouts(
            storage,
            storage_command,
            manipulator,
            manipulator_command,
            processor,
            processor_command,
            airlock_state,
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
                "Online",
                if computer.enabled { "Enabled" } else { "Disabled" },
                if computer.enabled {
                    Color::srgb(0.34, 0.78, 0.46)
                } else {
                    Color::srgb(0.84, 0.28, 0.28)
                },
            ),
            readout_bar(
                "Exec Budget",
                format!(
                    "{}/{}",
                    computer.last_result.executed, computer.last_result.budget
                ),
                budget_pct,
                Color::srgb(0.36, 0.72, 0.96),
            ),
            readout_light(
                "Writes",
                arch_summary.recent_writes.as_str(),
                Color::srgb(0.90, 0.64, 0.20),
            ),
            readout_light(
                "Halt State",
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
                "Program",
                computer.program.name.as_str(),
                Color::srgb(0.52, 0.76, 0.96),
            ),
        ]
    } else {
        vec![
            readout_bar(
                "Integrity",
                format!("{} / {}", integrity.current, integrity.max),
                percent_from_ratio(integrity.current as f32, integrity.max.max(1) as f32),
                Color::srgb(0.80, 0.34, 0.34),
            ),
            readout_bar(
                "Heat",
                format_fx1(runtime_state.current_heat),
                percent_from_fx(runtime_state.current_heat, Fx::from_num(16)),
                Color::srgb(0.96, 0.50, 0.24),
            ),
            readout_bar(
                "Electrical",
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

fn pending_station_command(
    action: StationPanelButtonAction,
    control_state: &ShipboardControlState,
) -> Option<netcode::PendingStationCommand> {
    let requires_focus = !matches!(
        action,
        StationPanelButtonAction::HelmThrottle { .. } | StationPanelButtonAction::HelmTurn { .. }
    );
    if requires_focus && control_state.focused_entity.is_none() {
        return None;
    }

    match action {
        StationPanelButtonAction::HelmThrottle { delta }
            if control_state.mode == ShipControlMode::Cockpit =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::HelmThrottle,
                arg0: (delta * 1000.0) as i16,
            })
        }
        StationPanelButtonAction::HelmTurn { value }
            if control_state.mode == ShipControlMode::Cockpit =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::HelmTurn,
                arg0: (value * 1000.0) as i16,
            })
        }
        StationPanelButtonAction::TurretAdjustAim { delta }
            if control_state.mode == ShipControlMode::Turret =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::TurretAdjustAim,
                arg0: (delta * 1000.0) as i16,
            })
        }
        StationPanelButtonAction::TurretFireToggle
            if control_state.mode == ShipControlMode::Turret =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::TurretFireToggle,
                arg0: 0,
            })
        }
        StationPanelButtonAction::ReactorAdjustRate { delta }
            if control_state.mode == ShipControlMode::Reactor =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::ReactorAdjustRate,
                arg0: (delta * 1000.0) as i16,
            })
        }
        StationPanelButtonAction::ReactorAdjustTurbine { delta }
            if control_state.mode == ShipControlMode::Reactor =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::ReactorAdjustTurbine,
                arg0: (delta * 1000.0) as i16,
            })
        }
        StationPanelButtonAction::LogisticsToggleStorageIntake
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsToggleStorageIntake,
                arg0: 0,
            })
        }
        StationPanelButtonAction::LogisticsToggleAirlock
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsToggleAirlock,
                arg0: 0,
            })
        }
        StationPanelButtonAction::LogisticsToggleManipulator
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsToggleManipulator,
                arg0: 0,
            })
        }
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction }
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsCycleManipulatorTarget,
                arg0: direction as i16,
            })
        }
        StationPanelButtonAction::LogisticsCycleResource
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsCycleResource,
                arg0: 0,
            })
        }
        StationPanelButtonAction::LogisticsToggleProcessor
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::LogisticsToggleProcessor,
                arg0: 0,
            })
        }
        StationPanelButtonAction::ComputerToggleEnabled
            if control_state.mode == ShipControlMode::Computer =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::ComputerToggleEnabled,
                arg0: 0,
            })
        }
        StationPanelButtonAction::ComputerCycleTemplate
            if control_state.mode == ShipControlMode::Computer =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::ComputerCycleTemplate,
                arg0: 0,
            })
        }
        _ => None,
    }
}

fn logistics_readouts(
    storage: Option<&StorageModule>,
    storage_command: Option<&StorageCommandState>,
    manipulator: Option<&ManipulatorModule>,
    manipulator_command: Option<&ManipulatorCommandState>,
    processor: Option<&ProcessorModule>,
    processor_command: Option<&ProcessorCommandState>,
    airlock_state: Option<&AirlockCommandState>,
    focused_station_context: &str,
) -> Vec<StationPanelReadout> {
    let mut readouts = Vec::new();

    if let Some(storage) = storage {
        readouts.push(readout_bar(
            "Cargo Fill",
            format!("{}/{}", storage.inventory.total_units(), storage.capacity),
            percent_from_ratio(storage.inventory.total_units() as f32, storage.capacity.max(1) as f32),
            Color::srgb(0.86, 0.74, 0.30),
        ));
        readouts.push(readout_light(
            "Intake",
            if storage_command.is_some_and(|command| command.allow_intake) {
                "Open"
            } else {
                "Closed"
            },
            if storage_command.is_some_and(|command| command.allow_intake) {
                Color::srgb(0.34, 0.78, 0.46)
            } else {
                Color::srgb(0.84, 0.28, 0.28)
            },
        ));
    }
    if let Some(airlock_state) = airlock_state {
        readouts.push(readout_light(
            "Airlock",
            if airlock_state.open { "Open" } else { "Sealed" },
            if airlock_state.open {
                Color::srgb(0.90, 0.48, 0.18)
            } else {
                Color::srgb(0.34, 0.78, 0.46)
            },
        ));
    }
    if let Some(manipulator) = manipulator {
        let progress_pct = if manipulator.transfer_duration > Fx::from_num(0) {
            percent_from_fx(manipulator.transfer_progress, manipulator.transfer_duration)
        } else {
            0.0
        };
        readouts.push(readout_bar(
            "Transfer",
            if manipulator.active && manipulator.transfer_duration > Fx::from_num(0) {
                format!(
                    "{}%",
                    format_fx0(
                        manipulator.transfer_progress / manipulator.transfer_duration
                            * Fx::from_num(100)
                    )
                )
            } else {
                "Idle".to_string()
            },
            progress_pct,
            Color::srgb(0.36, 0.72, 0.96),
        ));
        readouts.push(readout_light(
            "Manipulator",
            if manipulator_command.is_some_and(|command| command.transfer_enabled) {
                "Enabled"
            } else {
                "Disabled"
            },
            if manipulator_command.is_some_and(|command| command.transfer_enabled) {
                Color::srgb(0.34, 0.78, 0.46)
            } else {
                Color::srgb(0.84, 0.28, 0.28)
            },
        ));
        readouts.push(readout_light(
            "Resource",
            manipulator_command
                .map(|command| resource_kind_label(command.resource_kind))
                .unwrap_or("Unassigned"),
            Color::srgb(0.90, 0.64, 0.20),
        ));
    }
    if let Some(processor) = processor {
        let progress_pct = if processor.duration > Fx::from_num(0) {
            percent_from_fx(processor.progress, processor.duration)
        } else {
            0.0
        };
        readouts.push(readout_bar(
            "Fabrication",
            if processor.duration > Fx::from_num(0) {
                format!(
                    "{}%",
                    format_fx0(
                        processor.progress / processor.duration * Fx::from_num(100)
                    )
                )
            } else {
                "Idle".to_string()
            },
            progress_pct,
            Color::srgb(0.70, 0.48, 0.94),
        ));
        readouts.push(readout_light(
            "Processor",
            if processor_command.is_none_or(|command| command.enabled) {
                processor
                    .blocked_reason
                    .as_deref()
                    .unwrap_or("Running")
            } else {
                "Disabled"
            },
            if processor_command.is_none_or(|command| command.enabled) {
                Color::srgb(0.34, 0.78, 0.46)
            } else {
                Color::srgb(0.84, 0.28, 0.28)
            },
        ));
    }

    if readouts.is_empty() {
        readouts.push(readout_light(
            "Context",
            focused_station_context,
            Color::srgb(0.52, 0.76, 0.96),
        ));
    }

    readouts.truncate(6);
    readouts
}

fn readout_bar(
    label: impl Into<String>,
    value: impl Into<String>,
    percent: f32,
    color: Color,
) -> StationPanelReadout {
    StationPanelReadout {
        label: label.into(),
        value: value.into(),
        visual: StationReadoutVisual::Bar {
            percent: percent.clamp(0.0, 100.0),
            color,
        },
    }
}

fn readout_light(
    label: impl Into<String>,
    value: impl Into<String>,
    color: Color,
) -> StationPanelReadout {
    StationPanelReadout {
        label: label.into(),
        value: value.into(),
        visual: StationReadoutVisual::Light { color },
    }
}

fn percent_from_ratio(current: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        (current / max * 100.0).clamp(0.0, 100.0)
    }
}

fn percent_from_fx(current: Fx, max: Fx) -> f32 {
    if max <= Fx::from_num(0) {
        0.0
    } else {
        (current / max * Fx::from_num(100))
            .clamp(Fx::from_num(0), Fx::from_num(100))
            .to_num::<f32>()
    }
}

fn percent_from_wrapped_angle(angle: Fx) -> f32 {
    (((wrap_radians(angle) + Fx::PI) / (Fx::PI * Fx::from_num(2))) * Fx::from_num(100))
        .clamp(Fx::from_num(0), Fx::from_num(100))
        .to_num::<f32>()
}

fn angle_distance(a: Fx, b: Fx) -> Fx {
    wrap_radians(a - b).abs()
}

pub(super) fn station_panel_content(
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

    let flags = summary::StationFlags {
        storage: storage.is_some(),
        manipulator: manipulator.is_some(),
        processor: processor.is_some(),
        airlock: airlock_state.is_some(),
        computer: computer.is_some(),
        reactor: reactor.is_some(),
        turret: turret.is_some(),
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

pub(super) fn station_action_visible(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    active_station_kind: Option<crate::ship::ModuleKind>,
    flags: summary::StationFlags,
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
            StationPanelButtonAction::LogisticsToggleProcessor => flags.processor,
            _ => false,
        },
        ShipControlMode::Computer => matches!(
            action,
            StationPanelButtonAction::ComputerToggleEnabled
                | StationPanelButtonAction::ComputerCycleTemplate
        ),
        ShipControlMode::Interior => active_station_kind.is_none(),
    }
}

pub(super) fn station_button_label(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    flags: summary::StationFlags,
) -> String {
    match action {
        StationPanelButtonAction::HelmThrottle { delta } if delta < 0.0 => {
            "Throttle Down".to_string()
        }
        StationPanelButtonAction::HelmThrottle { .. } => "Throttle Up".to_string(),
        StationPanelButtonAction::HelmTurn { value } if value < 0.0 => "Turn Port".to_string(),
        StationPanelButtonAction::HelmTurn { value } if value > 0.0 => "Turn Starboard".to_string(),
        StationPanelButtonAction::HelmTurn { .. } => "Steady".to_string(),
        StationPanelButtonAction::TurretAdjustAim { delta } if delta < 0.0 => {
            "Trim Right".to_string()
        }
        StationPanelButtonAction::TurretAdjustAim { .. } => "Trim Left".to_string(),
        StationPanelButtonAction::TurretFireToggle => "Toggle Fire Gate".to_string(),
        StationPanelButtonAction::ReactorAdjustRate { delta } if delta < 0.0 => {
            "Reaction -".to_string()
        }
        StationPanelButtonAction::ReactorAdjustRate { .. } => "Reaction +".to_string(),
        StationPanelButtonAction::ReactorAdjustTurbine { delta } if delta < 0.0 => {
            "Turbine -".to_string()
        }
        StationPanelButtonAction::ReactorAdjustTurbine { .. } => "Turbine +".to_string(),
        StationPanelButtonAction::LogisticsToggleStorageIntake => {
            if flags.storage {
                "Toggle Intake".to_string()
            } else {
                "Storage N/A".to_string()
            }
        }
        StationPanelButtonAction::LogisticsToggleAirlock => {
            if mode == ShipControlMode::Logistics && flags.airlock {
                "Cycle Airlock".to_string()
            } else {
                "Airlock N/A".to_string()
            }
        }
        StationPanelButtonAction::LogisticsToggleManipulator => "Toggle Manipulator".to_string(),
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction }
            if direction < 0 =>
        {
            "Prev Target".to_string()
        }
        StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => {
            "Next Target".to_string()
        }
        StationPanelButtonAction::LogisticsCycleResource => "Cycle Resource".to_string(),
        StationPanelButtonAction::LogisticsToggleProcessor => "Toggle Processor".to_string(),
        StationPanelButtonAction::ComputerToggleEnabled => "Enable / Disable".to_string(),
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template".to_string(),
    }
}
