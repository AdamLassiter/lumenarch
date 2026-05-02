use bevy::log;

use super::*;

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
