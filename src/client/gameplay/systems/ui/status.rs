use bevy::{ecs::system::SystemParam, prelude::*};

use crate::client::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            AirlockCommandState,
            AngularVelocity,
            CarriedResource,
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
            ShipAtmosphereState,
            ShipAutomationState,
            ShipControlMode,
            ShipControlState,
            ShipMovementModel,
            ShipPowerModel,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            ShipboardControlState,
            ShipboardPlayer,
            SimPosition,
            StorageCommandState,
            StorageModule,
            TurretCommandState,
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
            resource_kind_label,
        },
        systems::shared::module_display_name,
    },
    state::{
        DebugOverlayState,
        DemoProgression,
        GameplayBarFill,
        GameplayBarKind,
        GameplayBarLabel,
        GameplayCompactStatusText,
        GameplayControlsText,
        GameplayPanelBodyText,
        GameplayPanelTitleText,
        GameplayStationPanel,
        GameplayStationPanelButton,
        GameplayStationPanelButtonLabel,
        GameplayTopBannerText,
        StationPanelButtonAction,
    },
};

pub(crate) fn update_gameplay_status_text(
    balance: Res<BalanceConfig>,
    debug_overlay: Res<DebugOverlayState>,
    ship_query: Single<
        (
            &SimPosition,
            &Children,
            &LinearVelocity,
            &AngularVelocity,
            &ShipAutomationState,
            &ShipMovementModel,
            &ShipPowerState,
            &ShipPowerModel,
            &ShipWeaponState,
            &MissionState,
            &ShipAtmosphereState,
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
    status_world: GameplayStatusWorldQueries,
    progression: Res<DemoProgression>,
    mut hud_ui: GameplayHudUiQueries,
) {
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        _automation_state,
        _movement_model,
        power_state,
        power_model,
        weapon_state,
        mission_state,
        atmosphere_state,
        control_mode,
        _ship_controls,
    ) = ship_query.into_inner();
    let (current_station, player_fields, player_motion, carried_resource) =
        player_query.into_inner();

    let frame_label = reference_frame_label(player_motion, &status_world.ship_identity_query);
    let focused_station_context = control_mode
        .focused_entity
        .and_then(|entity| status_world.module_parent_query.get(entity).ok())
        .map(|parent| ship_affiliation_label(parent.get(), &status_world.ship_identity_query))
        .unwrap_or("Unattached");

    let (current_integrity, max_integrity, active_modules, degraded_modules, disabled_modules) =
        summarize_modules(children, &status_world.module_query, &balance);
    let arch_summary = summarize_arch(children, &status_world.module_query);
    let top_banner = build_top_banner(
        mission_state,
        weapon_state,
        status_world.hostile_query.iter().len(),
        status_world.projectile_query.iter().len(),
    );
    let compact = build_compact_status(
        player_motion,
        ship_position,
        linear_velocity,
        angular_velocity,
        control_mode,
        current_station,
        frame_label.as_str(),
        focused_station_context,
        current_integrity,
        max_integrity,
        active_modules,
        degraded_modules,
        disabled_modules,
        atmosphere_state,
        carried_resource,
        mission_state,
        progression.scrap,
        &arch_summary,
    );
    let controls_text = controls_help_text(control_mode.mode);
    let (panel_title, panel_body, active_station_kind, active_station_flags) =
        station_panel_content(
            control_mode,
            mission_state,
            weapon_state,
            player_fields,
            player_motion,
            &status_world.module_query,
            focused_station_context,
            &arch_summary,
        );
    let show_station_panel =
        control_mode.mode != ShipControlMode::Interior || debug_overlay.enabled;

    for mut text in &mut hud_ui.text_queries.p0() {
        **text = top_banner.clone();
    }
    for mut text in &mut hud_ui.text_queries.p1() {
        **text = compact.clone();
    }
    for mut text in &mut hud_ui.text_queries.p2() {
        **text = controls_text.clone();
    }
    for mut text in &mut hud_ui.text_queries.p3() {
        **text = panel_title.clone();
    }
    for mut text in &mut hud_ui.text_queries.p4() {
        **text = panel_body.clone();
    }

    let hull_pct = percent(current_integrity as f32, max_integrity.max(1) as f32);
    let power_pct = if power_state.draw > Fx::from_num(0) {
        (power_state.generation / power_state.draw * Fx::from_num(100))
            .clamp(Fx::from_num(0), Fx::from_num(100))
            .to_num::<f32>()
    } else {
        100.0
    };
    let battery_pct = if power_model.battery_capacity > Fx::from_num(0) {
        (power_state.stored_energy / power_model.battery_capacity * Fx::from_num(100))
            .clamp(Fx::from_num(0), Fx::from_num(100))
            .to_num::<f32>()
    } else {
        0.0
    };
    let oxygen_pct = (atmosphere_state.average_oxygen
        / Fx::from_num(balance.atmosphere.max_tile_oxygen)
        * Fx::from_num(100))
    .clamp(Fx::from_num(0), Fx::from_num(100))
    .to_num::<f32>();
    let heat_pct = (player_fields.local_heat / Fx::from_num(16) * Fx::from_num(100))
        .clamp(Fx::from_num(0), Fx::from_num(100))
        .to_num::<f32>();
    let electrical_pct = (player_fields.local_electrical / Fx::from_num(12) * Fx::from_num(100))
        .clamp(Fx::from_num(0), Fx::from_num(100))
        .to_num::<f32>();

    for (bar, mut node) in &mut hud_ui.node_queries.p0() {
        let pct = match bar.kind {
            GameplayBarKind::Hull => hull_pct,
            GameplayBarKind::Power => power_pct,
            GameplayBarKind::Battery => battery_pct,
            GameplayBarKind::Oxygen => oxygen_pct,
            GameplayBarKind::Heat => heat_pct,
            GameplayBarKind::Electrical => electrical_pct,
        };
        node.width = Val::Percent(pct);
    }

    for (label, mut text) in &mut hud_ui.text_queries.p5() {
        **text = match label.kind {
            GameplayBarKind::Hull => format!("Hull {} / {}", current_integrity, max_integrity),
            GameplayBarKind::Power => format!(
                "Power {} / {}",
                format_fx1(power_state.generation),
                format_fx1(power_state.draw)
            ),
            GameplayBarKind::Battery => format!(
                "Battery {} / {}",
                format_fx1(power_state.stored_energy),
                format_fx1(power_model.battery_capacity)
            ),
            GameplayBarKind::Oxygen => format!(
                "Oxygen {} avg / {} min",
                format_fx1(atmosphere_state.average_oxygen),
                format_fx1(atmosphere_state.minimum_oxygen)
            ),
            GameplayBarKind::Heat => format!(
                "Heat {} ({})",
                format_fx1(player_fields.local_heat),
                danger_level(player_fields.local_heat, Fx::from_num(8), Fx::from_num(14))
            ),
            GameplayBarKind::Electrical => format!(
                "Electrical {} ({})",
                format_fx1(player_fields.local_electrical),
                danger_level(
                    player_fields.local_electrical,
                    Fx::from_num(7),
                    Fx::from_num(12)
                )
            ),
        };
    }

    for mut node in &mut hud_ui.node_queries.p2() {
        node.display = if show_station_panel {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (button, mut node, mut color, children) in &mut hud_ui.node_queries.p1() {
        let visible = station_action_visible(
            button.action,
            control_mode.mode,
            active_station_kind,
            active_station_flags,
        );
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *color = BackgroundColor(if visible {
            Color::srgb(0.24, 0.38, 0.58)
        } else {
            Color::srgba(0.24, 0.38, 0.58, 0.18)
        });
        for child in children.iter() {
            if let Ok((label, mut text)) = hud_ui.text_queries.p6().get_mut(*child) {
                **text =
                    station_button_label(label.action, control_mode.mode, active_station_flags);
            }
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct GameplayStatusWorldQueries<'w, 's> {
    ship_identity_query: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static PlayerShip>,
            Option<&'static HostileShip>,
        ),
        With<ShipRoot>,
    >,
    module_parent_query: Query<'w, 's, &'static Parent, With<RuntimeShipModule>>,
    hostile_query: Query<'w, 's, Entity, With<HostileTarget>>,
    projectile_query: Query<'w, 's, Entity, With<Projectile>>,
    module_query: Query<
        'w,
        's,
        (
            Entity,
            &'static RuntimeShipModule,
            &'static Integrity,
            &'static ModuleRuntimeState,
            Option<&'static RuntimeArchComputer>,
            Option<&'static StorageModule>,
            Option<&'static StorageCommandState>,
            Option<&'static ManipulatorModule>,
            Option<&'static ManipulatorCommandState>,
            Option<&'static ProcessorModule>,
            Option<&'static ProcessorCommandState>,
            Option<&'static ReactorCommandState>,
            Option<&'static TurretCommandState>,
            Option<&'static AirlockCommandState>,
            Option<&'static DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
}

#[derive(SystemParam)]
pub(crate) struct GameplayHudUiQueries<'w, 's> {
    text_queries: ParamSet<
        'w,
        's,
        (
            Query<'w, 's, &'static mut Text, With<GameplayTopBannerText>>,
            Query<'w, 's, &'static mut Text, With<GameplayCompactStatusText>>,
            Query<'w, 's, &'static mut Text, With<GameplayControlsText>>,
            Query<'w, 's, &'static mut Text, With<GameplayPanelTitleText>>,
            Query<'w, 's, &'static mut Text, With<GameplayPanelBodyText>>,
            Query<'w, 's, (&'static GameplayBarLabel, &'static mut Text)>,
            Query<'w, 's, (&'static GameplayStationPanelButtonLabel, &'static mut Text)>,
        ),
    >,
    node_queries: ParamSet<
        'w,
        's,
        (
            Query<'w, 's, (&'static GameplayBarFill, &'static mut Node)>,
            Query<
                'w,
                's,
                (
                    &'static GameplayStationPanelButton,
                    &'static mut Node,
                    &'static mut BackgroundColor,
                    &'static Children,
                ),
                With<Button>,
            >,
            Query<'w, 's, &'static mut Node, With<GameplayStationPanel>>,
        ),
    >,
}

pub(crate) fn station_panel_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &GameplayStationPanelButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    ship_query: Single<
        (
            &mut ShipboardControlState,
            &mut ShipControlState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    candidate_query: Query<&RuntimeShipModule>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        Option<&mut TurretCommandState>,
        Option<&mut ReactorCommandState>,
        Option<&mut StorageCommandState>,
        Option<&mut ManipulatorCommandState>,
        Option<&mut ProcessorCommandState>,
        Option<&mut AirlockCommandState>,
        Option<&mut RuntimeArchComputer>,
    )>,
) {
    let (control_state, mut ship_controls, mut mission_state) = ship_query.into_inner();

    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.20, 0.31, 0.48));
                let Some(focused_entity) = control_state.focused_entity else {
                    continue;
                };
                let Ok((
                    _,
                    runtime_module,
                    turret_state,
                    reactor_state,
                    storage_state,
                    manipulator_state,
                    processor_state,
                    airlock_state,
                    arch_runtime,
                )) = module_query.get_mut(focused_entity)
                else {
                    continue;
                };

                match button.action {
                    StationPanelButtonAction::HelmThrottle { delta } => {
                        if control_state.mode == ShipControlMode::Cockpit {
                            ship_controls.throttle_demand = (ship_controls.throttle_demand
                                + Fx::from_num(delta))
                            .clamp(Fx::from_num(0), Fx::from_num(1));
                        }
                    }
                    StationPanelButtonAction::HelmTurn { value } => {
                        if control_state.mode == ShipControlMode::Cockpit {
                            ship_controls.turn_input =
                                Fx::from_num(value).clamp(Fx::from_num(-1), Fx::from_num(1));
                        }
                    }
                    StationPanelButtonAction::TurretAdjustAim { delta } => {
                        if control_state.mode == ShipControlMode::Turret
                            && let Some(mut turret_state) = turret_state
                        {
                            turret_state.desired_angle += Fx::from_num(delta);
                        }
                    }
                    StationPanelButtonAction::TurretFireToggle => {
                        if control_state.mode == ShipControlMode::Turret
                            && let Some(mut turret_state) = turret_state
                        {
                            turret_state.fire_intent = !turret_state.fire_intent;
                        }
                    }
                    StationPanelButtonAction::ReactorAdjustRate { delta } => {
                        if control_state.mode == ShipControlMode::Reactor
                            && let Some(mut reactor_state) = reactor_state
                        {
                            reactor_state.reaction_rate = (reactor_state.reaction_rate
                                + Fx::from_num(delta))
                            .clamp(Fx::from_num(0), Fx::from_num(1));
                        }
                    }
                    StationPanelButtonAction::ReactorAdjustTurbine { delta } => {
                        if control_state.mode == ShipControlMode::Reactor
                            && let Some(mut reactor_state) = reactor_state
                        {
                            reactor_state.turbine_load = (reactor_state.turbine_load
                                + Fx::from_num(delta))
                            .clamp(Fx::from_num(0), Fx::from_num(1));
                        }
                    }
                    StationPanelButtonAction::LogisticsToggleStorageIntake => {
                        if let Some(mut storage_state) = storage_state {
                            storage_state.allow_intake = !storage_state.allow_intake;
                        }
                    }
                    StationPanelButtonAction::LogisticsToggleAirlock => {
                        if let Some(mut airlock_state) = airlock_state {
                            airlock_state.open = !airlock_state.open;
                            mission_state.airlocks_cycled += 1;
                        }
                    }
                    StationPanelButtonAction::LogisticsToggleManipulator => {
                        if let Some(mut manipulator_state) = manipulator_state {
                            manipulator_state.transfer_enabled =
                                !manipulator_state.transfer_enabled;
                        }
                    }
                    StationPanelButtonAction::LogisticsCycleManipulatorTarget { direction } => {
                        if let Some(mut manipulator_state) = manipulator_state {
                            let candidate_ids = nearby_logistics_target_ids(
                                runtime_module.module_id,
                                &candidate_query,
                            );
                            if !candidate_ids.is_empty() {
                                let current_index = manipulator_state
                                    .target_module_id
                                    .and_then(|id| {
                                        candidate_ids.iter().position(|candidate| *candidate == id)
                                    })
                                    .unwrap_or(0);
                                let next_index = ((current_index as i32 + direction)
                                    .rem_euclid(candidate_ids.len() as i32))
                                    as usize;
                                manipulator_state.target_module_id =
                                    Some(candidate_ids[next_index]);
                                manipulator_state.source_module_id = Some(runtime_module.module_id);
                            }
                        }
                    }
                    StationPanelButtonAction::LogisticsCycleResource => {
                        if let Some(mut manipulator_state) = manipulator_state {
                            manipulator_state.resource_kind = match manipulator_state.resource_kind
                            {
                                crate::client::gameplay::components::ResourceKind::RawSalvage => {
                                    crate::client::gameplay::components::ResourceKind::RepairCharge
                                }
                                crate::client::gameplay::components::ResourceKind::RepairCharge => {
                                    crate::client::gameplay::components::ResourceKind::RawSalvage
                                }
                            };
                        }
                    }
                    StationPanelButtonAction::LogisticsToggleProcessor => {
                        if let Some(mut processor_state) = processor_state {
                            processor_state.enabled = !processor_state.enabled;
                        }
                    }
                    StationPanelButtonAction::ComputerToggleEnabled => {
                        if let Some(mut arch_runtime) = arch_runtime {
                            arch_runtime.enabled = !arch_runtime.enabled;
                        }
                    }
                    StationPanelButtonAction::ComputerCycleTemplate => {
                        if let Some(mut arch_runtime) = arch_runtime {
                            arch_runtime.program = crate::ship::arch::ArchProgram::from_template(
                                arch_runtime.program.template.next(),
                            );
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.30, 0.46, 0.68));
            }
            Interaction::None => {}
        }
    }
}

fn build_top_banner(
    mission_state: &MissionState,
    weapon_state: &ShipWeaponState,
    hostile_count: usize,
    projectile_count: usize,
) -> String {
    let outcome = match mission_return_line(mission_state) {
        Some(return_line) => format!("{}  |  {}", mission_status_line(mission_state), return_line),
        None => mission_status_line(mission_state).to_string(),
    };
    format!(
        "{outcome}\nNode: {} [{}]  |  Threat x{}  |  Projectiles {}  |  Turret Cooldown {}",
        mission_state.node_name,
        mission_state.node_kind_name,
        hostile_count,
        projectile_count,
        format_fx2(weapon_state.cooldown_remaining.max(Fx::from_num(0)))
    )
}

#[allow(clippy::too_many_arguments)]
fn build_compact_status(
    player_motion: &PlayerMotionState,
    ship_position: &SimPosition,
    linear_velocity: &LinearVelocity,
    angular_velocity: &AngularVelocity,
    control_mode: &ShipboardControlState,
    current_station: &CurrentStation,
    frame_label: &str,
    focused_station_context: &str,
    current_integrity: i32,
    max_integrity: i32,
    active_modules: usize,
    degraded_modules: usize,
    disabled_modules: usize,
    atmosphere_state: &ShipAtmosphereState,
    carried_resource: &CarriedResource,
    mission_state: &MissionState,
    scrap_total: u32,
    arch_summary: &ArchSummary,
) -> String {
    format!(
        "Mode: {}  |  Focus: {}\nFrame: {}\nContext: {}\nStation: {}\nPlayer: {}, {} @ {}\nShip: {}, {} @ {}\nTurn: {}\nModules: {} active  |  {} degraded  |  {} disabled\nIntegrity: {} / {}\nAtmosphere: {} avg / {} min  |  venting {}\nCargo: {}\nMission Ops: repairs {}  stabs {}  transfers {}  cycles {}\nARCH: {}  [{}]  writes {}  invalid {}\nScrap: {}",
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
        active_modules,
        degraded_modules,
        disabled_modules,
        current_integrity,
        max_integrity,
        format_fx1(atmosphere_state.average_oxygen),
        format_fx1(atmosphere_state.minimum_oxygen),
        atmosphere_state.venting_tiles,
        carried_resource
            .kind
            .map(|kind| format!("{} {}", carried_resource.amount, resource_kind_label(kind)))
            .unwrap_or_else(|| "none".to_string()),
        mission_state.repairs_performed,
        mission_state.stabilizations_performed,
        mission_state.transfer_count,
        mission_state.processor_cycles,
        arch_summary.program_name,
        arch_summary.exec_summary,
        arch_summary.recent_writes,
        arch_summary.invalid_count,
        scrap_total,
    )
}

struct ArchSummary {
    program_name: String,
    exec_summary: String,
    invalid_count: u32,
    recent_writes: String,
}

#[derive(Clone, Copy, Default)]
struct StationFlags {
    storage: bool,
    manipulator: bool,
    processor: bool,
    airlock: bool,
    computer: bool,
    reactor: bool,
    turret: bool,
}

fn station_panel_content(
    control_mode: &ShipboardControlState,
    mission_state: &MissionState,
    weapon_state: &ShipWeaponState,
    player_fields: &PlayerFieldState,
    player_motion: &PlayerMotionState,
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
    focused_station_context: &str,
    arch_summary: &ArchSummary,
) -> (
    String,
    String,
    Option<crate::ship::ModuleKind>,
    StationFlags,
) {
    let Some(focused_entity) = control_mode.focused_entity else {
        return (
            "Station Console".to_string(),
            format!(
                "Walk the ship or EVA, then press E at a station.\nCurrent frame: {}\nLocal oxygen: {}  heat: {}  electrical: {}\nRecent action: {}",
                match player_motion.frame {
                    PlayerReferenceFrame::World => "EVA / World",
                    PlayerReferenceFrame::Ship(_) => "Ship Local",
                },
                format_fx1(player_fields.local_oxygen),
                format_fx1(player_fields.local_heat),
                format_fx1(player_fields.local_electrical),
                mission_state.recent_action.as_deref().unwrap_or("none"),
            ),
            None,
            StationFlags::default(),
        );
    };

    let Ok((
        _,
        runtime_module,
        integrity,
        runtime_state,
        computer,
        storage,
        storage_command,
        manipulator,
        _manipulator_command,
        processor,
        processor_command,
        reactor,
        turret,
        airlock_state,
        destroyed,
    )) = module_query.get(focused_entity)
    else {
        return (
            "Station Console".to_string(),
            "Focused module unavailable".to_string(),
            None,
            StationFlags::default(),
        );
    };

    let flags = StationFlags {
        storage: storage.is_some(),
        manipulator: manipulator.is_some(),
        processor: processor.is_some(),
        airlock: airlock_state.is_some(),
        computer: computer.is_some(),
        reactor: reactor.is_some(),
        turret: turret.is_some(),
    };

    if destroyed.is_some() {
        return (
            format!("{} Console", module_display_name(runtime_module.kind)),
            format!(
                "Context: {}\nModule destroyed.\nIntegrity: {} / {}\nHeat: {}  Electrical: {}\nLeave with Q or Esc.",
                focused_station_context,
                integrity.current,
                integrity.max,
                format_fx1(runtime_state.current_heat),
                format_fx1(runtime_state.electrical_instability),
            ),
            Some(runtime_module.kind),
            flags,
        );
    }

    let title = match control_mode.mode {
        ShipControlMode::Cockpit => "Helm Console".to_string(),
        ShipControlMode::Turret => "Turret Console".to_string(),
        ShipControlMode::Reactor => "Reactor Console".to_string(),
        ShipControlMode::Logistics => {
            format!("{} Console", module_display_name(runtime_module.kind))
        }
        ShipControlMode::Computer => "Computer Console".to_string(),
        ShipControlMode::Interior => "Station Console".to_string(),
    };

    let body = if let Some(reactor) = reactor {
        format!(
            "Context: {}\nReaction Rate: {}%\nTurbine Load: {}%\nPower Output: {}\nFuel Remaining: {}\nCore Heat: {}\nInstability: {}\nUse the buttons below or W/S + A/D.\nRecent: {}",
            focused_station_context,
            format_fx0(reactor.reaction_rate * Fx::from_num(100)),
            format_fx0(reactor.turbine_load * Fx::from_num(100)),
            format_fx1(reactor.power_output),
            format_fx0(reactor.fuel_remaining),
            format_fx1(runtime_state.current_heat),
            format_fx1(runtime_state.electrical_instability),
            mission_state.recent_action.as_deref().unwrap_or("none"),
        )
    } else if let Some(turret) = turret {
        format!(
            "Context: {}\nDesired Angle: {}\nActual Angle: {}\nFire Gate: {}\nCooldown: {}\nThreat State: {}\nUse buttons or mouse / A,D while manned.",
            focused_station_context,
            format_fx1(turret.desired_angle),
            format_fx1(turret.actual_angle),
            if turret.fire_intent { "open" } else { "hold" },
            format_fx2(weapon_state.cooldown_remaining),
            mission_status_line(mission_state),
        )
    } else if let Some(storage) = storage {
        format!(
            "Context: {}\nStorage Fill: {}/{}\nRaw Salvage: {}\nRepair Charge: {}\nIntake: {}\nAirlock Seal: {}\nManipulator: {}\nCargo can be moved with panel buttons or F/G.",
            focused_station_context,
            storage.inventory.total_units(),
            storage.capacity,
            storage.inventory.raw_salvage,
            storage.inventory.repair_charge,
            if storage_command.is_some_and(|command| command.allow_intake) {
                "open"
            } else {
                "closed"
            },
            if airlock_state.is_some_and(|state| state.open) {
                "open"
            } else {
                "sealed"
            },
            manipulator
                .and_then(|manipulator| manipulator.blocked_reason.clone())
                .unwrap_or_else(|| "ready".to_string()),
        )
    } else if let Some(processor) = processor {
        let progress_pct = if processor.duration > Fx::from_num(0) {
            processor.progress / processor.duration * Fx::from_num(100)
        } else {
            Fx::from_num(0)
        };
        format!(
            "Context: {}\nRecipe: {}\nEnabled: {}\nProgress: {}%\nHeld Raw: {}\nHeld Charge: {}\nState: {}\nFabrication output feeds the ship through logistics.",
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
        )
    } else if let Some(computer) = computer {
        format!(
            "Context: {}\nProgram: {}\nOnline: {}\nExec: {}\nWrites: {}\nInvalid/Halt: {}\nUse buttons here, then refine logic in the refit editor.",
            focused_station_context,
            computer.program.name,
            if computer.enabled { "yes" } else { "no" },
            arch_summary.exec_summary,
            arch_summary.recent_writes,
            arch_summary.invalid_count,
        )
    } else {
        format!(
            "Context: {}\nModule: {}\nCondition: {:?}\nHeat: {}\nElectrical: {}\nNeeds Attention: {}",
            focused_station_context,
            module_display_name(runtime_module.kind),
            module_condition(integrity, runtime_state, false, &BalanceConfig::default()),
            format_fx1(runtime_state.current_heat),
            format_fx1(runtime_state.electrical_instability),
            if runtime_state.needs_attention {
                "yes"
            } else {
                "no"
            },
        )
    };

    (title, body, Some(runtime_module.kind), flags)
}

fn station_action_visible(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    active_station_kind: Option<crate::ship::ModuleKind>,
    flags: StationFlags,
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
            StationPanelButtonAction::LogisticsToggleProcessor => flags.processor,
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

fn station_button_label(
    action: StationPanelButtonAction,
    mode: ShipControlMode,
    flags: StationFlags,
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
        StationPanelButtonAction::LogisticsToggleProcessor => "Toggle Processor".to_string(),
        StationPanelButtonAction::ComputerToggleEnabled => "Enable / Disable".to_string(),
        StationPanelButtonAction::ComputerCycleTemplate => "Cycle Template".to_string(),
    }
}

fn summarize_modules(
    children: &Children,
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
) -> (i32, i32, usize, usize, usize) {
    let mut current_integrity = 0;
    let mut max_integrity = 0;
    let mut active_modules = 0;
    let mut degraded = 0;
    let mut disabled = 0;

    for child in children.iter() {
        let Ok((_, _, integrity, runtime_state, _, _, _, _, _, _, _, _, _, _, destroyed)) =
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
            ModuleCondition::Degraded => degraded += 1,
            ModuleCondition::Disabled => disabled += 1,
            _ => {}
        }
    }

    (
        current_integrity,
        max_integrity,
        active_modules,
        degraded,
        disabled,
    )
}

fn summarize_arch(
    children: &Children,
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
) -> ArchSummary {
    for child in children.iter() {
        let Ok((_, _, _, _, computer, _, _, _, _, _, _, _, _, _, destroyed)) =
            module_query.get(*child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        if let Some(computer) = computer {
            return ArchSummary {
                program_name: if computer.last_result.program_name.is_empty() {
                    computer.program.name.clone()
                } else {
                    computer.last_result.program_name.clone()
                },
                exec_summary: format!(
                    "{}/{}",
                    computer.last_result.executed, computer.last_result.budget
                ),
                invalid_count: u32::from(computer.last_result.halted_reason.is_some()),
                recent_writes: if computer.last_result.recent_writes.is_empty() {
                    "none".to_string()
                } else {
                    computer.last_result.recent_writes.join(", ")
                },
            };
        }
    }

    ArchSummary {
        program_name: "none".to_string(),
        exec_summary: "0/0".to_string(),
        invalid_count: 0,
        recent_writes: "none".to_string(),
    }
}

fn controls_help_text(mode: ShipControlMode) -> String {
    match mode {
        ShipControlMode::Interior => {
            "Walk / EVA\nWASD move or thrust\nE enter station\nF pick up/deposit cargo\nG drop cargo\nQ or Esc leave station\nF3 diagnostics  |  Tab station hub".to_string()
        }
        ShipControlMode::Cockpit => {
            "Helm\nW/S throttle  |  A/D steer\nMouse can also drive helm\nUse on-panel controls for coarse trim\nQ or Esc leave cockpit".to_string()
        }
        ShipControlMode::Turret => {
            "Turret\nMouse or A/D aim\nSpace / left mouse fire\nPanel buttons trim aim or hold fire\nQ or Esc leave turret".to_string()
        }
        ShipControlMode::Reactor => {
            "Reactor\nW/S reaction rate  |  A/D turbine load\nPanel controls support coarse adjustment\nQ or Esc leave reactor".to_string()
        }
        ShipControlMode::Logistics => {
            "Logistics\nSpace, M, R, [ and ] still work\nPanel buttons mirror key actions\nF/G handle carried cargo\nQ or Esc leave panel".to_string()
        }
        ShipControlMode::Computer => {
            "Computer\nSpace toggles online  |  T cycles starter template\nUse the refit ARCH panel for line editing\nQ or Esc leave console".to_string()
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

fn nearby_logistics_target_ids(
    focused_module_id: u64,
    candidate_query: &Query<&RuntimeShipModule>,
) -> Vec<u64> {
    candidate_query
        .iter()
        .filter(|runtime_module| {
            matches!(
                runtime_module.kind,
                crate::ship::ModuleKind::Cargo
                    | crate::ship::ModuleKind::Processor
                    | crate::ship::ModuleKind::Airlock
            ) && runtime_module.module_id != focused_module_id
        })
        .map(|runtime_module| runtime_module.module_id)
        .collect()
}

fn percent(value: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        ((value / max) * 100.0).clamp(0.0, 100.0)
    }
}
