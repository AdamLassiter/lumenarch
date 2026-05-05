use super::*;
use crate::gameplay::helpers::wrap_radians;

pub(super) fn logistics_readouts(
    storage: Option<&StorageModule>,
    storage_command: Option<&StorageCommandState>,
    manipulator: Option<&ManipulatorModule>,
    manipulator_command: Option<&ManipulatorCommandState>,
    processor: Option<&ProcessorModule>,
    processor_command: Option<&ProcessorCommandState>,
    airlock_state: Option<&AirlockCommandState>,
    drone_station: Option<&DroneStationModule>,
    drone_station_command: Option<&DroneStationCommandState>,
    focused_station_context: &str,
) -> Vec<StationPanelReadout> {
    let mut readouts = Vec::new();

    if let Some(storage) = storage {
        readouts.push(readout_bar(
            "Cargo Fill",
            format!("{}/{}", storage.inventory.total_units(), storage.capacity),
            percent_from_ratio(
                storage.inventory.total_units() as f32,
                storage.capacity.max(1) as f32,
            ),
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
                    format_fx0(processor.progress / processor.duration * Fx::from_num(100))
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
                processor.blocked_reason.as_deref().unwrap_or("Running")
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
    if let Some(drone_station) = drone_station {
        readouts.push(readout_bar(
            "DRN0  Active Drones",
            format!(
                "{}/{}",
                drone_station.active_drones, drone_station.max_drones
            ),
            percent_from_ratio(
                drone_station.active_drones as f32,
                drone_station.max_drones.max(1) as f32,
            ),
            Color::srgb(0.54, 0.86, 1.0),
        ));
        readouts.push(readout_light(
            "DRC0  Drone Mode",
            drone_station_command
                .map(|command| command.selected_task.as_str())
                .unwrap_or("Logistics"),
            Color::srgb(0.72, 0.92, 1.0),
        ));
        readouts.push(readout_light(
            "DRS0  Status",
            &drone_station.last_status,
            if drone_station.active_tasks > 0 {
                Color::srgb(0.36, 0.84, 0.64)
            } else {
                Color::srgb(0.84, 0.68, 0.28)
            },
        ));
        readouts.push(readout_light(
            "DRL0  Queue",
            &format!(
                "{} queued / {} idle",
                drone_station.queued_tasks, drone_station.idle_drones
            ),
            Color::srgb(0.88, 0.78, 0.38),
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

pub(super) fn readout_bar(
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

pub(super) fn readout_light(
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

pub(super) fn register_readout_label(
    runtime_module: &RuntimeShipModule,
    register: &str,
    label: &str,
) -> String {
    if runtime_module.kind.supports_channel() {
        format!("{register}{}  {label}", runtime_module.channel)
    } else {
        format!("{register}  {label}")
    }
}

pub(super) fn first_program_line(source_text: &str) -> &str {
    source_text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no source")
}

pub(super) fn percent_from_ratio(current: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        (current / max * 100.0).clamp(0.0, 100.0)
    }
}

pub(super) fn percent_from_fx(current: Fx, max: Fx) -> f32 {
    if max <= Fx::from_num(0) {
        0.0
    } else {
        (current / max * Fx::from_num(100))
            .clamp(Fx::from_num(0), Fx::from_num(100))
            .to_num::<f32>()
    }
}

pub(super) fn percent_from_wrapped_angle(angle: Fx) -> f32 {
    (((wrap_radians(angle) + Fx::PI) / (Fx::PI * Fx::from_num(2))) * Fx::from_num(100))
        .clamp(Fx::from_num(0), Fx::from_num(100))
        .to_num::<f32>()
}

pub(super) fn angle_distance(a: Fx, b: Fx) -> Fx {
    wrap_radians(a - b).abs()
}
