use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        AngularVelocity,
        CollectedSalvage,
        CurrentStation,
        DestroyedModule,
        Integrity,
        LinearVelocity,
        MissionState,
        ModuleCondition,
        ModuleRuntimeState,
        PlayerFieldState,
        PlayerShip,
        ProcessorModule,
        Projectile,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipAutomationState,
        ShipMovementModel,
        ShipPowerState,
        ShipRoot,
        ShipWeaponState,
        ShipboardControlState,
        ShipboardPlayer,
        SimPosition,
        StorageModule,
        WeaponModule,
        HostileTarget,
        SalvagePickup,
        SalvageWreck,
    },
    helpers::{
        danger_level,
        format_fx0,
        format_fx1,
        format_fx2,
        meter_bar,
        mission_return_line,
        mission_status_line,
        module_condition,
        salvage_status_line,
        Fx,
    },
    systems::shared::module_display_name,
};
use crate::client::state::{DemoProgression, GameplayStatusText};

pub(crate) fn update_gameplay_status_text(
    ship_query: Single<
        (
            &SimPosition,
            &Children,
            &LinearVelocity,
            &AngularVelocity,
            &ShipAutomationState,
            &ShipMovementModel,
            &ShipPowerState,
            &ShipWeaponState,
            &MissionState,
            &ShipboardControlState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<(&CurrentStation, &PlayerFieldState), With<ShipboardPlayer>>,
    hostile_query: Query<Entity, With<HostileTarget>>,
    projectile_query: Query<Entity, With<Projectile>>,
    module_query: Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&ProcessorModule>,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
    salvage_query: Query<
        (&SimPosition, &SalvagePickup),
        (With<SalvageWreck>, Without<CollectedSalvage>),
    >,
    progression: Res<DemoProgression>,
    mut status_query: Query<&mut Text, With<GameplayStatusText>>,
) {
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        automation_state,
        movement_model,
        power_state,
        weapon_state,
        mission_state,
        control_mode,
    ) = ship_query.into_inner();
    let (current_station, player_fields) = player_query.into_inner();

    let salvage_line = salvage_status_line(ship_position.value, mission_state, &salvage_query);
    let (current_integrity, max_integrity, active_modules, degraded_modules, disabled_modules) =
        summarize_modules(children, &module_query);
    let (arch_program, arch_exec, arch_invalid, arch_writes) =
        summarize_arch(children, &module_query);

    for mut text in &mut status_query {
        let status_line = match mission_return_line(mission_state) {
            Some(return_line) => format!("{}\n{}", mission_status_line(mission_state), return_line),
            None => mission_status_line(mission_state).to_string(),
        };
        **text = format!(
            "Mission Status\nOutcome: {}\nMode: {:?}\nStation: {}\nPosition: {}, {}\nVelocity: {}\nTurn Rate: {}\nIntegrity\nHull / Systems: {} / {}\nActive Modules: {}\nDegraded / Disabled: {} / {}\nPower\nEngine Output: {} ({}%)\nGeneration / Draw: {} / {}\nBattery Reserve: {}\nWeapons Online: {}\nARCH\nMode: {:?}\nStatus: {}\nProgram: {}\nExec / Invalid: {} / {}\nWrites: {}\nCombat\nTurrets: {}  Cooldown: {}\nProjectiles: {}  Hostiles: {}\nLogistics\nRecovered Raw: {}\nProcessed / Used Charges: {} / {}\nTransfers / Cycles: {} / {}\nBottleneck: {}\nInterventions\nRepairs / Stabilizations: {} / {}\nRecent: {}\nInterior\nLocal Heat: {} {} ({})\nLocal Electrical: {} {} ({})\nSalvage: {}\nScrap Total: {}",
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
            if power_state.weapons_powered { "yes" } else { "no" },
            automation_state.mode,
            if automation_state.active { "engaged" } else { "standby" },
            arch_program,
            arch_exec,
            arch_invalid,
            arch_writes,
            weapon_state.turret_count,
            format_fx2(weapon_state.cooldown_remaining.max(Fx::from_num(0))),
            projectile_query.iter().len(),
            hostile_query.iter().len(),
            mission_state.recovered_raw_salvage,
            mission_state.processed_repair_charge,
            mission_state.consumed_repair_charge,
            mission_state.transfer_count,
            mission_state.processor_cycles,
            mission_state
                .logistics_bottleneck
                .as_deref()
                .unwrap_or("none"),
            mission_state.repairs_performed,
            mission_state.stabilizations_performed,
            mission_state.recent_action.as_deref().unwrap_or("none"),
            format_fx1(player_fields.local_heat),
            meter_bar(player_fields.local_heat, Fx::from_num(16), 12),
            danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14)),
            format_fx1(player_fields.local_electrical),
            meter_bar(player_fields.local_electrical, Fx::from_num(14), 12),
            danger_level(player_fields.local_electrical, Fx::from_num(7), Fx::from_num(12)),
            salvage_line,
            progression.scrap,
        );
    }
}

fn summarize_modules(
    children: &Children,
    module_query: &Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&ProcessorModule>,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> (i32, i32, usize, usize, usize) {
    let mut current_integrity = 0i32;
    let mut max_integrity = 0i32;
    let mut active_modules = 0usize;
    let mut degraded_modules = 0usize;
    let mut disabled_modules = 0usize;

    for child in children.iter() {
        let Ok((_, integrity, runtime_state, _, _, _, destroyed, _)) = module_query.get(*child) else {
            continue;
        };
        max_integrity += integrity.max;
        let condition = module_condition(integrity, runtime_state, destroyed.is_some());
        if condition != ModuleCondition::Destroyed {
            current_integrity += integrity.current;
            active_modules += 1;
        }
        match condition {
            ModuleCondition::Degraded => degraded_modules += 1,
            ModuleCondition::Disabled => disabled_modules += 1,
            _ => {}
        }
    }

    (
        current_integrity,
        max_integrity,
        active_modules,
        degraded_modules,
        disabled_modules,
    )
}

fn summarize_arch(
    children: &Children,
    module_query: &Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&ProcessorModule>,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> (String, String, u32, String) {
    for child in children.iter() {
        let Ok((_, _, _, computer, _, _, destroyed, _)) = module_query.get(*child) else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        if let Some(computer) = computer {
            return (
                if computer.last_result.program_name.is_empty() {
                    computer.program.name.clone()
                } else {
                    computer.last_result.program_name.clone()
                },
                format!("{}/{}", computer.last_result.executed, computer.last_result.budget),
                u32::from(computer.last_result.halted_reason.is_some()),
                if computer.last_result.recent_writes.is_empty() {
                    "none".to_string()
                } else {
                    computer.last_result.recent_writes.join(", ")
                },
            );
        }
    }

    (
        "none".to_string(),
        "0/0".to_string(),
        0,
        "none".to_string(),
    )
}
