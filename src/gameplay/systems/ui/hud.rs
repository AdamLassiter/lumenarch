use super::{alerts, *};
use crate::{
    gameplay::components::{DetectorModule, ModuleInfrastructureStatus},
    ship::ModuleKind,
};

/// Cycles the encounter info panel mode so players can switch between summaries and inspection views.
pub(crate) fn toggle_gameplay_info_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud_mode: ResMut<GameplayInfoPanelMode>,
) {
    if keys.just_pressed(KeyCode::F1) {
        *hud_mode = GameplayInfoPanelMode::StationConsole;
    }
    if keys.just_pressed(KeyCode::F2) {
        *hud_mode = GameplayInfoPanelMode::FocusedModule;
    }
    if keys.just_pressed(KeyCode::F3) {
        *hud_mode = GameplayInfoPanelMode::Alerts;
    }
    if keys.just_pressed(KeyCode::F4) {
        *hud_mode = GameplayInfoPanelMode::Overview;
    }
    if keys.just_pressed(KeyCode::F5) {
        *hud_mode = GameplayInfoPanelMode::Tubes;
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
                (
                    Entity,
                    &'static ChildOf,
                    &'static mut Node,
                    &'static Children,
                ),
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

/// Rebuilds the encounter HUD text, bars, and blackout state from current combat and player state.
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
            &ShipInfrastructureState,
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
            &PlayerFocusedTile,
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
    // SAFETY: HUD `ParamSet`s split text roles, bar fills, station buttons, readouts, and overlay nodes
    // into distinct marker-driven branches that are accessed sequentially, avoiding double mutable UI borrows.
    let (
        ship_position,
        children,
        linear_velocity,
        angular_velocity,
        automation_state,
        _movement_model,
        power_state,
        power_model,
        infrastructure_state,
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
        focused_tile,
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
        "{}\nF1 station  |  F2 focus  |  F3 alerts  |  F4 ship  |  F5 tubes",
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
        infrastructure_state,
    );
    let station_display = station_panel::station_panel_display(
        control_mode,
        mission_state,
        power_state,
        power_model,
        weapon_state,
        player_fields,
        &status_world.module_query,
        &status_world.drone_query,
        focused_station_context,
        &arch_summary,
        infrastructure_state,
    );
    let current_module = status_world
        .module_query
        .iter()
        .find(|(_, runtime_module, ..)| runtime_module.module_id == current_station.module_id);
    let focused_module = match focused_tile.ship {
        Some(ship_entity) => {
            status_world
                .module_query
                .iter()
                .find(|(entity, runtime_module, ..)| {
                    runtime_module.grid_x == focused_tile.grid_x
                        && runtime_module.grid_y == focused_tile.grid_y
                        && status_world
                            .module_parent_query
                            .get(*entity)
                            .is_ok_and(|parent| parent.get() == ship_entity)
                })
        }
        None => None,
    };
    let tubes_body = tubes_debug_text(
        infrastructure_state,
        focused_tile,
        focused_module.map(|(_, runtime_module, ..)| runtime_module),
    );
    let mut issues = alerts::collect_alert_issues(&status_world.module_query, &balance);
    issues.sort_by_key(|issue| std::cmp::Reverse(issue.0));
    issues.truncate(3);
    let shared_title = match *hud_mode {
        GameplayInfoPanelMode::Overview => "Ship Overview".to_string(),
        GameplayInfoPanelMode::FocusedModule => "Focused Module".to_string(),
        GameplayInfoPanelMode::Alerts => "Alerts".to_string(),
        GameplayInfoPanelMode::StationConsole => panel_title.clone(),
        GameplayInfoPanelMode::Tubes => "Tubes".to_string(),
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
        GameplayInfoPanelMode::Tubes => tubes_body,
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
            let fill_update = if let Ok((track_entity, parent, mut node, track_children)) =
                hud_ui.node_queries.p4().get_mut(child)
                && parent.get() == row_entity
            {
                let station_panel::StationReadoutVisual::Bar {
                    percent,
                    color: fill_color,
                } = readout.visual
                else {
                    node.display = Display::None;
                    continue;
                };

                node.display = Display::Flex;
                Some((track_entity, track_children.to_vec(), percent, fill_color))
            } else {
                None
            };

            if let Some((track_entity, track_children, percent, fill_color)) = fill_update {
                for track_child in track_children {
                    if let Ok((parent, mut text, is_label, is_value)) =
                        hud_ui.text_queries.p3().get_mut(track_child)
                        && parent.get() == track_entity
                    {
                        if is_label.is_some() {
                            **text = readout.label.clone();
                        } else if is_value.is_some() {
                            **text = readout.value.clone();
                        }
                    }
                    if let Ok((parent, mut fill_node, mut color)) =
                        hud_ui.node_queries.p5().get_mut(track_child)
                        && parent.get() == track_entity
                    {
                        fill_node.width = Val::Percent(percent);
                        fill_node.display = Display::Flex;
                        *color = BackgroundColor(fill_color);
                    }
                }
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

fn tubes_debug_text(
    infrastructure: &ShipInfrastructureState,
    focused_tile: &PlayerFocusedTile,
    current_module: Option<&RuntimeShipModule>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Networks: {}  |  Strict routing: {}",
        infrastructure.networks.len(),
        if infrastructure.strict_routing {
            "on"
        } else {
            "off"
        }
    ));

    if let Some(runtime_module) = current_module {
        lines.push(format!(
            "Focused tile: {}, {}  |  {} {} @ {}, {}",
            focused_tile.grid_x,
            focused_tile.grid_y,
            module_display_name(runtime_module.kind),
            runtime_module.module_id,
            runtime_module.grid_x,
            runtime_module.grid_y,
        ));
        lines.extend(tubes_module_lines(infrastructure, runtime_module));
    } else {
        lines.push(format!(
            "Focused tile: {}, {}  |  no component",
            focused_tile.grid_x, focused_tile.grid_y
        ));
    }
    lines.push("".to_string());
    lines.push("Networks".to_string());

    for network in infrastructure.networks.iter().take(10) {
        let kind = network
            .kind
            .map(|kind| kind.label().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        lines.push(format!(
            "#{:02} {}  |  tiles {}  modules {}  supply {}  demand {}  reserve {}  flow {}  blockers {}",
            network.id,
            kind,
            network.tile_count,
            network.attached_modules.len(),
            format_fx1(network.supply),
            format_fx1(network.demand),
            format_fx1(network.reserve),
            format_fx1(network.flow),
            network.blockers,
        ));
    }

    let blocked = infrastructure
        .module_statuses
        .iter()
        .filter(|status| status.blocked_reason.is_some())
        .take(8)
        .map(|status| {
            format!(
                "{} {}: {}",
                module_display_name(status.kind),
                status.module_id,
                status.blocked_reason.as_deref().unwrap_or("blocked")
            )
        })
        .collect::<Vec<_>>();
    if !blocked.is_empty() {
        lines.push("Blocked consumers:".to_string());
        lines.extend(blocked);
    }

    if lines.len() == 1 {
        lines.push("No routed infrastructure networks found.".to_string());
    }
    lines.join("\n")
}

fn tubes_module_lines(
    infrastructure: &ShipInfrastructureState,
    runtime_module: &RuntimeShipModule,
) -> Vec<String> {
    let mut lines = Vec::new();
    let module_name = module_display_name(runtime_module.kind);
    let Some(status) = infrastructure.status_for_module(runtime_module.module_id) else {
        lines.push(format!(
            "{} {} has no infrastructure status",
            module_name, runtime_module.module_id
        ));
        return lines;
    };

    lines.push(format!(
        "Status: {}",
        status.blocked_reason.as_deref().unwrap_or("connected")
    ));
    if status.power_required {
        lines.push(format!(
            "Power: {}  |  {}",
            if status.powered {
                "powered"
            } else {
                "unpowered"
            },
            network_debug_line(infrastructure, status.power_network)
        ));
    } else if let Some(network_id) = status.power_network {
        lines.push(format!(
            "Power route: {}",
            network_debug_line(infrastructure, Some(network_id))
        ));
    }
    if let Some(network_id) = status.duct_network {
        lines.push(format!(
            "O2 duct: {}",
            network_debug_line(infrastructure, Some(network_id))
        ));
    }
    lines.push("Service ports: self + cardinal neighbors".to_string());
    for service in &status.service_statuses {
        lines.push(format!(
            "{}{}: {}{}{}",
            service.route_kind.label(),
            if service.required { " required" } else { "" },
            network_debug_line(infrastructure, service.network_id),
            service
                .service_coord
                .map(|(x, y)| format!(" @ ({x},{y})"))
                .unwrap_or_default(),
            service
                .blocked_reason
                .as_deref()
                .map(|reason| format!(" [{reason}]"))
                .unwrap_or_default()
        ));
    }
    lines.push(format!(
        "Role: {}",
        module_tube_role(runtime_module.kind, status)
    ));
    lines
}

fn network_debug_line(infrastructure: &ShipInfrastructureState, network_id: Option<u32>) -> String {
    let Some(network_id) = network_id else {
        return "not connected".to_string();
    };
    let Some(network) = infrastructure.network(network_id) else {
        return format!("#{network_id} missing");
    };
    format!(
        "#{:02} supply={} demand={} reserve={} flow={} attached={}",
        network.id,
        format_fx1(network.supply),
        format_fx1(network.demand),
        format_fx1(network.reserve),
        format_fx1(network.flow),
        network.attached_modules.len(),
    )
}

fn module_tube_role(kind: ModuleKind, status: &ModuleInfrastructureStatus) -> String {
    match kind {
        ModuleKind::Reactor => "produces power; consumes fuel pipe".to_string(),
        ModuleKind::Battery => "stores power reserve on its wire network".to_string(),
        ModuleKind::Turret => "consumes wired power and ammunition pipe".to_string(),
        ModuleKind::Engine => "consumes wired power".to_string(),
        ModuleKind::Processor => {
            "consumes raw salvage; outputs repair/fuel/ammunition by selected recipe".to_string()
        }
        ModuleKind::O2Generator => "supplies oxygen to attached O2 infrastructure".to_string(),
        ModuleKind::Cargo => "storage endpoint for compatible resource pipes".to_string(),
        ModuleKind::JunctionBox => "power blocker/controller".to_string(),
        ModuleKind::Valve => "duct/pipe blocker/controller".to_string(),
        _ if status.power_required => "consumes wired power".to_string(),
        _ if !status.resource_networks.is_empty() => {
            "attached to resource infrastructure".to_string()
        }
        _ => "no active tube IO".to_string(),
    }
}
