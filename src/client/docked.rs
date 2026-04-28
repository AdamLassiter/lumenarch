use bevy::prelude::*;

use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    TOOLBOX_WIDTH,
    campaign::{CampaignSave, load_campaign, save_campaign},
    state::{
        CampaignLoadState,
        ClientAppState,
        ConnectionStatus,
        DemoProgression,
        DockedRoot,
        DockedState,
        DockedStatusText,
        EditingCleanup,
        EditorMode,
        EditorSessionState,
        EditorShip,
        EnemyShipLibraryState,
        LastMissionReport,
        OpenSectorMapButton,
        RefitButton,
        RepairShipButton,
        SectorState,
    },
};
use crate::ship::{ShipDefinition, enemy::load_default_enemy_library, storage::load_default_ship};

pub(crate) fn initialize_campaign_state(
    status: Res<ConnectionStatus>,
    mut campaign_load_state: ResMut<CampaignLoadState>,
    mut progression: ResMut<DemoProgression>,
    mut sector_state: ResMut<SectorState>,
    mut docked_state: ResMut<DockedState>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if !campaign_load_state.hydrated {
        match load_campaign() {
            Ok(Some(save)) => {
                *progression = save.progression;
                *sector_state = save.sector;
                sector_state.ensure_latest_layout();
                *last_mission_report = save.last_mission_report;
            }
            Ok(None) => {
                *progression = DemoProgression::default();
                *sector_state = SectorState::default();
                *last_mission_report = LastMissionReport::default();
            }
            Err(error) => {
                eprintln!("campaign: failed to load save state: {error}");
                *progression = DemoProgression::default();
                *sector_state = SectorState::default();
                *last_mission_report = LastMissionReport::default();
            }
        }

        if let Ok(Some(saved_ship)) = load_default_ship() {
            editor_ship.ship = saved_ship;
        } else if let Some(snapshot) = status.active_snapshot.as_ref() {
            editor_ship.ship = snapshot.clone();
        } else if editor_ship.ship.name.is_empty() && editor_ship.ship.modules.is_empty() {
            editor_ship.ship = ShipDefinition::empty("Untitled Knot");
        }

        if let Ok(Some(library)) = load_default_enemy_library() {
            enemy_library_state.library = library;
            enemy_library_state.library.ensure_seeded();
        }

        campaign_load_state.hydrated = true;
    }

    if let Some(current_node) = sector_state.current_node() {
        docked_state.station_title = current_node.label.clone();
    }
}

pub(crate) fn persist_campaign_state(
    campaign_load_state: Res<CampaignLoadState>,
    progression: Res<DemoProgression>,
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
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ship: Res<EditorShip>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

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
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
                EditingCleanup,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(format!("Docked At\n{}", docked_state.station_title)),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 30.0,
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
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.90, 0.93, 0.98)),
                    DockedStatusText,
                ));

                spawn_action_button(
                    panel,
                    "Refit Ship",
                    NORMAL_BUTTON,
                    RefitButton,
                    &title_font,
                );
                spawn_action_button(
                    panel,
                    "Open Sector Map",
                    Color::srgb(0.18, 0.50, 0.30),
                    OpenSectorMapButton,
                    &title_font,
                );
                spawn_action_button(
                    panel,
                    "Repair Ship",
                    Color::srgb(0.45, 0.34, 0.16),
                    RepairShipButton,
                    &title_font,
                );
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(TOOLBOX_WIDTH + 120.0),
                    top: Val::Px(48.0),
                    width: Val::Px(520.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.09, 0.12, 0.18, 0.90)),
                BorderRadius::all(Val::Px(12.0)),
                EditingCleanup,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Station Services\nRefit: adjust the ship layout and saved ARCH programs.\nSector Map: choose the next node on the local route graph.\nRepair Ship: spend scrap to clear persistent hull wear before the next jump.",
                    ),
                    TextFont {
                        font: mono_font,
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
}

pub(crate) fn cleanup_docked_ui(mut commands: Commands, query: Query<Entity, With<DockedRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn docked_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&RefitButton>,
            Option<&OpenSectorMapButton>,
            Option<&RepairShipButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut progression: ResMut<DemoProgression>,
    mut last_mission_report: ResMut<LastMissionReport>,
    mut editor_session: ResMut<EditorSessionState>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    for (interaction, mut background, refit, open_map, repair) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if refit.is_some() {
                    *background = BackgroundColor(PRESSED_BUTTON);
                    editor_session.mode = EditorMode::Player;
                    next_state.set(ClientAppState::Editing);
                } else if open_map.is_some() {
                    *background = BackgroundColor(Color::srgb(0.12, 0.40, 0.24));
                    next_state.set(ClientAppState::SectorMap);
                } else if repair.is_some() {
                    *background = BackgroundColor(Color::srgb(0.62, 0.44, 0.16));
                    let repair_cost = progression.hull_wear.saturating_mul(2);
                    if repair_cost > 0 && progression.scrap >= repair_cost {
                        progression.scrap -= repair_cost;
                        progression.hull_wear = 0;
                        last_mission_report.detail = Some(format!(
                            "Station service restored your ship for {repair_cost} scrap."
                        ));
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if refit.is_some() {
                    NORMAL_BUTTON
                } else if open_map.is_some() {
                    Color::srgb(0.18, 0.50, 0.30)
                } else {
                    Color::srgb(0.45, 0.34, 0.16)
                });
            }
        }
    }
}

pub(crate) fn update_docked_status_text(
    docked_state: Res<DockedState>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    editor_ship: Res<EditorShip>,
    mut query: Query<&mut Text, With<DockedStatusText>>,
) {
    if !docked_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
        && !editor_ship.is_changed()
    {
        return;
    }

    for mut text in &mut query {
        **text = docked_status_text(
            &docked_state,
            &progression,
            &editor_ship,
            &last_mission_report,
        );
    }
}

fn spawn_action_button<T: Bundle + 'static>(
    parent: &mut ChildBuilder,
    label: &str,
    color: Color,
    marker: T,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(10.0)),
            BackgroundColor(color),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn docked_status_text(
    docked_state: &DockedState,
    progression: &DemoProgression,
    editor_ship: &EditorShip,
    last_mission_report: &LastMissionReport,
) -> String {
    let mission = match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!("{headline}\n{detail}"),
        (Some(headline), None) => headline.clone(),
        _ => "No completed sorties yet.".to_string(),
    };

    format!(
        "Hub: {}\nScrap: {}\nHull Wear: {}\nJumps: {}\nShip: {}\nModules: {}\n\nLast Result\n{}",
        docked_state.station_title,
        progression.scrap,
        progression.hull_wear,
        progression.jump_count,
        editor_ship.ship.name,
        editor_ship.ship.modules.len(),
        mission
    )
}
