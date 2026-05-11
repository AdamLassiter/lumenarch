use bevy::{log, prelude::*};

mod campaign;
mod preview_helpers;
mod preview_sync;
mod ui_helpers;

pub(crate) use campaign::{initialize_campaign_state, persist_campaign_state};
pub(crate) use preview_sync::{
    cleanup_docked_ui,
    docked_ui_missing,
    docked_ui_present,
    rotate_docked_ship_preview,
    sync_docked_ship_preview,
};

use self::{
    preview_helpers::{docked_preview_ship, docked_preview_signature, spawn_docked_ship_preview},
    ui_helpers::{
        button_default_color,
        docked_content_text,
        docked_help_text,
        docked_status_text,
        docked_surface_color,
        spawn_action_button,
        spawn_dual_action_row,
    },
};
use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    TOOLBOX_WIDTH,
    UI_BODY_FONT_SIZE,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    netcode,
    state::{
        ControlsHelpPanel,
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

#[derive(Component)]
pub(crate) struct DockedPreviewRoot;

#[derive(Component, Clone)]
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

/// Spawns the docked station UI so the player can refit, resupply, and browse station content.
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
                    display: Display::None,
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.90)),
                ControlsHelpPanel,
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

/// Handles docked-station button presses and turns them into surface changes or session actions.
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
                } else if launch_contract.is_some() && progression.active_contract_id.is_some() {
                    log::debug!("Docked UI queued LaunchContract meta command");
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::LaunchContract,
                        ..Default::default()
                    });
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

/// Refreshes the docked summary text so station, ship, and campaign status stay readable.
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

/// Swaps docked surface content and action visibility to match the currently selected station tab.
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
