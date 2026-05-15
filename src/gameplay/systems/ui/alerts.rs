use bevy::prelude::*;

use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            AirlockCommandState,
            DestroyedModule,
            DetectorKind,
            DetectorModule,
            DroneStationModule,
            HeldInteraction,
            Integrity,
            ManipulatorCommandState,
            ManipulatorModule,
            MissionState,
            ModuleRuntimeState,
            NearbyInteraction,
            PlayerFieldState,
            ProcessorCommandState,
            ProcessorModule,
            ReactorCommandState,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipAtmosphereState,
            ShipAutomationState,
            StorageCommandState,
            StorageModule,
            TurretCommandState,
        },
        helpers::{
            Fx,
            condition_severity,
            danger_level,
            format_fx0,
            format_fx1,
            interaction_label,
            meter_bar,
            module_condition,
            module_condition_label,
            module_display_name,
        },
    },
};

/// Collects focused-module inspection text so F2-style diagnostics expose useful live state.
pub(crate) fn inspection_text(
    current_module: Option<(
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
    )>,
    detector: Option<&DetectorModule>,
    drone_station: Option<&DroneStationModule>,
    nearby: &NearbyInteraction,
    held: &HeldInteraction,
    automation_state: &ShipAutomationState,
    balance: &BalanceConfig,
) -> String {
    let Some((
        _entity,
        runtime_module,
        integrity,
        runtime_state,
        computer,
        storage,
        _storage_command,
        _manipulator,
        _manipulator_command,
        processor,
        _processor_command,
        _reactor,
        _turret,
        _airlock,
        destroyed,
    )) = current_module
    else {
        return "Module: unavailable".to_string();
    };

    let condition = module_condition(integrity, runtime_state, destroyed.is_some(), balance);
    let interaction_line = match nearby.kind {
        Some(kind) if held.target.is_some() => format!(
            "Action: {} {:.0}%",
            interaction_label(kind),
            (held.progress / held.required * Fx::from_num(100)).to_num::<f32>()
        ),
        Some(kind) => format!("Action: {}", interaction_label(kind)),
        None => nearby
            .unavailable_reason
            .clone()
            .unwrap_or_else(|| "Action: none".to_string()),
    };
    let logistics_line = if let Some(computer) = computer {
        format!(
            "Computer: {}  exec {}/{}  {}  |  LUMEN: {} ({})",
            if computer.last_result.program_name.is_empty() {
                computer.program.name.as_str()
            } else {
                computer.last_result.program_name.as_str()
            },
            computer.last_result.executed,
            computer.last_result.budget,
            computer
                .last_result
                .halted_reason
                .as_deref()
                .unwrap_or("ok"),
            computer.last_lumen_result.program_name.as_str(),
            if computer.lumen_program.enabled {
                computer
                    .last_lumen_result
                    .recent_effects
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("ready")
            } else {
                "disabled"
            }
        )
    } else if let Some(storage) = storage {
        format!(
            "Storage: raw {}  charge {}  used {}/{}",
            storage.inventory.raw_salvage,
            storage.inventory.repair_charge,
            storage.inventory.total_units(),
            storage.capacity
        )
    } else if let Some(processor) = processor {
        format!(
            "Processor: raw {}  charge {}  state {}",
            processor.inventory.raw_salvage,
            processor.inventory.repair_charge,
            if processor.active {
                "running"
            } else {
                processor.blocked_reason.as_deref().unwrap_or("idle")
            }
        )
    } else {
        "Logistics: none".to_string()
    };

    let register_lines = register_dump(
        runtime_module,
        storage,
        processor,
        computer,
        detector,
        _reactor,
        _turret,
        drone_station,
        automation_state,
    );

    format!(
        "Module: {}\nGrid: {}, {}\nIntegrity: {} / {}\nCondition: {}\nHeat: {} {}\nElectrical: {} {}\nField Heat: {}\nField Electrical: {}\nNeeds Attention: {}\n{}\nARCH: {:?} ({})\n{}\n{}",
        module_display_name(runtime_module.kind),
        runtime_module.grid_x,
        runtime_module.grid_y,
        integrity.current,
        integrity.max,
        module_condition_label(condition),
        format_fx1(runtime_state.current_heat),
        meter_bar(runtime_state.current_heat, Fx::from_num(16), 10),
        format_fx1(runtime_state.electrical_instability),
        meter_bar(runtime_state.electrical_instability, Fx::from_num(14), 10),
        format_fx1(runtime_state.sampled_heat),
        format_fx1(runtime_state.sampled_electrical),
        if runtime_state.needs_attention {
            "yes"
        } else {
            "no"
        },
        logistics_line,
        automation_state.mode,
        if automation_state.active {
            "active"
        } else {
            "standby"
        },
        interaction_line,
        register_lines,
    )
}

fn register_dump(
    runtime_module: &RuntimeShipModule,
    storage: Option<&StorageModule>,
    processor: Option<&ProcessorModule>,
    computer: Option<&RuntimeArchComputer>,
    detector: Option<&DetectorModule>,
    reactor: Option<&ReactorCommandState>,
    turret: Option<&TurretCommandState>,
    drone_station: Option<&DroneStationModule>,
    automation_state: &ShipAutomationState,
) -> String {
    let mut lines = Vec::new();

    if let Some(reactor) = reactor {
        lines.push(register_line(
            runtime_module,
            "RRH0",
            format_fx1(reactor.reaction_rate * Fx::from_num(10)),
        ));
        lines.push(register_line(
            runtime_module,
            "RRS0",
            format_fx1(reactor.power_output),
        ));
    }
    if let Some(storage) = storage {
        lines.push(register_line(
            runtime_module,
            "LSR0",
            storage.inventory.raw_salvage,
        ));
        lines.push(register_line(
            runtime_module,
            "LSR1",
            storage.inventory.repair_charge,
        ));
    }
    if let Some(processor) = processor {
        lines.push(register_line(
            runtime_module,
            "LPR0",
            processor.inventory.raw_salvage,
        ));
        lines.push(register_line(
            runtime_module,
            "LPR1",
            processor.inventory.repair_charge,
        ));
    }
    if let Some(turret) = turret {
        lines.push(register_line(
            runtime_module,
            "WTR0",
            if turret.fire_intent { "1" } else { "0" },
        ));
        lines.push(register_line(
            runtime_module,
            "WTC0",
            format_fx1(turret.actual_angle),
        ));
    }
    if let Some(computer) = computer {
        lines.push(register_line(
            runtime_module,
            "RRC0",
            format!("{:?}", automation_state.mode),
        ));
        lines.push(register_line(
            runtime_module,
            "WTF0",
            if automation_state.active { "1" } else { "0" },
        ));
        if !computer.last_result.recent_writes.is_empty() {
            lines.push(format!(
                "Writes: {}",
                computer.last_result.recent_writes.join(", ")
            ));
        }
    }
    if let Some(detector) = detector {
        match detector.kind {
            DetectorKind::LifeSign => {
                lines.push(register_line(
                    runtime_module,
                    "DLF0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLH0",
                    bool_num(detector.secondary_detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLX0",
                    signed_fx(detector.direction.x),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLY0",
                    signed_fx(detector.direction.y),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLD0",
                    format_fx0(detector.distance),
                ));
            }
            DetectorKind::Ship => {
                lines.push(register_line(
                    runtime_module,
                    "DSN0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DSH0",
                    bool_num(detector.secondary_detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DSX0",
                    signed_fx(detector.direction.x),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DSY0",
                    signed_fx(detector.direction.y),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DSD0",
                    format_fx0(detector.distance),
                ));
            }
            DetectorKind::Damage => {
                lines.push(register_line(
                    runtime_module,
                    "DDM0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DDC0",
                    bool_num(detector.critical),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DDX0",
                    signed_fx(detector.direction.x),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DDY0",
                    signed_fx(detector.direction.y),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DDI0",
                    format_fx1(detector.magnitude),
                ));
            }
            DetectorKind::Power => {
                lines.push(register_line(
                    runtime_module,
                    "DPP0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DPB0",
                    bool_num(detector.secondary_detected),
                ));
            }
            DetectorKind::Heat => {
                lines.push(register_line(
                    runtime_module,
                    "DHH0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DHX0",
                    signed_fx(detector.direction.x),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DHY0",
                    signed_fx(detector.direction.y),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DHI0",
                    format_fx1(detector.magnitude),
                ));
            }
            DetectorKind::Logistics => {
                lines.push(register_line(
                    runtime_module,
                    "DLG0",
                    bool_num(detector.detected),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLX1",
                    signed_fx(detector.direction.x),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLY1",
                    signed_fx(detector.direction.y),
                ));
                lines.push(register_line(
                    runtime_module,
                    "DLI0",
                    format_fx1(detector.magnitude),
                ));
            }
        }
    }
    if let Some(drone_station) = drone_station {
        lines.push(register_line(
            runtime_module,
            "DRN0",
            format!(
                "{}/{}",
                drone_station.active_drones, drone_station.max_drones
            ),
        ));
        lines.push(register_line(
            runtime_module,
            "DRS0",
            drone_station.last_status.as_str(),
        ));
    }

    if lines.is_empty() {
        "Registers: none".to_string()
    } else {
        format!("Registers:\n{}", lines.join("\n"))
    }
}

fn register_line(
    runtime_module: &RuntimeShipModule,
    register: &str,
    value: impl ToString,
) -> String {
    if runtime_module.kind.supports_channel() {
        format!(
            "{register}{} = {}",
            runtime_module.channel,
            value.to_string()
        )
    } else {
        format!("{register} = {}", value.to_string())
    }
}

fn bool_num(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn signed_fx(value: Fx) -> String {
    format!("{:+.1}", value.to_num::<f32>())
}

/// Aggregates actionable encounter issues so the HUD can surface urgent ship problems first.
pub(crate) fn collect_alert_issues(
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
    balance: &BalanceConfig,
) -> Vec<(i32, String)> {
    module_query
        .iter()
        .filter_map(
            |(
                _entity,
                runtime_module,
                integrity,
                runtime_state,
                _computer,
                storage,
                _storage_command,
                _manipulator,
                _manipulator_command,
                processor,
                _processor_command,
                _reactor,
                _turret,
                _airlock,
                destroyed,
            )| {
                let condition =
                    module_condition(integrity, runtime_state, destroyed.is_some(), balance);
                let severity = condition_severity(condition);
                let logistics_issue = storage
                    .filter(|storage| storage.inventory.total_units() >= storage.capacity)
                    .map(|_| format!("{}: storage full", module_display_name(runtime_module.kind)))
                    .or_else(|| {
                        processor.and_then(|processor| {
                            processor.blocked_reason.as_ref().map(|reason| {
                                format!("{}: {}", module_display_name(runtime_module.kind), reason)
                            })
                        })
                    });
                if let Some(line) = logistics_issue {
                    return Some((severity.max(1), line));
                }
                (severity > 0).then(|| {
                    (
                        severity,
                        format!(
                            "{}: {}",
                            module_display_name(runtime_module.kind),
                            module_condition_label(condition)
                        ),
                    )
                })
            },
        )
        .collect()
}

/// Builds the concise alert panel text so players can triage threats without opening a station.
pub(crate) fn alerts_text(
    player_fields: &PlayerFieldState,
    nearby: &NearbyInteraction,
    mission_state: &MissionState,
    atmosphere_state: &ShipAtmosphereState,
    issues: &[(i32, String)],
) -> String {
    let summary = if issues.is_empty() {
        "Ship alerts: no urgent modules".to_string()
    } else {
        format!(
            "Ship alerts:\n{}",
            issues
                .iter()
                .map(|(_, line)| line.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "Local Heat: {} ({})\nLocal Electrical: {} ({})\nLocal Oxygen: {} ({})\nShip Atmosphere: avg {}  min {}  venting {}\n{}\nPrompt: {}\nHottest: {}\nFirst Disabled: {}\nRecent: {}\nFlow: raw {}  charge {}  transfers {}",
        format_fx1(player_fields.local_heat),
        danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14)),
        format_fx1(player_fields.local_electrical),
        danger_level(
            player_fields.local_electrical,
            Fx::from_num(7),
            Fx::from_num(12)
        ),
        format_fx1(player_fields.local_oxygen),
        if player_fields.oxygen_critical {
            "critical"
        } else if player_fields.oxygen_warning {
            "warning"
        } else {
            "stable"
        },
        format_fx1(atmosphere_state.average_oxygen),
        format_fx1(atmosphere_state.minimum_oxygen),
        atmosphere_state.venting_tiles,
        summary,
        nearby
            .prompt
            .clone()
            .or_else(|| nearby.unavailable_reason.clone())
            .unwrap_or_else(|| "none".to_string()),
        mission_state
            .hottest_module_kind
            .map(|kind| kind.as_str())
            .unwrap_or("n/a"),
        mission_state
            .first_disabled_module_kind
            .map(|kind| kind.as_str())
            .unwrap_or("n/a"),
        mission_state.recent_action.as_deref().unwrap_or("none"),
        mission_state.recovered_raw_salvage,
        mission_state.processed_repair_charge,
        mission_state.transfer_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gameplay::components::ShipAutomationMode,
        helpers::FixedVec2,
        ship::{ModuleKind, ModuleVariant},
    };

    #[test]
    fn inspection_text_includes_detector_register_dump() {
        let runtime_module = RuntimeShipModule {
            module_id: 9,
            kind: ModuleKind::Detector,
            variant: ModuleVariant::LifeSurvey,
            channel: 2,
            grid_x: 3,
            grid_y: 4,
            rotation_quadrants: 0,
            local_position: FixedVec2::zero(),
        };
        let integrity = Integrity { current: 7, max: 7 };
        let runtime_state = ModuleRuntimeState {
            current_heat: Fx::from_num(1),
            electrical_instability: Fx::from_num(0.5),
            sampled_heat: Fx::from_num(0.2),
            sampled_electrical: Fx::from_num(0.1),
            is_disabled: false,
            was_disabled_last_frame: false,
            needs_attention: false,
            extracted: false,
            last_interaction_age: Fx::from_num(0),
        };
        let detector = DetectorModule {
            kind: DetectorKind::LifeSign,
            tier: 3,
            range: Fx::from_num(300),
            detected: true,
            secondary_detected: false,
            direction: FixedVec2::from_num(1, 0),
            distance: Fx::from_num(48),
            magnitude: Fx::from_num(0),
            critical: false,
        };
        let automation_state = ShipAutomationState {
            mode: ShipAutomationMode::BalancedOps,
            active: false,
            output_scale: Fx::from_num(1),
            trigger_count: 0,
            invalid_executions: 0,
            last_primary_program: None,
            last_secondary_program: None,
            recent_writes: Vec::new(),
        };

        let text = inspection_text(
            Some((
                Entity::PLACEHOLDER,
                &runtime_module,
                &integrity,
                &runtime_state,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )),
            Some(&detector),
            None,
            &NearbyInteraction::default(),
            &HeldInteraction::default(),
            &automation_state,
            &BalanceConfig::default(),
        );

        assert!(text.contains("Registers:"));
        assert!(text.contains("DLF02 = 1"));
        assert!(text.contains("DLD02 = 48"));
    }
}
