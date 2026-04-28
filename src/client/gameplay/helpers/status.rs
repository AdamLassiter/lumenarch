use bevy::prelude::*;

use super::{FixedVec2, Fx, fixed_radius_sq};
use crate::{
    client::gameplay::components::{
        CollectedSalvage,
        MissionState,
        SalvagePickup,
        SalvageWreck,
        SimPosition,
    },
    ship::ShipDefinition,
};

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

pub(crate) fn salvage_status_line(
    ship_position: FixedVec2,
    mission_state: &MissionState,
    salvage_query: &Query<
        (&SimPosition, &SalvagePickup),
        (With<SalvageWreck>, Without<CollectedSalvage>),
    >,
) -> String {
    if mission_state.salvage_collected {
        return format!("recovered {} scrap", mission_state.salvage_scrap_awarded);
    }

    if !mission_state.encounter_cleared || mission_state.failed {
        return "secure the encounter first".to_string();
    }

    let pickup_radius_sq = fixed_radius_sq(super::super::SALVAGE_PICKUP_RADIUS);
    for (position, salvage) in salvage_query.iter() {
        if ship_position.distance_sq(position.value) <= pickup_radius_sq {
            return format!("press F for {} scrap", salvage.scrap_value);
        }
    }

    "find the salvage wreck".to_string()
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
