use bevy::prelude::*;

use super::Fx;
use crate::{
    gameplay::components::{InteractionKind, MissionState, ModuleCondition},
    ship::{ModuleKind, ShipDefinition},
};

pub(crate) fn module_display_name(kind: ModuleKind) -> String {
    sentence_case(kind.as_str())
}

fn sentence_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
        }
    }
}

pub(crate) fn interaction_label(kind: InteractionKind) -> &'static str {
    match kind {
        InteractionKind::Cockpit => "enter cockpit station",
        InteractionKind::Computer => "open computer console",
        InteractionKind::Storage => "open storage panel",
        InteractionKind::Manipulator => "open manipulator panel",
        InteractionKind::Processor => "open processor panel",
        InteractionKind::Reactor => "operate reactor",
        InteractionKind::Turret => "man turret",
        InteractionKind::Engine => "reset engine",
        InteractionKind::Repair => "repair module",
        InteractionKind::Extract => "extract component",
    }
}

pub(crate) fn condition_severity(condition: ModuleCondition) -> i32 {
    match condition {
        ModuleCondition::Healthy => 0,
        ModuleCondition::Degraded => 1,
        ModuleCondition::Disabled => 2,
        ModuleCondition::Destroyed => 3,
    }
}

pub(crate) fn gameplay_status_line(ship: &ShipDefinition) -> String {
    format!(
        "Ship: {}\nModules: {}\nEncounter runtime active\nPress Tab or use the button to abort to station",
        ship.name,
        ship.modules.len()
    )
}

pub(crate) fn mission_status_line(mission_state: &MissionState) -> &str {
    if mission_state.failed {
        mission_state
            .failure_reason
            .as_deref()
            .unwrap_or("Mission failed")
    } else if mission_state.encounter_cleared && !mission_state.salvage_collected {
        "Encounter cleared - recover salvage"
    } else if mission_state.salvage_collected {
        "Salvage recovered"
    } else if mission_state.completed {
        mission_state
            .completion_reason
            .as_deref()
            .unwrap_or("Mission complete")
    } else {
        "Operational"
    }
}

pub(crate) fn mission_return_line(mission_state: &MissionState) -> Option<String> {
    mission_state.return_delay_remaining.map(|seconds| {
        format!(
            "returning to station in {:.1}s",
            seconds.to_num::<f32>().max(0.0)
        )
    })
}

pub(crate) fn meter_bar(value: Fx, max: Fx, width: usize) -> String {
    let safe_max = max.max(Fx::from_num(1));
    let fill = ((value / safe_max).clamp(Fx::from_num(0), Fx::from_num(1))
        * Fx::from_num(width as i32))
    .to_num::<usize>()
    .min(width);
    let empty = width.saturating_sub(fill);
    format!("[{}{}]", "#".repeat(fill), "-".repeat(empty))
}

pub(crate) fn danger_level(value: Fx, warning: Fx, critical: Fx) -> &'static str {
    if value >= critical {
        "critical"
    } else if value >= warning {
        "warning"
    } else {
        "safe"
    }
}
