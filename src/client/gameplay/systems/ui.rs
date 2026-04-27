use bevy::prelude::*;

use super::{
    super::{
        super::state::{
            DemoProgression,
            GameplayAlertsText,
            GameplayInspectionText,
            GameplayStatusText,
        },
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
            RuntimeShipModule,
            ShipRoot,
            ShipWeaponState,
            ShipboardControlState,
            ShipboardPlayer,
            SimPosition,
            WeaponModule,
        },
        helpers::{
            Fx,
            danger_level,
            format_fx0,
            format_fx1,
            format_fx2,
            mission_return_line,
            mission_status_line,
            module_condition,
            module_condition_label,
            salvage_status_line,
        },
    },
    shared::{condition_severity, interaction_label, module_display_name},
};

pub(crate) fn update_gameplay_status_text(
    ship_query: Single<
        (
            &SimPosition,
            &Children,
            &super::super::components::LinearVelocity,
            &super::super::components::AngularVelocity,
            &super::super::components::ShipMovementModel,
            &super::super::components::ShipPowerState,
            &ShipWeaponState,
            &MissionState,
            &ShipboardControlState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<(&CurrentStation, &PlayerFieldState), With<ShipboardPlayer>>,
    hostile_query: Query<Entity, With<super::super::components::HostileTarget>>,
    projectile_query: Query<Entity, With<super::super::components::Projectile>>,
    module_query: Query<
        (
            &Integrity,
            &ModuleRuntimeState,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
    salvage_query: Query<
        (&SimPosition, &super::super::components::SalvagePickup),
        (
            With<super::super::components::SalvageWreck>,
            Without<super::super::components::CollectedSalvage>,
        ),
    >,
    progression: Res<DemoProgression>,
    mut status_query: Query<&mut Text, With<GameplayStatusText>>,
) {
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        movement_model,
        power_state,
        weapon_state,
        mission_state,
        control_mode,
    ) = ship_query.into_inner();
    let (current_station, player_fields) = player_query.into_inner();

    let salvage_line = salvage_status_line(ship_position.value, mission_state, &salvage_query);
    let mut current_integrity = 0i32;
    let mut max_integrity = 0i32;
    let mut active_modules = 0usize;
    let mut degraded_modules = 0usize;
    let mut disabled_modules = 0usize;

    for child in children.iter() {
        let Ok((integrity, runtime_state, destroyed, _)) = module_query.get(*child) else {
            continue;
        };
        max_integrity += integrity.max;
        let condition = module_condition(integrity, runtime_state, destroyed.is_some());
        if condition != super::super::components::ModuleCondition::Destroyed {
            current_integrity += integrity.current;
            active_modules += 1;
        }
        match condition {
            super::super::components::ModuleCondition::Degraded => degraded_modules += 1,
            super::super::components::ModuleCondition::Disabled => disabled_modules += 1,
            _ => {}
        }
    }

    for mut text in &mut status_query {
        let status_line = match mission_return_line(mission_state) {
            Some(return_line) => format!("{}\n{}", mission_status_line(mission_state), return_line),
            None => mission_status_line(mission_state).to_string(),
        };
        **text = format!(
            "Mission Status\nOutcome: {}\nMode: {:?}\nStation: {}\nPosition: {}, {}\nVelocity: {}\nTurn Rate: {}\nIntegrity\nHull / Systems: {} / {}\nActive Modules: {}\nDegraded / Disabled: {} / {}\nPower\nEngine Output: {} ({}%)\nGeneration / Draw: {} / {}\nBattery Reserve: {}\nWeapons Online: {}\nCombat\nTurrets: {}  Cooldown: {}\nProjectiles: {}  Hostiles: {}\nInterior\nLocal Heat: {} ({})\nLocal Electrical: {} ({})\nSalvage: {}\nScrap Total: {}",
            status_line,
            control_mode.mode,
            module_display_name(current_station.kind),
            format_fx0(ship_position.value.x),
            format_fx0(ship_position.value.y),
            format_fx1(linear_velocity.value.length()),
            format_fx2(angular_velocity.radians_per_second),
            current_integrity,
            max_integrity,
            active_modules,
            degraded_modules,
            disabled_modules,
            movement_model.engine_count,
            format_fx0(power_state.engine_power_ratio * Fx::from_num(100)),
            format_fx1(power_state.generation),
            format_fx1(power_state.draw),
            format_fx1(power_state.stored_energy),
            if power_state.weapons_powered {
                "yes"
            } else {
                "no"
            },
            weapon_state.turret_count,
            format_fx2(weapon_state.cooldown_remaining.max(Fx::from_num(0))),
            projectile_query.iter().len(),
            hostile_query.iter().len(),
            format_fx1(player_fields.local_heat),
            danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14)),
            format_fx1(player_fields.local_electrical),
            danger_level(
                player_fields.local_electrical,
                Fx::from_num(7),
                Fx::from_num(12)
            ),
            salvage_line,
            progression.scrap,
        );
    }
}

pub(crate) fn update_inspection_and_alerts_text(
    player_query: Single<
        (
            &CurrentStation,
            &NearbyInteraction,
            &HeldInteraction,
            &PlayerFieldState,
        ),
        With<ShipboardPlayer>,
    >,
    module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    mut inspection_query: Query<
        &mut Text,
        (With<GameplayInspectionText>, Without<GameplayAlertsText>),
    >,
    mut alerts_query: Query<&mut Text, (With<GameplayAlertsText>, Without<GameplayInspectionText>)>,
) {
    let (station, nearby, held, player_fields) = player_query.into_inner();
    let current_module = module_query
        .iter()
        .find(|(runtime_module, _, _, _)| runtime_module.module_id == station.module_id);

    for mut text in &mut inspection_query {
        if let Some((runtime_module, integrity, runtime_state, destroyed)) = current_module {
            let condition = module_condition(integrity, runtime_state, destroyed.is_some());
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
            **text = format!(
                "Module: {}\nGrid: {}, {}\nIntegrity: {} / {}\nCondition: {}\nHeat: {}\nElectrical: {}\nField Heat: {}\nField Electrical: {}\nNeeds Attention: {}\n{}",
                module_display_name(runtime_module.kind),
                runtime_module.grid_x,
                runtime_module.grid_y,
                integrity.current,
                integrity.max,
                module_condition_label(condition),
                format_fx1(runtime_state.current_heat),
                format_fx1(runtime_state.electrical_instability),
                format_fx1(runtime_state.sampled_heat),
                format_fx1(runtime_state.sampled_electrical),
                if runtime_state.needs_attention {
                    "yes"
                } else {
                    "no"
                },
                interaction_line,
            );
        } else {
            **text = "Module: unavailable".to_string();
        }
    }

    let mut issues: Vec<(i32, String)> = module_query
        .iter()
        .filter_map(|(runtime_module, integrity, runtime_state, destroyed)| {
            let condition = module_condition(integrity, runtime_state, destroyed.is_some());
            let severity = condition_severity(condition);
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
        })
        .collect();
    issues.sort_by(|a, b| b.0.cmp(&a.0));
    issues.truncate(3);

    for mut text in &mut alerts_query {
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
        **text = format!(
            "Local Heat: {} ({})\nLocal Electrical: {} ({})\n{}\nPrompt: {}",
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
        );
    }
}
