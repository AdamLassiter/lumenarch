use bevy::prelude::*;

use crate::client::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            AngularVelocity,
            CarriedResource,
            CollectedSalvage,
            CurrentStation,
            DestroyedModule,
            HostileShip,
            HostileTarget,
            Integrity,
            LinearVelocity,
            ManipulatorCommandState,
            ManipulatorModule,
            MissionState,
            ModuleCondition,
            ModuleRuntimeState,
            PlayerFieldState,
            PlayerMotionState,
            PlayerReferenceFrame,
            PlayerShip,
            ProcessorCommandState,
            ProcessorModule,
            Projectile,
            ReactorCommandState,
            RuntimeArchComputer,
            RuntimeShipModule,
            SalvagePickup,
            SalvageWreck,
            ShipAutomationState,
            ShipControlState,
            ShipMovementModel,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            ShipboardControlState,
            ShipboardPlayer,
            SimPosition,
            StorageCommandState,
            StorageModule,
            TurretCommandState,
            WeaponModule,
        },
        helpers::{
            Fx,
            danger_level,
            format_fx0,
            format_fx1,
            format_fx2,
            meter_bar,
            mission_return_line,
            mission_status_line,
            module_condition,
            salvage_status_line,
        },
        systems::shared::module_display_name,
    },
    state::{DemoProgression, GameplayControlsText, GameplayStatusText},
};

pub(crate) fn update_gameplay_status_text(
    balance: Res<BalanceConfig>,
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
            &ShipControlState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<
        (
            &CurrentStation,
            &PlayerFieldState,
            &PlayerMotionState,
            &CarriedResource,
        ),
        With<ShipboardPlayer>,
    >,
    ship_identity_query: Query<(Entity, Option<&PlayerShip>, Option<&HostileShip>), With<ShipRoot>>,
    module_parent_query: Query<&Parent, With<RuntimeShipModule>>,
    hostile_query: Query<Entity, With<HostileTarget>>,
    projectile_query: Query<Entity, With<Projectile>>,
    module_query: Query<
        (
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
    mut status_query: Query<&mut Text, (With<GameplayStatusText>, Without<GameplayControlsText>)>,
    mut controls_query: Query<&mut Text, (With<GameplayControlsText>, Without<GameplayStatusText>)>,
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
        ship_controls,
    ) = ship_query.into_inner();
    let (current_station, player_fields, player_motion, carried_resource) =
        player_query.into_inner();
    let frame_label = reference_frame_label(player_motion, &ship_identity_query);
    let focused_station_context = control_mode
        .focused_entity
        .and_then(|entity| module_parent_query.get(entity).ok())
        .map(|parent| ship_affiliation_label(parent.get(), &ship_identity_query))
        .unwrap_or("Unattached");

    let salvage_line = salvage_status_line(
        player_motion.world_position,
        mission_state,
        &salvage_query,
        balance.combat.salvage_pickup_radius,
    );
    let (current_integrity, max_integrity, active_modules, degraded_modules, disabled_modules) =
        summarize_modules(children, &module_query, &balance);
    let (arch_program, arch_exec, arch_invalid, arch_writes) =
        summarize_arch(children, &module_query);

    for mut text in &mut status_query {
        let status_line = match mission_return_line(mission_state) {
            Some(return_line) => format!("{}\n{}", mission_status_line(mission_state), return_line),
            None => mission_status_line(mission_state).to_string(),
        };
        **text = format!(
            "Mission Status\nOutcome: {}\nMode: {}\nFocus: {}\nFrame: {}\nFocused Context: {}\nStation: {}\nPlayer Position: {}, {}\nPlayer Velocity: {}\nShip Position: {}, {}\nShip Velocity: {}\nTurn Rate: {}\nIntegrity\nHull / Systems: {} / {}\nActive Modules: {}\nDegraded / Disabled: {} / {}\nPower\nEngine Output: {} ({}%)\nGeneration / Draw: {} / {}\nBattery Reserve: {}\nWeapons Online: {}\nARCH\nMode: {:?}\nStatus: {}\nProgram: {}\nExec / Invalid: {} / {}\nWrites: {}\nCombat\nTurrets: {}  Cooldown: {}\nProjectiles: {}  Hostiles: {}\nLogistics\nRecovered Raw: {}\nProcessed / Used Charges: {} / {}\nTransfers / Cycles: {} / {}\nBottleneck: {}\nInterventions\nRepairs / Stabilizations: {} / {}\nRecent: {}\nHelm\nThrottle: {}%\nTurn Demand: {}\nInterior\nLocal Heat: {} {} ({})\nLocal Electrical: {} {} ({})\nCarried Cargo: {}\nSalvage: {}\nScrap Total: {}",
            status_line,
            control_mode.mode.as_str(),
            control_mode
                .focused_family
                .map(|family| family.as_str())
                .unwrap_or("None"),
            frame_label,
            focused_station_context,
            module_display_name(current_station.kind),
            format_fx0(player_motion.world_position.x),
            format_fx0(player_motion.world_position.y),
            format_fx1(player_motion.world_velocity.length()),
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
            automation_state.mode,
            if automation_state.active {
                "engaged"
            } else {
                "standby"
            },
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
            format_fx0(ship_controls.throttle_demand * Fx::from_num(100)),
            format_fx1(ship_controls.turn_input),
            format_fx1(player_fields.local_heat),
            meter_bar(player_fields.local_heat, Fx::from_num(16), 12),
            danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14)),
            format_fx1(player_fields.local_electrical),
            meter_bar(player_fields.local_electrical, Fx::from_num(14), 12),
            danger_level(
                player_fields.local_electrical,
                Fx::from_num(7),
                Fx::from_num(12)
            ),
            carried_resource
                .kind
                .map(|kind| format!(
                    "{} {}",
                    carried_resource.amount,
                    crate::client::gameplay::helpers::resource_kind_label(kind)
                ))
                .unwrap_or_else(|| "none".to_string()),
            salvage_line,
            progression.scrap,
        );
    }

    let controls_text = control_mode
        .focused_entity
        .and_then(|entity| module_query.get(entity).ok())
        .map(
            |(
                runtime_module,
                _,
                runtime_state,
                computer,
                storage,
                storage_command,
                manipulator,
                manipulator_command,
                processor,
                processor_command,
                reactor,
                turret,
                destroyed,
                _,
            )| {
                if destroyed.is_some() {
                    return format!(
                        "Focused Station\nContext: {}\nDestroyed module",
                        focused_station_context
                    );
                }
                if let Some(reactor) = reactor {
                    return format!(
                        "Focused Station\nContext: {}\nReactor Controls\nRate: {}%\nTurbine: {}%\nPower Output: {}\nFuel: {}\nHeat: {}\nControls: W/S reaction, A/D turbine, Q leave",
                        focused_station_context,
                        format_fx0(reactor.reaction_rate * Fx::from_num(100)),
                        format_fx0(reactor.turbine_load * Fx::from_num(100)),
                        format_fx1(reactor.power_output),
                        format_fx0(reactor.fuel_remaining),
                        format_fx1(runtime_state.current_heat),
                    );
                }
                if let Some(turret) = turret {
                    return format!(
                        "Focused Station\nContext: {}\nTurret Controls\nDesired Angle: {}\nActual Angle: {}\nFire Gate: {}\nCooldown: {}\nControls: mouse or A/D aim, Space or left mouse fire, Q leave",
                        focused_station_context,
                        format_fx1(turret.desired_angle),
                        format_fx1(turret.actual_angle),
                        if turret.fire_intent { "open" } else { "hold" },
                        format_fx2(weapon_state.cooldown_remaining),
                    );
                }
                if let Some(storage) = storage {
                    return format!(
                        "Focused Station\nContext: {}\nStorage Panel\nRaw: {}  Charge: {}\nFill: {}/{}\nIntake: {}\nControls: Space toggles intake, F deposit/extract, G drop cargo, Q leave",
                        focused_station_context,
                        storage.inventory.raw_salvage,
                        storage.inventory.repair_charge,
                        storage.inventory.total_units(),
                        storage.capacity,
                        if storage_command.is_some_and(|command| command.allow_intake) {
                            "open"
                        } else {
                            "closed"
                        },
                    );
                }
                if let Some(manipulator) = manipulator {
                    return format!(
                        "Focused Station\nContext: {}\nManipulator Panel\nManual: {}\nTarget: {}\nResource: {:?}\nArmed: {}\nStatus: {}\nControls: M manual, [/] target, R resource, Space arm, Q leave",
                        focused_station_context,
                        if manipulator_command.is_some_and(|command| command.manual_mode) {
                            "yes"
                        } else {
                            "no"
                        },
                        manipulator_command
                            .and_then(|command| command.target_module_id)
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        manipulator_command
                            .map(|command| command.resource_kind)
                            .unwrap_or(crate::client::gameplay::components::ResourceKind::RawSalvage),
                        if manipulator_command.is_some_and(|command| command.transfer_enabled) {
                            "yes"
                        } else {
                            "no"
                        },
                        manipulator.blocked_reason.as_deref().unwrap_or("ready"),
                    );
                }
                if let Some(processor) = processor {
                    let progress_pct = if processor.duration > Fx::from_num(0) {
                        processor.progress / processor.duration * Fx::from_num(100)
                    } else {
                        Fx::from_num(0)
                    };
                    return format!(
                        "Focused Station\nContext: {}\nProcessor Panel\nRecipe: {}\nEnabled: {}\nProgress: {}%\nRaw: {}  Charge: {}\nState: {}\nControls: Space enable, R recipe, Q leave",
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
                    );
                }
                if let Some(computer) = computer {
                    return format!(
                        "Focused Station\nContext: {}\nComputer Console\nProgram: {}\nOnline: {}\nExec: {}/{}\nLast: {}\nControls: Space power, T cycle template, Q leave",
                        focused_station_context,
                        computer.program.name,
                        if computer.enabled { "yes" } else { "no" },
                        computer.last_result.executed,
                        computer.last_result.budget,
                        computer.last_result.halted_reason.as_deref().unwrap_or("ok"),
                    );
                }
                format!(
                    "Focused Station\nContext: {}\n{} ready",
                    focused_station_context,
                    runtime_module.kind.as_str()
                )
            },
        )
        .unwrap_or_else(|| controls_help_text(control_mode.mode));

    for mut text in &mut controls_query {
        **text = controls_text.clone();
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
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
    balance: &BalanceConfig,
) -> (i32, i32, usize, usize, usize) {
    let mut current_integrity = 0i32;
    let mut max_integrity = 0i32;
    let mut active_modules = 0usize;
    let mut degraded_modules = 0usize;
    let mut disabled_modules = 0usize;

    for child in children.iter() {
        let Ok((_, integrity, runtime_state, _, _, _, _, _, _, _, _, _, destroyed, _)) =
            module_query.get(*child)
        else {
            continue;
        };
        max_integrity += integrity.max;
        let condition = module_condition(integrity, runtime_state, destroyed.is_some(), balance);
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
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&DestroyedModule>,
            Option<&WeaponModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> (String, String, u32, String) {
    for child in children.iter() {
        let Ok((_, _, _, computer, _, _, _, _, _, _, _, _, destroyed, _)) =
            module_query.get(*child)
        else {
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
                format!(
                    "{}/{}",
                    computer.last_result.executed, computer.last_result.budget
                ),
                u32::from(computer.last_result.halted_reason.is_some()),
                if computer.last_result.recent_writes.is_empty() {
                    "none".to_string()
                } else {
                    computer.last_result.recent_writes.join(", ")
                },
            );
        }
    }

    ("none".to_string(), "0/0".to_string(), 0, "none".to_string())
}

fn controls_help_text(mode: crate::client::gameplay::components::ShipControlMode) -> String {
    match mode {
        crate::client::gameplay::components::ShipControlMode::Interior => {
            "Controls\nW/A/S/D: move continuously or EVA thrust\nE: enter nearby station\nF: pick up or deposit cargo\nG: drop carried cargo\nQ or Esc: leave focused station\nF3: diagnostics\nTab: return to station".to_string()
        }
        crate::client::gameplay::components::ShipControlMode::Cockpit => {
            "Cockpit Controls\nW/S or mouse: throttle demand\nA/D or mouse: steering\nQ or Esc: leave cockpit".to_string()
        }
        crate::client::gameplay::components::ShipControlMode::Turret => {
            "Turret Controls\nMouse or A/D: aim turret\nSpace or left mouse: fire\nQ or Esc: leave turret".to_string()
        }
        crate::client::gameplay::components::ShipControlMode::Reactor => {
            "Reactor Controls\nW/S: reaction rate\nA/D: turbine load\nQ or Esc: leave reactor".to_string()
        }
        crate::client::gameplay::components::ShipControlMode::Logistics => {
            "Logistics Controls\nStorage: Space toggles intake\nManipulator: M manual, [/] target, R resource, Space arm\nProcessor: Space enable, R recipe\nQ or Esc: leave panel".to_string()
        }
        crate::client::gameplay::components::ShipControlMode::Computer => {
            "Computer Controls\nSpace: online/offline\nT: cycle starter template\nQ or Esc: leave console".to_string()
        }
    }
}

fn reference_frame_label(
    player_motion: &PlayerMotionState,
    ship_identity_query: &Query<
        (Entity, Option<&PlayerShip>, Option<&HostileShip>),
        With<ShipRoot>,
    >,
) -> String {
    match player_motion.frame {
        PlayerReferenceFrame::World => "EVA / World".to_string(),
        PlayerReferenceFrame::Ship(ship_entity) => {
            format!(
                "Ship Local ({})",
                ship_affiliation_label(ship_entity, ship_identity_query)
            )
        }
    }
}

fn ship_affiliation_label(
    ship_entity: Entity,
    ship_identity_query: &Query<
        (Entity, Option<&PlayerShip>, Option<&HostileShip>),
        With<ShipRoot>,
    >,
) -> &'static str {
    ship_identity_query
        .get(ship_entity)
        .map(|(_, player_ship, hostile_ship)| {
            if player_ship.is_some() {
                "Player Ship"
            } else if hostile_ship.is_some() {
                "Hostile Ship"
            } else {
                "Unmarked Ship"
            }
        })
        .unwrap_or("Unknown Ship")
}
