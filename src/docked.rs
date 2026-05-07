use std::hash::{Hash, Hasher};

use bevy::{log, prelude::*};

use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    SELECTED_BUTTON,
    TILE_SIZE,
    TOOLBOX_WIDTH,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    campaign::{CampaignSave, load_campaign, save_campaign},
    netcode,
    ship::{ModuleKind, ModuleVariant},
    state::{
        CampaignLoadState,
        DockedAcceptContractButton,
        DockedContactNextButton,
        DockedContactPrevButton,
        DockedContentText,
        DockedContractNextButton,
        DockedContractPrevButton,
        DockedHelpText,
        DockedLaunchContractButton,
        DockedLoreNextButton,
        DockedLorePrevButton,
        DockedRoot,
        DockedState,
        DockedStatusText,
        DockedSurface,
        DockedSurfaceButton,
        EditorMode,
        EditorSessionState,
        EditorShip,
        EnemyShipLibraryState,
        LastMissionReport,
        LocalPlayerProfile,
        OpenSectorMapButton,
        Progression,
        RefitButton,
        RepairShipButton,
        SectorState,
    },
    stations::{self, StationCatalogResource},
};
use crate::ship::{
    ShipDefinition,
    enemy::load_validated_default_enemy_library,
    storage::load_default_ship,
};

#[derive(Component)]
pub(crate) struct DockedPreviewRoot;

#[derive(Component)]
struct DockedPreviewTile;

#[derive(Component)]
pub(crate) struct DockedPreviewSignature(u128);

#[derive(Component, Clone, Copy)]
pub(crate) struct DockedActionVisibility {
    shipyard: bool,
    quartermaster: bool,
    contracts: bool,
    archives: bool,
}

impl DockedActionVisibility {
    const fn always() -> Self {
        Self {
            shipyard: true,
            quartermaster: true,
            contracts: true,
            archives: true,
        }
    }

    const fn shipyard() -> Self {
        Self {
            shipyard: true,
            quartermaster: false,
            contracts: false,
            archives: false,
        }
    }

    const fn shipyard_and_quartermaster() -> Self {
        Self {
            shipyard: true,
            quartermaster: true,
            contracts: false,
            archives: false,
        }
    }

    const fn contracts() -> Self {
        Self {
            shipyard: false,
            quartermaster: false,
            contracts: true,
            archives: false,
        }
    }

    const fn archives() -> Self {
        Self {
            shipyard: false,
            quartermaster: false,
            contracts: false,
            archives: true,
        }
    }

    fn visible_on(self, surface: DockedSurface) -> bool {
        match surface {
            DockedSurface::Shipyard => self.shipyard,
            DockedSurface::Quartermaster => self.quartermaster,
            DockedSurface::Contracts => self.contracts,
            DockedSurface::Archives => self.archives,
        }
    }
}

pub(crate) fn initialize_campaign_state(
    status: Res<netcode::SessionStatus>,
    rollback_state: Res<netcode::RollbackGameState>,
    stations: Res<StationCatalogResource>,
    mut campaign_load_state: ResMut<CampaignLoadState>,
    mut progression: ResMut<Progression>,
    mut sector_state: ResMut<SectorState>,
    mut docked_state: ResMut<DockedState>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if !campaign_load_state.hydrated {
        if matches!(status.phase, netcode::SessionPhase::Connected) {
            *progression = rollback_state.progression.clone();
            *sector_state = rollback_state.sector.clone();
            *last_mission_report = rollback_state.last_mission_report.clone();
            editor_ship.ship = rollback_state.editor_ship.clone();
            campaign_load_state.hydrated = true;
        } else {
            match load_campaign() {
                Ok(Some(save)) => {
                    *progression = save.progression;
                    *sector_state = save.sector;
                    let previous_enemy_ship_ids = sector_state
                        .nodes
                        .iter()
                        .map(|node| {
                            (
                                node.id,
                                node.encounter.enemy_ship_ids.clone(),
                                node.encounter.hostile_count,
                            )
                        })
                        .collect::<Vec<_>>();
                    sector_state.ensure_latest_layout();
                    let refreshed_enemy_ship_nodes = sector_state
                        .nodes
                        .iter()
                        .filter_map(|node| {
                            previous_enemy_ship_ids
                                .iter()
                                .find(|(id, _, _)| *id == node.id)
                                .and_then(|(_, previous_ids, previous_hostile_count)| {
                                    if *previous_ids != node.encounter.enemy_ship_ids
                                        || *previous_hostile_count != node.encounter.hostile_count
                                    {
                                        Some((
                                            node.id,
                                            previous_ids.clone(),
                                            node.encounter.enemy_ship_ids.clone(),
                                            *previous_hostile_count,
                                            node.encounter.hostile_count,
                                        ))
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect::<Vec<_>>();
                    if !refreshed_enemy_ship_nodes.is_empty() {
                        log::warn!(
                            "Campaign sector data differed from current sector layout; refreshed {} node(s) while entering docked flow",
                            refreshed_enemy_ship_nodes.len()
                        );
                        for (
                            node_id,
                            previous_ids,
                            refreshed_ids,
                            previous_hostile_count,
                            refreshed_hostile_count,
                        ) in &refreshed_enemy_ship_nodes
                        {
                            log::debug!(
                                "Refreshed docked sector node {} hostile config: enemy_ship_ids {:?} -> {:?}, hostile_count {} -> {}",
                                node_id,
                                previous_ids,
                                refreshed_ids,
                                previous_hostile_count,
                                refreshed_hostile_count
                            );
                        }
                    }
                    *last_mission_report = save.last_mission_report;
                }
                Ok(None) => {
                    *progression = Progression::default();
                    *sector_state = SectorState::default();
                    *last_mission_report = LastMissionReport::default();
                }
                Err(error) => {
                    eprintln!("campaign: failed to load save state: {error}");
                    *progression = Progression::default();
                    *sector_state = SectorState::default();
                    *last_mission_report = LastMissionReport::default();
                }
            }

            if let Ok(Some(saved_ship)) = load_default_ship() {
                editor_ship.ship = saved_ship;
            } else if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
                editor_ship.ship = snapshot.clone();
            } else if editor_ship.ship.name.is_empty() && editor_ship.ship.modules.is_empty() {
                editor_ship.ship = ShipDefinition::empty("Untitled Knot");
            }

            if let Ok(Some(validated)) = load_validated_default_enemy_library() {
                enemy_library_state.library = validated.library;
                enemy_library_state.entry_statuses = validated.statuses;
                enemy_library_state.library.ensure_seeded();
            }

            campaign_load_state.hydrated = true;
        }
    }

    if let Some(station_id) = stations::current_station_id(&sector_state) {
        progression.unlock_station(station_id.to_string());
        if let Some(station) = stations.0.station(station_id) {
            if docked_state.station_title != station.name {
                docked_state.station_title = station.name.clone();
            }
            for contact in &station.contacts {
                progression.unlock_contact(contact.id.clone());
            }
            return;
        }
    }

    if let Some(current_node) = sector_state.current_node() {
        if docked_state.station_title != current_node.label {
            docked_state.station_title = current_node.label.clone();
        }
    }
}

pub(crate) fn persist_campaign_state(
    campaign_load_state: Res<CampaignLoadState>,
    progression: Res<Progression>,
    sector_state: Res<SectorState>,
    last_mission_report: Res<LastMissionReport>,
) {
    if !campaign_load_state.hydrated {
        return;
    }

    if !progression.is_changed() && !sector_state.is_changed() && !last_mission_report.is_changed()
    {
        return;
    }

    let save = CampaignSave {
        progression: progression.clone(),
        sector: sector_state.clone(),
        last_mission_report: last_mission_report.clone(),
    };
    if let Err(error) = save_campaign(&save) {
        eprintln!("campaign: failed to persist save state: {error}");
    }
}

pub(crate) fn spawn_docked_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    docked_state: Res<DockedState>,
    progression: Res<Progression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ship: Res<EditorShip>,
    status: Res<netcode::SessionStatus>,
    local_profile: Res<LocalPlayerProfile>,
    sector_state: Res<SectorState>,
    stations: Res<StationCatalogResource>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let preview_ship = docked_preview_ship(&editor_ship, &status);
    spawn_docked_ship_preview(
        &mut commands,
        &asset_server,
        preview_ship.clone(),
        docked_preview_signature(&preview_ship),
    );

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            DockedRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(TOOLBOX_WIDTH + 80.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(22.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(format!("Docked at\n{}", docked_state.station_title)),
                    TextFont {
                        font: title_font.clone(),
                        font_size: UI_TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                panel.spawn((
                    Text::new(docked_status_text(
                        &docked_state,
                        &progression,
                        &editor_ship,
                        &last_mission_report,
                        &local_profile,
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.90, 0.93, 0.98)),
                    DockedStatusText,
                ));

                spawn_action_button(
                    panel,
                    "Shipyard",
                    docked_surface_color(docked_state.selected_surface, DockedSurface::Shipyard),
                    DockedSurfaceButton {
                        surface: DockedSurface::Shipyard,
                    },
                    &title_font,
                    DockedActionVisibility::always(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Quartermaster",
                    docked_surface_color(
                        docked_state.selected_surface,
                        DockedSurface::Quartermaster,
                    ),
                    DockedSurfaceButton {
                        surface: DockedSurface::Quartermaster,
                    },
                    &title_font,
                    DockedActionVisibility::always(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Contracts",
                    docked_surface_color(docked_state.selected_surface, DockedSurface::Contracts),
                    DockedSurfaceButton {
                        surface: DockedSurface::Contracts,
                    },
                    &title_font,
                    DockedActionVisibility::always(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Archives",
                    docked_surface_color(docked_state.selected_surface, DockedSurface::Archives),
                    DockedSurfaceButton {
                        surface: DockedSurface::Archives,
                    },
                    &title_font,
                    DockedActionVisibility::always(),
                    docked_state.selected_surface,
                );
            });

            let station = stations::current_station(&stations.0, &sector_state);
            let content_text = docked_content_text(
                station,
                &docked_state,
                &progression,
                &last_mission_report,
                &sector_state,
            );
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(TOOLBOX_WIDTH + 120.0),
                    top: Val::Px(48.0),
                    width: Val::Px(520.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.09, 0.12, 0.18, 0.90)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(content_text),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                    DockedContentText,
                ));

                spawn_action_button(
                    panel,
                    "Refit Ship",
                    NORMAL_BUTTON,
                    RefitButton,
                    &title_font,
                    DockedActionVisibility::shipyard(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Repair Ship",
                    Color::srgb(0.45, 0.34, 0.16),
                    RepairShipButton,
                    &title_font,
                    DockedActionVisibility::shipyard_and_quartermaster(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Open Sector Map",
                    Color::srgb(0.18, 0.50, 0.30),
                    OpenSectorMapButton,
                    &title_font,
                    DockedActionVisibility::shipyard(),
                    docked_state.selected_surface,
                );
                spawn_dual_action_row(
                    panel,
                    ("Previous Contract", DockedContractPrevButton),
                    ("Next Contract", DockedContractNextButton),
                    &title_font,
                    DockedActionVisibility::contracts(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Accept Contract",
                    Color::srgb(0.28, 0.46, 0.74),
                    DockedAcceptContractButton,
                    &title_font,
                    DockedActionVisibility::contracts(),
                    docked_state.selected_surface,
                );
                spawn_action_button(
                    panel,
                    "Launch Contract",
                    Color::srgb(0.18, 0.50, 0.30),
                    DockedLaunchContractButton,
                    &title_font,
                    DockedActionVisibility::contracts(),
                    docked_state.selected_surface,
                );
                spawn_dual_action_row(
                    panel,
                    ("Previous Contact", DockedContactPrevButton),
                    ("Next Contact", DockedContactNextButton),
                    &title_font,
                    DockedActionVisibility::archives(),
                    docked_state.selected_surface,
                );
                spawn_dual_action_row(
                    panel,
                    ("Previous Lore", DockedLorePrevButton),
                    ("Next Lore", DockedLoreNextButton),
                    &title_font,
                    DockedActionVisibility::archives(),
                    docked_state.selected_surface,
                );
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.90)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(docked_help_text(docked_state.selected_surface)),
                    TextFont {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: UI_HELP_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                    DockedHelpText,
                ));
            });
        });
}

pub(crate) fn cleanup_docked_ui(
    mut commands: Commands,
    query: Query<Entity, With<DockedRoot>>,
    preview_query: Query<Entity, With<DockedPreviewRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn sync_docked_ship_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    existing_query: Query<(Entity, &DockedPreviewSignature), With<DockedPreviewRoot>>,
) {
    let ship = editor_ship.ship.clone();
    let ship_signature = docked_preview_signature(&ship);

    let existing = existing_query.iter().collect::<Vec<_>>();
    let matching_count = existing
        .iter()
        .filter(|(_, signature)| signature.0 == ship_signature)
        .count();

    if matching_count == 1 && existing.len() == 1 {
        return;
    }

    for (entity, _) in existing {
        commands.entity(entity).despawn();
    }

    spawn_docked_ship_preview(&mut commands, &asset_server, ship, ship_signature);
}

pub(crate) fn rotate_docked_ship_preview(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DockedPreviewRoot>>,
) {
    let panel_width = TOOLBOX_WIDTH + 80.0;
    let desired_x = panel_width * 0.5;
    let desired_y = 0.0f32;
    for mut transform in &mut query {
        transform.translation.x = desired_x;
        transform.translation.y = desired_y;
        transform.rotate_z(0.12 * time.delta_secs());
    }
}

pub(crate) fn docked_ui_missing(query: Query<Entity, With<DockedRoot>>) -> bool {
    query.is_empty()
}

pub(crate) fn docked_ui_present(query: Query<Entity, With<DockedRoot>>) -> bool {
    !query.is_empty()
}

pub(crate) fn docked_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&DockedSurfaceButton>,
            Option<&RefitButton>,
            Option<&OpenSectorMapButton>,
            Option<&RepairShipButton>,
            Option<&DockedContractPrevButton>,
            Option<&DockedContractNextButton>,
            Option<&DockedAcceptContractButton>,
            Option<&DockedLaunchContractButton>,
            Option<&DockedContactPrevButton>,
            Option<&DockedContactNextButton>,
            Option<&DockedLorePrevButton>,
            Option<&DockedLoreNextButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut editor_session: ResMut<EditorSessionState>,
    mut docked_state: ResMut<DockedState>,
    progression: Res<Progression>,
    sector_state: Res<SectorState>,
    stations: Res<StationCatalogResource>,
) {
    for (
        interaction,
        mut background,
        surface,
        refit,
        open_map,
        repair,
        contract_prev,
        contract_next,
        accept_contract,
        launch_contract,
        contact_prev,
        contact_next,
        lore_prev,
        lore_next,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(surface) = surface {
                    docked_state.selected_surface = surface.surface;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if contract_prev.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.contracts.is_empty()
                    {
                        let len = station.contracts.len();
                        docked_state.selected_contract_index =
                            (docked_state.selected_contract_index + len - 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if contract_next.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.contracts.is_empty()
                    {
                        let len = station.contracts.len();
                        docked_state.selected_contract_index =
                            (docked_state.selected_contract_index + 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if contact_prev.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.contacts.is_empty()
                    {
                        let len = station.contacts.len();
                        docked_state.selected_contact_index =
                            (docked_state.selected_contact_index + len - 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if contact_next.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.contacts.is_empty()
                    {
                        let len = station.contacts.len();
                        docked_state.selected_contact_index =
                            (docked_state.selected_contact_index + 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if lore_prev.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.lore_entries.is_empty()
                    {
                        let len = station.lore_entries.len();
                        docked_state.selected_lore_index =
                            (docked_state.selected_lore_index + len - 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if lore_next.is_some() {
                    if let Some(station) = stations::current_station(&stations.0, &sector_state)
                        && !station.lore_entries.is_empty()
                    {
                        let len = station.lore_entries.len();
                        docked_state.selected_lore_index =
                            (docked_state.selected_lore_index + 1) % len;
                    }
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if !netcode::is_host_authority(&status) {
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if refit.is_some() {
                    *background = BackgroundColor(PRESSED_BUTTON);
                    editor_session.mode = EditorMode::Player;
                    log::debug!("Docked UI queued OpenEditor meta command");
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::OpenEditor,
                        ..Default::default()
                    });
                } else if open_map.is_some() {
                    *background = BackgroundColor(Color::srgb(0.12, 0.40, 0.24));
                    log::debug!("Docked UI queued OpenSectorMap meta command");
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::OpenSectorMap,
                        ..Default::default()
                    });
                } else if repair.is_some() {
                    *background = BackgroundColor(Color::srgb(0.62, 0.44, 0.16));
                    log::debug!("Docked UI queued RepairShip meta command");
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::RepairShip,
                        ..Default::default()
                    });
                } else if accept_contract.is_some() {
                    if let Some(station_id) = stations::current_station_id(&sector_state)
                        && stations
                            .0
                            .contract_by_index(station_id, docked_state.selected_contract_index)
                            .is_some()
                    {
                        log::debug!(
                            "Docked UI queued AcceptContract meta command for station {}, contract index {}",
                            station_id,
                            docked_state.selected_contract_index
                        );
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::AcceptContract,
                            arg0: docked_state.selected_contract_index as i16,
                            ..Default::default()
                        });
                    }
                } else if launch_contract.is_some() {
                    if progression.active_contract_id.is_some() {
                        log::debug!("Docked UI queued LaunchContract meta command");
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LaunchContract,
                            ..Default::default()
                        });
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if let Some(surface) = surface {
                    docked_surface_color(docked_state.selected_surface, surface.surface)
                } else if refit.is_some() {
                    NORMAL_BUTTON
                } else if open_map.is_some() {
                    Color::srgb(0.18, 0.50, 0.30)
                } else {
                    button_default_color(
                        repair.is_some(),
                        contract_prev.is_some()
                            || contract_next.is_some()
                            || contact_prev.is_some()
                            || contact_next.is_some()
                            || lore_prev.is_some()
                            || lore_next.is_some(),
                        accept_contract.is_some(),
                        launch_contract.is_some(),
                    )
                });
            }
        }
    }
}

pub(crate) fn update_docked_status_text(
    docked_state: Res<DockedState>,
    progression: Res<Progression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ship: Res<EditorShip>,
    local_profile: Res<LocalPlayerProfile>,
    mut query: Query<&mut Text, With<DockedStatusText>>,
) {
    if !docked_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
        && !editor_ship.is_changed()
        && !local_profile.is_changed()
    {
        return;
    }

    for mut text in &mut query {
        **text = docked_status_text(
            &docked_state,
            &progression,
            &editor_ship,
            &last_mission_report,
            &local_profile,
        );
    }
}

pub(crate) fn update_docked_surface_ui(
    docked_state: Res<DockedState>,
    progression: Res<Progression>,
    last_mission_report: Res<LastMissionReport>,
    sector_state: Res<SectorState>,
    stations: Res<StationCatalogResource>,
    mut content_query: Query<&mut Text, (With<DockedContentText>, Without<DockedHelpText>)>,
    mut help_query: Query<&mut Text, (With<DockedHelpText>, Without<DockedContentText>)>,
    mut surface_buttons: Query<(&DockedSurfaceButton, &mut BackgroundColor)>,
    mut action_buttons: Query<
        (
            Option<&RefitButton>,
            Option<&OpenSectorMapButton>,
            Option<&RepairShipButton>,
            Option<&DockedContractPrevButton>,
            Option<&DockedContractNextButton>,
            Option<&DockedAcceptContractButton>,
            Option<&DockedLaunchContractButton>,
            Option<&DockedContactPrevButton>,
            Option<&DockedContactNextButton>,
            Option<&DockedLorePrevButton>,
            Option<&DockedLoreNextButton>,
            Option<&DockedActionVisibility>,
            &mut Node,
            &mut BackgroundColor,
        ),
        (With<Button>, Without<DockedSurfaceButton>),
    >,
    mut action_rows: Query<(&DockedActionVisibility, &mut Node), Without<Button>>,
) {
    if !docked_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
        && !sector_state.is_changed()
    {
        return;
    }

    let station = stations::current_station(&stations.0, &sector_state);
    let content_text = docked_content_text(
        station,
        &docked_state,
        &progression,
        &last_mission_report,
        &sector_state,
    );
    for mut text in &mut content_query {
        **text = content_text.clone();
    }
    let help_text = docked_help_text(docked_state.selected_surface);
    for mut text in &mut help_query {
        **text = help_text.clone();
    }

    for (button, mut background) in &mut surface_buttons {
        *background = BackgroundColor(docked_surface_color(
            docked_state.selected_surface,
            button.surface,
        ));
    }

    for (
        refit,
        open_map,
        repair,
        contract_prev,
        contract_next,
        accept_contract,
        launch_contract,
        contact_prev,
        contact_next,
        lore_prev,
        lore_next,
        visibility,
        mut node,
        mut background,
    ) in &mut action_buttons
    {
        if let Some(visibility) = visibility {
            node.display = if visibility.visible_on(docked_state.selected_surface) {
                Display::Flex
            } else {
                Display::None
            };
        }
        *background = BackgroundColor(if refit.is_some() {
            NORMAL_BUTTON
        } else if open_map.is_some() || launch_contract.is_some() {
            Color::srgb(0.18, 0.50, 0.30)
        } else {
            button_default_color(
                repair.is_some(),
                contract_prev.is_some()
                    || contract_next.is_some()
                    || contact_prev.is_some()
                    || contact_next.is_some()
                    || lore_prev.is_some()
                    || lore_next.is_some(),
                accept_contract.is_some(),
                launch_contract.is_some(),
            )
        });
    }

    for (visibility, mut node) in &mut action_rows {
        node.display = if visibility.visible_on(docked_state.selected_surface) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn spawn_action_button<T: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    color: Color,
    marker: T,
    font: &Handle<Font>,
    visibility: DockedActionVisibility,
    selected_surface: DockedSurface,
) {
    parent
        .spawn((
            Button,
            Node {
                display: if visibility.visible_on(selected_surface) {
                    Display::Flex
                } else {
                    Display::None
                },
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(color),
            visibility,
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: UI_TITLE_FONT_SIZE - 2.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn spawn_dual_action_row<L: Bundle + 'static, R: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    left: (&str, L),
    right: (&str, R),
    font: &Handle<Font>,
    visibility: DockedActionVisibility,
    selected_surface: DockedSurface,
) {
    parent
        .spawn((
            Node {
                display: if visibility.visible_on(selected_surface) {
                    Display::Flex
                } else {
                    Display::None
                },
                width: Val::Percent(100.0),
                column_gap: Val::Px(10.0),
                ..default()
            },
            visibility,
        ))
        .with_children(|row| {
            spawn_half_width_action_button(row, left.0, left.1, font);
            spawn_half_width_action_button(row, right.0, right.1, font);
        });
}

fn spawn_half_width_action_button<T: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: T,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(50.0),
                height: Val::Px(42.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.30, 0.44)),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: UI_BODY_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn docked_status_text(
    docked_state: &DockedState,
    progression: &Progression,
    editor_ship: &EditorShip,
    last_mission_report: &LastMissionReport,
    local_profile: &LocalPlayerProfile,
) -> String {
    let mission = match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!("{headline}\n{detail}"),
        (Some(headline), None) => headline.clone(),
        _ => "No completed sorties yet.".to_string(),
    };

    let damaged_components = progression
        .damaged_components()
        .map(|entry| format!("{} x{}", entry.label(), entry.damaged))
        .collect::<Vec<_>>();

    format!(
        "Hub: {}\nCrew: {} ({})\nDefault Suit: {}\nScrap: {}\nDamaged Parts: {}\nHull Wear: {}\nJumps: {}\nShip: {}\nModules: {}\n\nLast Result\n{}",
        docked_state.station_title,
        local_profile.name,
        local_profile.role.as_str(),
        local_profile.starting_suit().as_str(),
        progression.scrap,
        if damaged_components.is_empty() {
            "none".to_string()
        } else {
            damaged_components.join(", ")
        },
        progression.hull_wear,
        progression.jump_count,
        editor_ship.ship.name,
        editor_ship.ship.modules.len(),
        mission
    )
}

fn docked_surface_color(selected: DockedSurface, surface: DockedSurface) -> Color {
    if selected == surface {
        SELECTED_BUTTON
    } else {
        Color::srgb(0.18, 0.24, 0.34)
    }
}

fn button_default_color(
    is_repair: bool,
    is_cycle: bool,
    is_accept: bool,
    is_launch: bool,
) -> Color {
    if is_repair {
        Color::srgb(0.45, 0.34, 0.16)
    } else if is_cycle {
        Color::srgb(0.22, 0.30, 0.44)
    } else if is_accept {
        Color::srgb(0.28, 0.46, 0.74)
    } else if is_launch {
        Color::srgb(0.18, 0.50, 0.30)
    } else {
        NORMAL_BUTTON
    }
}

fn docked_help_text(surface: DockedSurface) -> String {
    match surface {
        DockedSurface::Shipyard => "Dock Controls\nClick a hub surface to move around the station UI\nRefit: open shipyard refit\nRepair: spend scrap to clear hull wear\nSector Map: inspect the local route graph".to_string(),
        DockedSurface::Quartermaster => "Quartermaster Controls\nReview scrap, damaged components, and service availability\nRepair Ship spends scrap immediately when available".to_string(),
        DockedSurface::Contracts => "Contract Board Controls\nPrevious/Next: browse offers\nAccept: make the selected contract active\nLaunch: depart on the active contract".to_string(),
        DockedSurface::Archives => "Archives Controls\nBrowse station contacts and recovered lore entries\nViewing lore is local; progression unlocks are shared when earned".to_string(),
    }
}

fn docked_content_text(
    station: Option<&stations::StationDefinition>,
    docked_state: &DockedState,
    progression: &Progression,
    last_mission_report: &LastMissionReport,
    sector_state: &SectorState,
) -> String {
    let Some(station) = station else {
        return "No station record available for this dock.".to_string();
    };

    match docked_state.selected_surface {
        DockedSurface::Shipyard => {
            let repair_cost = progression.hull_wear.saturating_mul(2);
            format!(
                "{}\nFaction: {}\n\n{}\n\nServices: {}\n\nHull wear repair cost: {} scrap\nCurrent route node: {}\nLast report: {}",
                station.name,
                station.faction.as_str(),
                station.flavor,
                station
                    .services
                    .iter()
                    .map(|service| service.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                repair_cost,
                sector_state
                    .current_node()
                    .map(|node| node.label.as_str())
                    .unwrap_or("unknown"),
                last_mission_report
                    .headline
                    .clone()
                    .unwrap_or_else(|| "No mission report yet".to_string()),
            )
        }
        DockedSurface::Quartermaster => {
            let damaged = progression
                .damaged_components()
                .map(|entry| {
                    format!(
                        "{} x{} (repair {} scrap)",
                        entry.label(),
                        entry.damaged,
                        entry.repair_cost()
                    )
                })
                .collect::<Vec<_>>();
            format!(
                "Quartermaster Ledger\n\nScrap on hand: {}\nStored component stacks: {}\nDamaged inventory:\n{}\n\nQuartermaster note\n'Scrap is no good until it becomes pressure, wiring, or time.'",
                progression.scrap,
                progression.stored_components.len(),
                if damaged.is_empty() {
                    "  none".to_string()
                } else {
                    damaged
                        .into_iter()
                        .map(|line| format!("  {line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )
        }
        DockedSurface::Contracts => {
            let contract = station
                .contracts
                .get(docked_state.selected_contract_index % station.contracts.len().max(1));
            let active = progression
                .active_contract_id
                .as_ref()
                .map_or("none".to_string(), |id| id.clone());
            if let Some(contract) = contract {
                let contact_name = station
                    .contacts
                    .iter()
                    .find(|contact| contact.id == contract.contact_id)
                    .map(|contact| contact.name.as_str())
                    .unwrap_or("Unknown Contact");
                let status =
                    if progression.active_contract_id.as_deref() == Some(contract.id.as_str()) {
                        "ACTIVE"
                    } else if progression.contract_completed(&contract.id) {
                        "COMPLETED"
                    } else {
                        "AVAILABLE"
                    };
                format!(
                    "Contract Board\n\nOffer: {} [{}]\nContact: {}\nTarget Node: {}\nType: {}\nReward Bonus: {} scrap\nStatus: {}\n\nBriefing\n{}\n\nLaunch Note\n{}\n\nCurrent active contract: {}",
                    contract.title,
                    contract.id,
                    contact_name,
                    contract.target_node_id,
                    contract.kind.as_str(),
                    contract.reward_bonus_scrap,
                    status,
                    contract.briefing,
                    contract.launch_blurb,
                    active,
                )
            } else {
                "No contracts are currently posted.".to_string()
            }
        }
        DockedSurface::Archives => {
            let contact = station
                .contacts
                .get(docked_state.selected_contact_index % station.contacts.len().max(1));
            let lore = station
                .lore_entries
                .get(docked_state.selected_lore_index % station.lore_entries.len().max(1));
            let contact_block = contact.map_or_else(
                || "No contact selected.".to_string(),
                |contact| {
                    format!(
                        "Contact\n{} // {}\n{}\n\n'{}'",
                        contact.name, contact.role, contact.bio, contact.brief
                    )
                },
            );
            let lore_block = lore.map_or_else(
                || "No lore entry selected.".to_string(),
                |entry| {
                    if progression.lore_unlocked(&entry.id) {
                        format!("Lore\n{}\n{}", entry.title, entry.body)
                    } else {
                        format!(
                            "Lore\n{}\nEntry locked. Recover more field intel or complete station work to unlock it.",
                            entry.title
                        )
                    }
                },
            );
            format!("{contact_block}\n\n{lore_block}")
        }
    }
}

fn spawn_docked_ship_preview(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: ShipDefinition,
    ship_signature: u128,
) {
    if ship.modules.is_empty() {
        return;
    }

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for module in &ship.modules {
        min_x = min_x.min(module.grid_x);
        max_x = max_x.max(module.grid_x);
        min_y = min_y.min(module.grid_y);
        max_y = max_y.max(module.grid_y);
    }

    let center_x = (min_x + max_x) as f32 * 0.5;
    let center_y = (min_y + max_y) as f32 * 0.5;

    commands
        .spawn((
            Transform::from_xyz((TOOLBOX_WIDTH + 80.0) * 0.5, 0.0, 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            DockedPreviewRoot,
            DockedPreviewSignature(ship_signature),
        ))
        .with_children(|root| {
            for module in &ship.modules {
                root.spawn((
                    Sprite::from_image(
                        asset_server
                            .load(docked_sprite_path_for_kind(&module.kind, module.variant)),
                    ),
                    Transform {
                        translation: Vec3::new(
                            (module.grid_x as f32 - center_x) * TILE_SIZE,
                            -(module.grid_y as f32 - center_y) * TILE_SIZE,
                            0.1,
                        ),
                        rotation: Quat::from_rotation_z(
                            -(module.rotation_quadrants as f32) * std::f32::consts::FRAC_PI_2,
                        ),
                        scale: Vec3::splat(1.0),
                    },
                    DockedPreviewTile,
                ));
            }
        });
}

fn docked_sprite_path_for_kind(kind: &ModuleKind, variant: ModuleVariant) -> String {
    let _ = variant;
    match kind {
        ModuleKind::Turret => "tiles/hardpoint.png".to_string(),
        ModuleKind::Shield => "tiles/battery.png".to_string(),
        _ => format!("tiles/{}.png", kind.as_str()),
    }
}

fn docked_preview_ship(
    editor_ship: &EditorShip,
    status: &netcode::SessionStatus,
) -> ShipDefinition {
    if !editor_ship.ship.modules.is_empty() {
        return editor_ship.ship.clone();
    }
    if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
        return snapshot.clone();
    }
    match load_default_ship() {
        Ok(Some(ship)) => ship,
        Ok(None) | Err(_) => ShipDefinition::empty("Untitled Knot"),
    }
}

fn docked_preview_signature(ship: &ShipDefinition) -> u128 {
    let encoded = serde_json::to_vec(ship).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    encoded.hash(&mut hasher);
    hasher.finish() as u128
}
