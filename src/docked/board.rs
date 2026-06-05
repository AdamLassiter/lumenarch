use bevy::prelude::*;

use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_HELP_FONT_SIZE,
    UI_PANEL_RADIUS,
    UI_TITLE_FONT_SIZE,
    netcode::{self, PendingLocalMetaCommand, PendingMetaCommand, RollbackMetaOp},
    state::{DockedSurface, LastMissionReport, Progression, SectorNodeKind, SectorState},
    stations::{self, StationCatalogResource, StationContract},
};

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DockedBoardState {
    pub(crate) active_surface: Option<DockedSurface>,
    selected_contract_index: usize,
    selected_lore_index: usize,
    acknowledged_debrief_key: Option<String>,
}

#[derive(Component)]
pub(crate) struct DockedBoardRoot;

#[derive(Component)]
pub(crate) struct DockedBoardCloseButton;

#[derive(Component)]
pub(crate) struct DockedBoardContractButton {
    index: usize,
}

#[derive(Component)]
pub(crate) struct DockedBoardLoreButton {
    index: usize,
}

#[derive(Component)]
pub(crate) struct DockedBoardAcceptContractButton;

#[derive(Component)]
pub(crate) struct DockedBoardLaunchContractButton;

pub(crate) fn open_docked_board(
    board_state: &mut DockedBoardState,
    surface: DockedSurface,
    stations: &StationCatalogResource,
    sector_state: &SectorState,
    progression: &Progression,
) {
    board_state.active_surface = Some(surface);
    if let Some(station) = stations::current_station(&stations.0, sector_state) {
        board_state.selected_contract_index = board_state
            .selected_contract_index
            .min(station.contracts.len().saturating_sub(1));
        board_state.selected_lore_index = match surface {
            DockedSurface::Archives => first_unlocked_lore_index(station, progression)
                .unwrap_or(0)
                .min(station.lore_entries.len().saturating_sub(1)),
            _ => board_state
                .selected_lore_index
                .min(station.lore_entries.len().saturating_sub(1)),
        };
    }
}

/// Closes the local docked board with Q/Escape before docked movement or interaction shortcuts run.
pub(crate) fn docked_board_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    last_mission_report: Res<LastMissionReport>,
    mut board_state: ResMut<DockedBoardState>,
) {
    if board_state.active_surface.is_some()
        && (keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape))
    {
        close_docked_board(&mut board_state, &last_mission_report);
    }
}

/// Opens the docked debrief once for each new mission report so returns have an explicit acknowledgement.
pub(crate) fn open_docked_debrief_for_new_report(
    last_mission_report: Res<LastMissionReport>,
    mut board_state: ResMut<DockedBoardState>,
) {
    let Some(key) = debrief_key(&last_mission_report) else {
        return;
    };
    if board_state.active_surface.is_none()
        && board_state.acknowledged_debrief_key.as_deref() != Some(key.as_str())
    {
        board_state.active_surface = Some(DockedSurface::Debrief);
    }
}

/// Applies local board selection, close, accept, and launch button presses.
pub(crate) fn docked_board_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&DockedBoardCloseButton>,
            Option<&DockedBoardContractButton>,
            Option<&DockedBoardLoreButton>,
            Option<&DockedBoardAcceptContractButton>,
            Option<&DockedBoardLaunchContractButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    status: Res<netcode::SessionStatus>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    progression: Res<Progression>,
    last_mission_report: Res<LastMissionReport>,
    mut board_state: ResMut<DockedBoardState>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
) {
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    for (interaction, mut background, close, contract, lore, accept, launch) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                if close.is_some() {
                    close_docked_board(&mut board_state, &last_mission_report);
                } else if let Some(contract) = contract {
                    board_state.selected_contract_index = contract
                        .index
                        .min(station.contracts.len().saturating_sub(1));
                } else if let Some(lore) = lore {
                    board_state.selected_lore_index =
                        lore.index.min(station.lore_entries.len().saturating_sub(1));
                } else if accept.is_some() {
                    if netcode::is_host_authority(&status)
                        && selected_contract(station, board_state.selected_contract_index)
                            .is_some_and(|contract| contract_acceptable(contract, &progression))
                    {
                        pending_meta.0 = Some(PendingMetaCommand {
                            op: RollbackMetaOp::AcceptContract,
                            arg0: board_state.selected_contract_index as i16,
                            ..Default::default()
                        });
                    }
                } else if launch.is_some() {
                    if netcode::is_host_authority(&status)
                        && selected_contract(station, board_state.selected_contract_index)
                            .is_some_and(|contract| {
                                contract_launchable(contract, &progression, &sector_state)
                            })
                    {
                        pending_meta.0 = Some(PendingMetaCommand {
                            op: RollbackMetaOp::LaunchContract,
                            ..Default::default()
                        });
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }
}

/// Spawns, refreshes, or removes the local full-screen docked board presentation.
pub(crate) fn sync_docked_board_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    progression: Res<Progression>,
    last_mission_report: Res<LastMissionReport>,
    board_state: Res<DockedBoardState>,
    existing_query: Query<Entity, With<DockedBoardRoot>>,
) {
    let should_refresh = board_state.is_changed()
        || progression.is_changed()
        || last_mission_report.is_changed()
        || sector_state.is_changed()
        || board_state.active_surface.is_some() && existing_query.is_empty();
    if !should_refresh {
        return;
    }
    for entity in &existing_query {
        commands.entity(entity).despawn();
    }
    let Some(surface) = board_state.active_surface else {
        return;
    };
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    spawn_docked_board(
        &mut commands,
        &asset_server,
        surface,
        station,
        &sector_state,
        &progression,
        &last_mission_report,
        &board_state,
    );
}

/// Removes full-screen board UI when leaving docked presentation.
pub(crate) fn cleanup_docked_board_ui(
    mut commands: Commands,
    mut board_state: ResMut<DockedBoardState>,
    query: Query<Entity, With<DockedBoardRoot>>,
) {
    board_state.active_surface = None;
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn docked_board_closed(board_state: Res<DockedBoardState>) -> bool {
    board_state.active_surface.is_none()
}

pub(crate) fn docked_board_ui_present(query: Query<Entity, With<DockedBoardRoot>>) -> bool {
    !query.is_empty()
}

fn spawn_docked_board(
    commands: &mut Commands,
    asset_server: &AssetServer,
    surface: DockedSurface,
    station: &crate::stations::StationDefinition,
    sector_state: &SectorState,
    progression: &Progression,
    last_mission_report: &LastMissionReport,
    board_state: &DockedBoardState,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let title = match surface {
        DockedSurface::Contracts => "Contract Board",
        DockedSurface::Archives => "Archives",
        _ => surface.as_str(),
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                column_gap: Val::Px(18.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.96)),
            DockedBoardRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(360.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(format!("{title}\n{}", station.name)),
                    TextFont {
                        font: title_font.clone(),
                        font_size: UI_TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                match surface {
                    DockedSurface::Contracts => spawn_contract_list(
                        panel,
                        &mono_font,
                        station,
                        progression,
                        board_state.selected_contract_index,
                    ),
                    DockedSurface::Archives => spawn_lore_list(
                        panel,
                        &mono_font,
                        station,
                        progression,
                        board_state.selected_lore_index,
                    ),
                    DockedSurface::Debrief => {}
                    _ => {}
                }
                board_button(
                    panel,
                    "Back To Station",
                    Color::srgb(0.32, 0.24, 0.18),
                    &title_font,
                    DockedBoardCloseButton,
                );
            });

            root.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    row_gap: Val::Px(14.0),
                    flex_direction: FlexDirection::Column,
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.09, 0.13, 0.94)),
            ))
            .with_children(|detail| match surface {
                DockedSurface::Contracts => spawn_contract_detail(
                    detail,
                    &title_font,
                    &mono_font,
                    station,
                    sector_state,
                    progression,
                    board_state.selected_contract_index,
                ),
                DockedSurface::Archives => spawn_lore_detail(
                    detail,
                    &title_font,
                    &mono_font,
                    station,
                    progression,
                    board_state.selected_lore_index,
                ),
                DockedSurface::Debrief => {
                    spawn_debrief_detail(detail, &title_font, &mono_font, last_mission_report)
                }
                _ => {}
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    bottom: Val::Px(24.0),
                    display: Display::None,
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
            ))
            .with_child((
                Text::new(
                    "Board Controls\nSelect an entry from the left list\nQ: return to station",
                ),
                TextFont {
                    font: mono_font,
                    font_size: UI_HELP_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.86, 0.92)),
            ));
        });
}

fn spawn_contract_list(
    panel: &mut ChildSpawnerCommands<'_>,
    font: &Handle<Font>,
    station: &crate::stations::StationDefinition,
    progression: &Progression,
    selected_index: usize,
) {
    if station.contracts.is_empty() {
        panel.spawn((
            Text::new("No contracts posted."),
            TextFont {
                font: font.clone(),
                font_size: UI_BODY_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.86, 0.92)),
        ));
        return;
    }
    for (index, contract) in station.contracts.iter().enumerate() {
        let selected = index == selected_index.min(station.contracts.len().saturating_sub(1));
        let active = progression.active_contract_id.as_deref() == Some(contract.id.as_str());
        let completed = progression.contract_completed(&contract.id);
        let suffix = if completed {
            "Complete"
        } else if active {
            "Active"
        } else {
            "Posted"
        };
        board_button(
            panel,
            &format!("{}\n{}", contract.title, suffix),
            if selected {
                Color::srgb(0.28, 0.46, 0.74)
            } else {
                Color::srgb(0.20, 0.30, 0.44)
            },
            font,
            DockedBoardContractButton { index },
        );
    }
}

fn spawn_lore_list(
    panel: &mut ChildSpawnerCommands<'_>,
    font: &Handle<Font>,
    station: &crate::stations::StationDefinition,
    progression: &Progression,
    selected_index: usize,
) {
    if station.lore_entries.is_empty() {
        panel.spawn((
            Text::new("No archived entries."),
            TextFont {
                font: font.clone(),
                font_size: UI_BODY_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.86, 0.92)),
        ));
        return;
    }
    for (index, entry) in station.lore_entries.iter().enumerate() {
        let selected = index == selected_index.min(station.lore_entries.len().saturating_sub(1));
        let unlocked = progression.lore_unlocked(&entry.id);
        let title = if unlocked {
            entry.title.as_str()
        } else {
            "Locked Archive"
        };
        board_button(
            panel,
            title,
            if selected {
                Color::srgb(0.28, 0.46, 0.74)
            } else if unlocked {
                Color::srgb(0.20, 0.30, 0.44)
            } else {
                Color::srgb(0.16, 0.18, 0.22)
            },
            font,
            DockedBoardLoreButton { index },
        );
    }
}

fn spawn_contract_detail(
    panel: &mut ChildSpawnerCommands<'_>,
    title_font: &Handle<Font>,
    mono_font: &Handle<Font>,
    station: &crate::stations::StationDefinition,
    sector_state: &SectorState,
    progression: &Progression,
    selected_index: usize,
) {
    let Some(contract) = selected_contract(station, selected_index) else {
        board_text(panel, "No contract selected.", mono_font, UI_BODY_FONT_SIZE);
        return;
    };
    let active = progression.active_contract_id.as_deref() == Some(contract.id.as_str());
    let completed = progression.contract_completed(&contract.id);
    let reachable = contract_target_reachable(contract, sector_state);
    let status = if completed {
        "Complete"
    } else if active {
        "Active"
    } else {
        "Available"
    };
    panel.spawn((
        Text::new(format!("{}\n{}", contract.title, status)),
        TextFont {
            font: title_font.clone(),
            font_size: UI_TITLE_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
    board_text(
        panel,
        &format!(
            "Kind: {}\nTarget Node: {}\nRequired: {}\nReward Bonus: {} scrap\nRoute: {}\n\n{}\n\nLaunch Brief:\n{}",
            contract.kind.as_str(),
            contract.target_node_id,
            contract
                .required_artifact
                .map(|artifact| artifact.label())
                .unwrap_or("none"),
            contract.reward_bonus_scrap,
            if reachable {
                "Reachable"
            } else {
                "Not reachable"
            },
            contract.briefing,
            contract.launch_blurb
        ),
        mono_font,
        UI_BODY_FONT_SIZE,
    );
    board_button(
        panel,
        if completed {
            "Contract Complete"
        } else if active {
            "Contract Active"
        } else {
            "Accept Contract"
        },
        if contract_acceptable(contract, progression) {
            Color::srgb(0.28, 0.46, 0.74)
        } else {
            Color::srgb(0.16, 0.18, 0.22)
        },
        title_font,
        DockedBoardAcceptContractButton,
    );
    board_button(
        panel,
        "Launch Contract",
        if contract_launchable(contract, progression, sector_state) {
            Color::srgb(0.18, 0.50, 0.30)
        } else {
            Color::srgb(0.16, 0.18, 0.22)
        },
        title_font,
        DockedBoardLaunchContractButton,
    );
}

fn spawn_lore_detail(
    panel: &mut ChildSpawnerCommands<'_>,
    title_font: &Handle<Font>,
    mono_font: &Handle<Font>,
    station: &crate::stations::StationDefinition,
    progression: &Progression,
    selected_index: usize,
) {
    let Some(entry) = station.lore_entries.get(selected_index) else {
        board_text(panel, "No archive selected.", mono_font, UI_BODY_FONT_SIZE);
        return;
    };
    let unlocked = progression.lore_unlocked(&entry.id);
    panel.spawn((
        Text::new(if unlocked {
            entry.title.clone()
        } else {
            "Locked Archive".to_string()
        }),
        TextFont {
            font: title_font.clone(),
            font_size: UI_TITLE_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
    board_text(
        panel,
        if unlocked {
            &entry.body
        } else {
            "This archive entry has not been recovered yet. Complete related station work to unlock it."
        },
        mono_font,
        UI_BODY_FONT_SIZE,
    );
}

fn spawn_debrief_detail(
    panel: &mut ChildSpawnerCommands<'_>,
    title_font: &Handle<Font>,
    mono_font: &Handle<Font>,
    last_mission_report: &LastMissionReport,
) {
    let headline = last_mission_report
        .headline
        .as_deref()
        .unwrap_or("No completed sorties yet");
    panel.spawn((
        Text::new(format!("Debrief\n{headline}")),
        TextFont {
            font: title_font.clone(),
            font_size: UI_TITLE_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
    let artifacts = if last_mission_report.recovered_artifacts.is_empty() {
        "Recovered artifacts: none".to_string()
    } else {
        format!(
            "Recovered artifacts: {}",
            last_mission_report
                .recovered_artifacts
                .iter()
                .map(|artifact| artifact.label())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let unlocked_archives = last_mission_report
        .recovered_artifacts
        .iter()
        .filter_map(|artifact| artifact.archive_title())
        .collect::<Vec<_>>();
    let archive_line = if unlocked_archives.is_empty() {
        "Unlocked Archives: none".to_string()
    } else {
        format!("Unlocked Archives: {}", unlocked_archives.join(", "))
    };
    let objective = last_mission_report
        .contract_objective_status
        .as_deref()
        .unwrap_or("Objective: n/a");
    let detail = last_mission_report
        .detail
        .as_deref()
        .unwrap_or("Return from a sortie to populate this debrief.");
    board_text(
        panel,
        &format!(
            "{detail}\n\n{objective}\n{artifacts}\n{archive_line}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded, last_mission_report.total_scrap
        ),
        mono_font,
        UI_BODY_FONT_SIZE,
    );
}

fn close_docked_board(board_state: &mut DockedBoardState, last_mission_report: &LastMissionReport) {
    if board_state.active_surface == Some(DockedSurface::Debrief) {
        board_state.acknowledged_debrief_key = debrief_key(last_mission_report);
    }
    board_state.active_surface = None;
}

fn debrief_key(last_mission_report: &LastMissionReport) -> Option<String> {
    last_mission_report.headline.as_ref().map(|headline| {
        format!(
            "{headline}|{}",
            last_mission_report.detail.as_deref().unwrap_or("")
        )
    })
}

fn board_button<T: Component>(
    panel: &mut ChildSpawnerCommands<'_>,
    label: &str,
    color: Color,
    font: &Handle<Font>,
    marker: T,
) {
    panel
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                padding: UiRect::all(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(color),
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

fn board_text(
    panel: &mut ChildSpawnerCommands<'_>,
    text: &str,
    font: &Handle<Font>,
    font_size: f32,
) {
    panel.spawn((
        Text::new(text),
        TextFont {
            font: font.clone(),
            font_size,
            ..default()
        },
        TextColor(Color::srgb(0.82, 0.86, 0.92)),
    ));
}

fn selected_contract(
    station: &crate::stations::StationDefinition,
    selected_index: usize,
) -> Option<&StationContract> {
    station.contracts.get(selected_index)
}

fn contract_acceptable(contract: &StationContract, progression: &Progression) -> bool {
    progression.active_contract_id.as_deref() != Some(contract.id.as_str())
        && !progression.contract_completed(&contract.id)
}

fn contract_launchable(
    contract: &StationContract,
    progression: &Progression,
    sector_state: &SectorState,
) -> bool {
    progression.active_contract_id.as_deref() == Some(contract.id.as_str())
        && contract_target_reachable(contract, sector_state)
}

fn contract_target_reachable(contract: &StationContract, sector_state: &SectorState) -> bool {
    sector_state.is_reachable(contract.target_node_id)
        && sector_state
            .node(contract.target_node_id)
            .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
            .unwrap_or(false)
}

fn first_unlocked_lore_index(
    station: &crate::stations::StationDefinition,
    progression: &Progression,
) -> Option<usize> {
    station
        .lore_entries
        .iter()
        .position(|entry| progression.lore_unlocked(&entry.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stations::{FactionId, LoreEntry, StationDefinition};

    fn test_station() -> StationDefinition {
        StationDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            faction: FactionId::ContinuantGuild,
            flavor: String::new(),
            ship: Default::default(),
            contacts: Vec::new(),
            services: Vec::new(),
            contracts: Vec::new(),
            lore_entries: vec![
                LoreEntry {
                    id: "locked".to_string(),
                    title: "Locked".to_string(),
                    body: "Hidden".to_string(),
                },
                LoreEntry {
                    id: "unlocked".to_string(),
                    title: "Unlocked".to_string(),
                    body: "Visible".to_string(),
                },
            ],
        }
    }

    #[test]
    fn archives_default_to_first_unlocked_lore() {
        let station = test_station();
        let mut progression = Progression::default();
        progression.unlocked_lore_ids = vec!["unlocked".to_string()];

        assert_eq!(first_unlocked_lore_index(&station, &progression), Some(1));
    }

    #[test]
    fn completed_contracts_are_not_acceptable() {
        let mut progression = Progression::default();
        let contract = StationContract {
            id: "done".to_string(),
            title: "Done".to_string(),
            contact_id: "contact".to_string(),
            target_node_id: 1,
            kind: crate::stations::StationContractKind::Calibration,
            briefing: String::new(),
            launch_blurb: String::new(),
            success_debrief: String::new(),
            failure_debrief: String::new(),
            reward_bonus_scrap: 0,
            lore_unlock_ids: Vec::new(),
            required_artifact: None,
        };
        progression.complete_contract("done");

        assert!(!contract_acceptable(&contract, &progression));
    }
}
