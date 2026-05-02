use super::*;

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
            &ShipboardControlState,
        ),
        With<ObservedLocalPlayerMarker>,
    >,
    status_world: GameplayStatusWorldQueries,
    progression: Res<DemoProgression>,
    rollback_state: Res<netcode::RollbackGameState>,
    checksum_history: Res<netcode::ChecksumHistory>,
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
        _ship_controls,
    ) = ship_query.into_inner();
    let (current_station, player_fields, player_motion, carried_resource, control_mode) =
        player_query.into_inner();

    let frame_label = summary::reference_frame_label(player_motion, &status_world.ship_identity_query);
    let focused_station_context = control_mode
        .focused_entity
        .and_then(|entity| status_world.module_parent_query.get(entity).ok())
        .map(|parent| summary::ship_affiliation_label(parent.get(), &status_world.ship_identity_query))
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
        rollback_state.frame,
        checksum_history.last_value,
    );
    let controls_text = summary::controls_help_text(control_mode.mode);
    let (panel_title, panel_body, active_station_kind, active_station_flags) =
        station_panel::station_panel_content(
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
        let visible = station_panel::station_action_visible(
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
            if let Ok((label, mut text)) = hud_ui.text_queries.p6().get_mut(child) {
                **text = station_panel::station_button_label(
                    label.action,
                    control_mode.mode,
                    active_station_flags,
                );
            }
        }
    }
}
