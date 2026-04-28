use bevy::prelude::*;

use crate::client::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            CurrentStation,
            DestroyedModule,
            HeldInteraction,
            Integrity,
            MissionState,
            ModuleRuntimeState,
            NearbyInteraction,
            PlayerFieldState,
            PlayerShip,
            ProcessorModule,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipAutomationState,
            ShipRoot,
            ShipboardPlayer,
            StorageModule,
        },
        helpers::{
            Fx,
            danger_level,
            format_fx1,
            meter_bar,
            module_condition,
            module_condition_label,
        },
        systems::shared::{condition_severity, interaction_label, module_display_name},
    },
    state::{GameplayAlertsText, GameplayInspectionText},
};

pub(crate) fn update_inspection_and_alerts_text(
    balance: Res<BalanceConfig>,
    player_query: Single<
        (
            &CurrentStation,
            &NearbyInteraction,
            &HeldInteraction,
            &PlayerFieldState,
        ),
        With<ShipboardPlayer>,
    >,
    ship_query: Single<(&ShipAutomationState, &MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    mut inspection_query: Query<
        &mut Text,
        (With<GameplayInspectionText>, Without<GameplayAlertsText>),
    >,
    mut alerts_query: Query<&mut Text, (With<GameplayAlertsText>, Without<GameplayInspectionText>)>,
) {
    let (station, nearby, held, player_fields) = player_query.into_inner();
    let (automation_state, mission_state) = ship_query.into_inner();
    let current_module = module_query
        .iter()
        .find(|(runtime_module, _, _, _, _, _, _)| runtime_module.module_id == station.module_id);

    for mut text in &mut inspection_query {
        **text = inspection_text(current_module, nearby, held, automation_state, &balance);
    }

    let mut issues = collect_alert_issues(&module_query, &balance);
    issues.sort_by_key(|a| std::cmp::Reverse(a.0));
    issues.truncate(3);

    for mut text in &mut alerts_query {
        **text = alerts_text(player_fields, nearby, mission_state, &issues);
    }
}

fn inspection_text(
    current_module: Option<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    nearby: &NearbyInteraction,
    held: &HeldInteraction,
    automation_state: &ShipAutomationState,
    balance: &BalanceConfig,
) -> String {
    let Some((runtime_module, integrity, runtime_state, computer, storage, processor, destroyed)) =
        current_module
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
            "Computer: {}  exec {}/{}  {}",
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
                .unwrap_or("ok")
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

    format!(
        "Module: {}\nGrid: {}, {}\nIntegrity: {} / {}\nCondition: {}\nHeat: {} {}\nElectrical: {} {}\nField Heat: {}\nField Electrical: {}\nNeeds Attention: {}\n{}\nARCH: {:?} ({})\n{}",
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
    )
}

fn collect_alert_issues(
    module_query: &Query<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
    balance: &BalanceConfig,
) -> Vec<(i32, String)> {
    module_query
        .iter()
        .filter_map(
            |(
                runtime_module,
                integrity,
                runtime_state,
                _computer,
                storage,
                processor,
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

fn alerts_text(
    player_fields: &PlayerFieldState,
    nearby: &NearbyInteraction,
    mission_state: &MissionState,
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
        "Local Heat: {} ({})\nLocal Electrical: {} ({})\n{}\nPrompt: {}\nHottest: {}\nFirst Disabled: {}\nRecent: {}\nFlow: raw {}  charge {}  transfers {}",
        format_fx1(player_fields.local_heat),
        danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14)),
        format_fx1(player_fields.local_electrical),
        danger_level(
            player_fields.local_electrical,
            Fx::from_num(7),
            Fx::from_num(12)
        ),
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
