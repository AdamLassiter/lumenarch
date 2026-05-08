use super::{alerts, *};
use crate::gameplay::components::DetectorModule;

pub(crate) fn toggle_gameplay_info_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud_mode: ResMut<GameplayInfoPanelMode>,
) {
    if keys.just_pressed(KeyCode::F1) {
        *hud_mode = GameplayInfoPanelMode::Overview;
    }
    if keys.just_pressed(KeyCode::F2) {
        *hud_mode = GameplayInfoPanelMode::FocusedModule;
    }
    if keys.just_pressed(KeyCode::F3) {
        *hud_mode = GameplayInfoPanelMode::Alerts;
    }
    if keys.just_pressed(KeyCode::F4) {
        *hud_mode = GameplayInfoPanelMode::StationConsole;
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
    module_parent_query: Query<'w, 's, &'static ChildOf, With<RuntimeShipModule>>,
    hostile_query: Query<'w, 's, Entity, With<HostileTarget>>,
    projectile_query: Query<'w, 's, Entity, With<Projectile>>,
    detector_query: Query<'w, 's, &'static DetectorModule, With<RuntimeShipModule>>,
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
    drone_query: Query<
        'w,
        's,
        (
            Option<&'static DroneStationModule>,
            Option<&'static DroneStationCommandState>,
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
            Query<
                'w,
                's,
                (
                    &'static mut Text,
                    Option<&'static GameplayTopBannerText>,
                    Option<&'static GameplayCompactStatusText>,
                    Option<&'static GameplayControlsText>,
                    Option<&'static GameplayPanelTitleText>,
                    Option<&'static GameplayPanelBodyText>,
                    Option<&'static GameplayStationTitleText>,
                ),
            >,
            Query<'w, 's, (&'static GameplayBarLabel, &'static mut Text)>,
            Query<'w, 's, (&'static GameplayStationPanelButtonLabel, &'static mut Text)>,
            Query<
                'w,
                's,
                (
                    &'static ChildOf,
                    &'static mut Text,
                    Option<&'static GameplayStationReadoutLabel>,
                    Option<&'static GameplayStationReadoutValue>,
                ),
            >,
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
            Query<
                'w,
                's,
                (
                    Entity,
                    &'static GameplayStationReadoutSlot,
                    &'static mut Node,
                    &'static Children,
                ),
            >,
            Query<
                'w,
                's,
                (&'static ChildOf, &'static mut Node),
                With<GameplayStationReadoutBarTrack>,
            >,
            Query<
                'w,
                's,
                (
                    &'static ChildOf,
                    &'static mut Node,
                    &'static mut BackgroundColor,
                ),
                With<GameplayStationReadoutBarFill>,
            >,
            Query<
                'w,
                's,
                (
                    &'static ChildOf,
                    &'static mut Node,
                    &'static mut BackgroundColor,
                ),
                With<GameplayStationReadoutLight>,
            >,
            Query<'w, 's, &'static mut BackgroundColor, With<GameplayBlackoutOverlay>>,
        ),
    >,
}

pub(crate) fn update_gameplay_status_text(
    balance: Res<BalanceConfig>,
    hud_mode: Res<GameplayInfoPanelMode>,
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
            &ShipControlState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    player_query: Single<
        (
            &PlayerIdentity,
            &CurrentStation,
            &NearbyInteraction,
            &HeldInteraction,
            &PlayerFieldState,
            &PlayerMotionState,
            &CarriedResource,
            &EquippedSuit,
            &PlayerConditionState,
            &ShipboardControlState,
        ),
        With<ObservedLocalPlayerMarker>,
    >,
    status_world: GameplayStatusWorldQueries,
    progression: Res<Progression>,
    rollback_state: Res<netcode::RollbackGameState>,
    checksum_history: Res<netcode::ChecksumHistory>,
    mut hud_ui: GameplayHudUiQueries,
) {
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        automation_state,
        _movement_model,
        power_state,
        power_model,
        weapon_state,
        mission_state,
        atmosphere_state,
        _ship_controls,
    ) = ship_query.into_inner();
    let (
        player_identity,
        current_station,
        nearby_interaction,
        held_interaction,
        player_fields,
        player_motion,
        carried_resource,
        equipped_suit,
        player_condition,
        control_mode,
    ) = player_query.into_inner();

    let frame_label =
        summary::reference_frame_label(player_motion, &status_world.ship_identity_query);
    let focused_station_context = control_mode
        .focused_entity
        .and_then(|entity| status_world.module_parent_query.get(entity).ok())
        .map(|parent| {
            summary::ship_affiliation_label(parent.get(), &status_world.ship_identity_query)
        })
        .unwrap_or("Unattached");

    let (current_integrity, max_integrity, active_modules, degraded_modules, disabled_modules) =
        summary::summarize_modules(children, &status_world.module_query, &balance);
    let arch_summary = summary::summarize_arch(children, &status_world.module_query);
    let top_banner = summary::build_top_banner(
        mission_state,
        weapon_state,
        status_world.hostile_query.iter().len(),
        status_world.projectile_query.iter().len(),
    );
    let compact = summary::build_compact_status(
        player_identity,
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
        equipped_suit,
        carried_resource,
        player_condition,
        mission_state,
        progression.scrap,
        &arch_summary,
        rollback_state.frame,
        checksum_history.last_value,
    );
    let controls_text = format!(
        "{}\nF1 ship  |  F2 focus  |  F3 alerts  |  F4 station",
        summary::controls_help_text(control_mode.mode)
    );
    let (panel_title, panel_body, _, _) = station_panel::station_panel_content(
        control_mode,
        mission_state,
        weapon_state,
        player_fields,
        player_motion,
        &status_world.module_query,
        &status_world.drone_query,
        focused_station_context,
        &arch_summary,
    );
    let station_display = station_panel::station_panel_display(
        control_mode,
        mission_state,
        weapon_state,
        player_fields,
        &status_world.module_query,
        &status_world.drone_query,
        focused_station_context,
        &arch_summary,
    );
    let current_module = status_world.module_query.iter().find(
        |(_, runtime_module, _, _, _, _, _, _, _, _, _, _, _, _, _)| {
            runtime_module.module_id == current_station.module_id
        },
    );
    let mut issues = alerts::collect_alert_issues(&status_world.module_query, &balance);
    issues.sort_by_key(|issue| std::cmp::Reverse(issue.0));
    issues.truncate(3);
    let shared_title = match *hud_mode {
        GameplayInfoPanelMode::Overview => "Ship Overview".to_string(),
        GameplayInfoPanelMode::FocusedModule => "Focused Module".to_string(),
        GameplayInfoPanelMode::Alerts => "Alerts".to_string(),
        GameplayInfoPanelMode::StationConsole => panel_title.clone(),
    };
    let shared_body = match *hud_mode {
        GameplayInfoPanelMode::Overview => format!("{top_banner}\n\n{compact}"),
        GameplayInfoPanelMode::FocusedModule => {
            let detector = current_module
                .and_then(|(entity, ..)| status_world.detector_query.get(entity).ok());
            let drone_station = current_module.and_then(|(entity, ..)| {
                status_world
                    .drone_query
                    .get(entity)
                    .ok()
                    .and_then(|(station, _)| station)
            });
            alerts::inspection_text(
                current_module,
                detector,
                drone_station,
                nearby_interaction,
                held_interaction,
                automation_state,
                &balance,
            )
        }
        GameplayInfoPanelMode::Alerts => alerts::alerts_text(
            player_fields,
            nearby_interaction,
            mission_state,
            atmosphere_state,
            &issues,
        ),
        GameplayInfoPanelMode::StationConsole => panel_body.clone(),
    };
    let show_station_panel = !matches!(
        control_mode.mode,
        ShipControlMode::Interior | ShipControlMode::Cockpit | ShipControlMode::Turret
    );

    for (
        mut text,
        top_banner_marker,
        compact_marker,
        controls_marker,
        panel_title_marker,
        panel_body_marker,
        station_title_marker,
    ) in &mut hud_ui.text_queries.p0()
    {
        if top_banner_marker.is_some() {
            **text = top_banner.clone();
        } else if compact_marker.is_some() {
            **text = compact.clone();
        } else if controls_marker.is_some() {
            **text = controls_text.clone();
        } else if panel_title_marker.is_some() {
            **text = shared_title.clone();
        } else if panel_body_marker.is_some() {
            **text = shared_body.clone();
        } else if station_title_marker.is_some() {
            **text = station_display.title.clone();
        }
    }

    let hull_pct = summary::percent(current_integrity as f32, max_integrity.max(1) as f32);
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
    let blackout_alpha = player_condition
        .blackout
        .clamp(Fx::from_num(0), Fx::from_num(1));

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

    for (label, mut text) in &mut hud_ui.text_queries.p1() {
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
        let visible = station_panel::station_action_visible(
            button.action,
            control_mode.mode,
            station_display.active_station_kind,
            station_display.flags,
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
            if let Ok((label, mut text)) = hud_ui.text_queries.p2().get_mut(child) {
                **text = station_panel::station_button_label(
                    label.action,
                    control_mode.mode,
                    station_display.flags,
                );
            }
        }
    }

    let mut readout_rows = Vec::new();
    for (row_entity, slot, mut row_node, children) in &mut hud_ui.node_queries.p3() {
        let readout = station_display.readouts.get(slot.index as usize).cloned();
        row_node.display = if show_station_panel && readout.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(readout) = readout {
            readout_rows.push((row_entity, children.to_vec(), readout));
        }
    }

    for (row_entity, children, readout) in readout_rows {
        for child in children {
            if let Ok((parent, mut text, is_label, is_value)) =
                hud_ui.text_queries.p3().get_mut(child)
                && parent.get() == row_entity
            {
                if is_label.is_some() {
                    **text = readout.label.clone();
                } else if is_value.is_some() {
                    **text = readout.value.clone();
                }
            }
            if let Ok((parent, mut node)) = hud_ui.node_queries.p4().get_mut(child)
                && parent.get() == row_entity
            {
                node.display = if matches!(
                    readout.visual,
                    station_panel::StationReadoutVisual::Bar { .. }
                ) {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Ok((parent, mut node, mut color)) = hud_ui.node_queries.p5().get_mut(child)
                && parent.get() == row_entity
            {
                if let station_panel::StationReadoutVisual::Bar {
                    percent,
                    color: fill_color,
                } = readout.visual
                {
                    node.width = Val::Percent(percent);
                    *color = BackgroundColor(fill_color);
                }
                node.display = Display::Flex;
            }
            if let Ok((parent, mut node, mut color)) = hud_ui.node_queries.p6().get_mut(child)
                && parent.get() == row_entity
            {
                if let station_panel::StationReadoutVisual::Light { color: light_color } =
                    readout.visual
                {
                    node.display = Display::Flex;
                    *color = BackgroundColor(light_color);
                } else {
                    node.display = Display::None;
                }
            }
        }
    }

    for mut color in &mut hud_ui.node_queries.p7() {
        *color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, blackout_alpha.to_num::<f32>()));
    }
}
