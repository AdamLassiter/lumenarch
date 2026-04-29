use bevy::prelude::*;

use crate::{
    client::{
        balance::BalanceConfig,
        gameplay::{
            components::{
                DestroyedModule,
                HostileTarget,
                Integrity,
                MissionState,
                ModuleRuntimeState,
                PlayerShip,
                ProcessorModule,
                ReactorCommandState,
                RuntimeArchComputer,
                RuntimeShipModule,
                SalvageWreck,
                ShipAutomationState,
                ShipMovementModel,
                ShipPowerModel,
                ShipRoot,
                ShipWeaponState,
                StorageModule,
                WeaponModule,
            },
            helpers::{
                Fx,
                fx_from_time_delta,
                module_effectiveness,
                ship_movement_model_with_effective,
                ship_power_model_with_effective,
            },
        },
        state::{
            ClientAppState,
            DemoProgression,
            LastMissionReport,
            SectorNodeStatus,
            SectorState,
            TravelOutcome,
        },
    },
    ship::ModuleKind,
};

pub(crate) fn update_mission_telemetry(
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    module_query: Query<(
        &RuntimeShipModule,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
) {
    let mut mission_state = mission_query.into_inner();

    for (runtime_module, runtime_state, destroyed) in &module_query {
        if destroyed.is_some() {
            continue;
        }
        if runtime_state.current_heat > mission_state.highest_heat {
            mission_state.highest_heat = runtime_state.current_heat;
            mission_state.hottest_module_kind = Some(runtime_module.kind);
        }
        if mission_state.first_disabled_module_kind.is_none()
            && !runtime_state.was_disabled_last_frame
            && runtime_state.is_disabled
        {
            mission_state.first_disabled_module_kind = Some(runtime_module.kind);
        }
    }
}

pub(crate) fn update_mission_state(
    hostile_query: Query<Entity, With<HostileTarget>>,
    processor_query: Query<&ProcessorModule>,
    salvage_query: Query<Entity, With<SalvageWreck>>,
    player_ship_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    let mut mission_state = player_ship_query.into_inner();
    if mission_state.failed {
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state
            .return_delay_remaining
            .get_or_insert(Fx::from_num(2.5));
        return;
    }

    if hostile_query.is_empty() {
        mission_state.encounter_cleared = true;
        let requires_processing = !processor_query.is_empty();
        let salvage_required = !salvage_query.is_empty();
        mission_state.completed = if salvage_required {
            mission_state.salvage_collected
                && (!requires_processing || mission_state.processed_repair_charge > 0)
        } else {
            true
        };
        if mission_state.completion_reason.is_none() {
            mission_state.completion_reason = Some(if salvage_required {
                "Encounter cleared".to_string()
            } else {
                "Test route complete".to_string()
            });
        }
        if mission_state.completed {
            mission_state
                .return_delay_remaining
                .get_or_insert(Fx::from_num(2.5));
        } else {
            mission_state.return_delay_remaining = None;
        }
    } else {
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state.completion_reason = None;
        mission_state.return_delay_remaining = None;
    }
}

pub(crate) fn sync_runtime_ship_state(
    balance: Res<BalanceConfig>,
    player_ship_query: Single<
        (
            &Children,
            &ShipAutomationState,
            &mut ShipMovementModel,
            &mut ShipPowerModel,
            &mut ShipWeaponState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&ReactorCommandState>,
        Option<&DestroyedModule>,
        Option<&WeaponModule>,
    )>,
) {
    let (
        children,
        automation_state,
        mut movement_model,
        mut power_model,
        mut weapon_state,
        mut mission_state,
    ) = player_ship_query.into_inner();

    let mut live_modules = 0usize;
    let mut engine_count = 0u32;
    let mut reactor_count = 0u32;
    let mut battery_count = 0u32;
    let mut turret_count = 0u32;
    let mut core_alive = false;
    let mut effective_engines = Fx::from_num(0);
    let mut effective_reactors = Fx::from_num(0);
    let mut effective_reactor_output = Fx::from_num(0);
    let mut effective_batteries = Fx::from_num(0);
    let mut effective_turrets = Fx::from_num(0);

    for child in children.iter() {
        let Ok((runtime_module, integrity, runtime_state, reactor_state, destroyed, weapon_module)) =
            module_query.get(*child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }

        live_modules += 1;
        let effectiveness = module_effectiveness(integrity, runtime_state, false);
        match runtime_module.kind {
            ModuleKind::Core => core_alive = true,
            ModuleKind::Engine => {
                engine_count += 1;
                effective_engines += effectiveness;
            }
            ModuleKind::Reactor => {
                reactor_count += 1;
                effective_reactors += effectiveness;
                if let Some(reactor_state) = reactor_state {
                    effective_reactor_output += reactor_state.power_output * effectiveness;
                }
            }
            ModuleKind::Battery => {
                battery_count += 1;
                effective_batteries += effectiveness;
            }
            ModuleKind::Turret => {
                turret_count += 1;
                if weapon_module.is_some() && !runtime_state.is_disabled {
                    effective_turrets += effectiveness;
                }
            }
            _ => {}
        }
    }

    *movement_model = ship_movement_model_with_effective(
        live_modules.max(1),
        engine_count,
        effective_engines,
        &balance,
    );
    *power_model = ship_power_model_with_effective(
        live_modules.max(1),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
        effective_reactors,
        effective_batteries,
        effective_engines,
        effective_turrets,
        &balance,
    );
    if reactor_count > 0 {
        power_model.reactor_output = effective_reactor_output.max(
            Fx::from_num(reactor_count)
                * Fx::from_num(balance.ship.reactor_output_floor_per_reactor),
        );
    }
    power_model.reactor_output *= automation_state.output_scale;
    weapon_state.turret_count = effective_turrets.to_num::<u32>();
    if weapon_state.turret_count == 0 {
        weapon_state.cooldown_remaining = Fx::from_num(0);
    }

    if !core_alive {
        mission_state.failed = true;
        mission_state.failure_reason = Some("Core destroyed".to_string());
        mission_state.encounter_cleared = false;
        mission_state.completed = false;
        mission_state
            .return_delay_remaining
            .get_or_insert(Fx::from_num(2.5));
    }
}

pub(crate) fn return_after_mission_resolution(
    time: Res<Time>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut progression: ResMut<DemoProgression>,
    mut sector_state: ResMut<SectorState>,
    inventory_query: Query<(
        &RuntimeShipModule,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
    )>,
    computer_query: Query<&RuntimeArchComputer>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    let mut mission_state = mission_query.into_inner();
    let Some(delay) = mission_state.return_delay_remaining.as_mut() else {
        return;
    };

    *delay = (*delay - fx_from_time_delta(&time)).max(Fx::from_num(0));
    if *delay > Fx::from_num(0) {
        return;
    }

    let mut raw_salvage_returned = 0u32;
    let mut repair_charge_returned = 0u32;
    for (_, storage, processor) in &inventory_query {
        if let Some(storage) = storage {
            raw_salvage_returned += storage.inventory.raw_salvage;
            repair_charge_returned += storage.inventory.repair_charge;
        }
        if let Some(processor) = processor {
            raw_salvage_returned += processor.inventory.raw_salvage;
            repair_charge_returned += processor.inventory.repair_charge;
        }
    }
    let logistics_payout = (raw_salvage_returned + repair_charge_returned * 3)
        * mission_state.reward_multiplier.max(1);
    progression.scrap += logistics_payout;
    mission_state.salvage_scrap_awarded = logistics_payout;

    let hull_wear_delta = if mission_state.failed {
        3
    } else if mission_state.first_disabled_module_kind.is_some() {
        2
    } else if mission_state.highest_heat >= Fx::from_num(10) {
        1
    } else {
        0
    };
    progression.hull_wear = progression.hull_wear.saturating_add(hull_wear_delta);

    let (headline, detail) = if mission_state.failed {
        (
            "Mission Failed".to_string(),
            mission_state
                .failure_reason
                .clone()
                .unwrap_or_else(|| "The ship was lost.".to_string()),
        )
    } else {
        let atmosphere_suffix = if mission_state.hostile_decompression_events > 0
            || mission_state.player_ship_breached
        {
            format!(
                " Atmosphere: hostile ships vented {}, own breaches {}, lowest player oxygen {}.",
                mission_state.hostile_decompression_events,
                if mission_state.player_ship_breached {
                    "yes"
                } else {
                    "no"
                },
                mission_state.lowest_player_oxygen.to_num::<f32>().round()
            )
        } else {
            String::new()
        };
        let detail = if mission_state.salvage_collected {
            format!(
                "Recovered {} raw salvage and returned {} repair charge worth {} scrap.{}",
                mission_state.recovered_raw_salvage,
                repair_charge_returned,
                mission_state.salvage_scrap_awarded,
                atmosphere_suffix
            )
        } else {
            format!("Encounter cleared, but no salvage was recovered.{atmosphere_suffix}")
        };
        ("Mission Complete".to_string(), detail)
    };

    last_mission_report.headline = Some(headline);
    last_mission_report.detail = Some(detail);
    last_mission_report.scrap_awarded = mission_state.salvage_scrap_awarded;
    last_mission_report.total_scrap = progression.scrap;
    last_mission_report.hottest_module = mission_state
        .hottest_module_kind
        .map(|kind| kind.as_str().to_string());
    last_mission_report.first_disabled_module = mission_state
        .first_disabled_module_kind
        .map(|kind| kind.as_str().to_string());
    last_mission_report.repairs_performed = mission_state.repairs_performed;
    last_mission_report.stabilizations_performed = mission_state.stabilizations_performed;
    last_mission_report.automation_used = mission_state.automation_used;
    last_mission_report.automation_triggers = mission_state.automation_trigger_count;
    last_mission_report.recovered_raw_salvage = mission_state.recovered_raw_salvage;
    last_mission_report.processed_repair_charge = mission_state.processed_repair_charge;
    last_mission_report.consumed_repair_charge = mission_state.consumed_repair_charge;
    last_mission_report.transfer_count = mission_state.transfer_count;
    last_mission_report.processor_cycles = mission_state.processor_cycles;
    last_mission_report.logistics_bottleneck = mission_state.logistics_bottleneck.clone();
    last_mission_report.logistics_automation_used = mission_state.logistics_automation_used;
    last_mission_report.arch_primary_program = automation_program_name(&computer_query);
    last_mission_report.arch_invalid_executions = computer_query
        .iter()
        .filter(|computer| computer.last_result.halted_reason.is_some())
        .count() as u32;
    last_mission_report.arch_recent_writes = computer_query
        .iter()
        .flat_map(|computer| computer.last_result.recent_writes.clone())
        .take(4)
        .collect();
    let mut hints = Vec::new();
    if mission_state.hottest_module_kind == Some(ModuleKind::Reactor) {
        hints.push("Reactor ran hottest. Consider more spacing or cooler hull nearby.".to_string());
    }
    if mission_state.first_disabled_module_kind == Some(ModuleKind::Turret) {
        hints.push("Turret failed first. Consider better protection or easier access.".to_string());
    }
    if mission_state.repairs_performed + mission_state.stabilizations_performed >= 3 {
        hints.push("High intervention count. Interior access may be too awkward.".to_string());
    }
    if mission_state.automation_used {
        hints.push("Automation reduced reactor babysitting during the fight.".to_string());
    }
    if mission_state.logistics_bottleneck.is_some() {
        hints.push("Logistics stalled during the run. Intake, storage, or processing may be poorly placed.".to_string());
    }
    if mission_state.processor_cycles == 0 && mission_state.recovered_raw_salvage > 0 {
        hints.push("Recovered cargo never reached a useful processor cycle.".to_string());
    }
    if mission_state.transfer_count <= 1 && mission_state.recovered_raw_salvage > 0 {
        hints.push(
            "Cargo flow barely moved. Manipulator reach or automation priority may be wrong."
                .to_string(),
        );
    }
    if mission_state.player_ship_breached {
        hints.push(
            "Player ship vented during the run. Airlock placement or edge protection may be weak."
                .to_string(),
        );
    }
    if mission_state.hostile_decompression_events > 0 {
        hints.push(
            "Hostile decompression proved useful. Breach access and boarding routing are becoming tactical."
                .to_string(),
        );
    }
    if mission_state.lowest_player_oxygen <= Fx::from_num(3) {
        hints.push(
            "Player oxygen dipped critically low. Compartments or seal control may need improvement."
                .to_string(),
        );
    }
    last_mission_report.redesign_hints = hints;
    last_mission_report.node_name = Some(mission_state.node_name.clone());
    last_mission_report.node_kind = Some(mission_state.node_kind_name.clone());

    let travel_outcome = TravelOutcome {
        node_id: mission_state.node_id,
        success: !mission_state.failed,
        failed: mission_state.failed,
        scrap_awarded: mission_state.salvage_scrap_awarded,
        hull_wear_delta,
        exhausted: !mission_state.failed,
    };
    last_mission_report.travel_outcome = Some(if travel_outcome.failed {
        format!(
            "{} failed, +{} hull wear",
            mission_state.node_name, travel_outcome.hull_wear_delta
        )
    } else {
        format!(
            "{} cleared, +{} scrap, +{} hull wear",
            mission_state.node_name, travel_outcome.scrap_awarded, travel_outcome.hull_wear_delta
        )
    });

    if let Some(node) = sector_state.node_mut(mission_state.node_id) {
        node.status = if travel_outcome.failed {
            SectorNodeStatus::Failed
        } else {
            SectorNodeStatus::Exhausted
        };
    }
    sector_state.current_node_id = 0;
    sector_state.active_encounter_node_id = None;
    if let Some(next_node) = sector_state.available_neighbors().into_iter().find(|node| {
        matches!(
            node.status,
            SectorNodeStatus::Fresh | SectorNodeStatus::Completed
        )
    }) {
        sector_state.selected_node_id = Some(next_node.id);
    }

    mission_state.return_delay_remaining = None;
    next_state.set(ClientAppState::Docked);
}

fn automation_program_name(computer_query: &Query<&RuntimeArchComputer>) -> Option<String> {
    computer_query
        .iter()
        .find(|computer| computer.enabled)
        .map(|computer| computer.program.name.clone())
}
