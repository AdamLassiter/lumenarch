use super::*;
use crate::ship::ModuleKind;

pub(super) fn pending_station_command(
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
        StationPanelButtonAction::InfrastructureToggleBlocker
            if control_state.mode == ShipControlMode::Logistics =>
        {
            Some(netcode::PendingStationCommand {
                op: netcode::StationControlOp::InfrastructureToggleBlocker,
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

pub(crate) fn station_action_visible(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    active_station_kind: Option<ModuleKind>,
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
            StationPanelButtonAction::LogisticsToggleProcessor => flags.processor || flags.drone,
            StationPanelButtonAction::InfrastructureToggleBlocker => flags.blocker,
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

pub(crate) fn station_button_label(
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
        StationPanelButtonAction::LogisticsToggleProcessor => {
            if flags.drone && !flags.processor {
                "Cycle Drone Mode".to_string()
            } else {
                "Toggle Processor".to_string()
            }
        }
        StationPanelButtonAction::InfrastructureToggleBlocker => "Open / Close".to_string(),
        StationPanelButtonAction::ComputerToggleEnabled => "Enable / Disable".to_string(),
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template".to_string(),
    }
}
