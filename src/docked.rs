use bevy::{log, prelude::*};
use std::hash::{Hash, Hasher};

use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
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
        DemoProgression,
        DockedRoot,
        DockedState,
        DockedStatusText,
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

#[derive(Component)]
pub(crate) struct DockedPreviewRoot;

#[derive(Component)]
struct DockedPreviewTile;

#[derive(Component)]
pub(crate) struct DockedPreviewSignature(u128);

pub(crate) fn initialize_campaign_state(
    status: Res<netcode::SessionStatus>,
    rollback_state: Res<netcode::RollbackGameState>,
    mut campaign_load_state: ResMut<CampaignLoadState>,
    mut progression: ResMut<DemoProgression>,
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
            } else if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
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
    status: Res<netcode::SessionStatus>,
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
                    border_radius: BorderRadius::all(Val::Px(UI_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.09, 0.12, 0.18, 0.90)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Station Services\nRefit: adjust the ship layout and saved ARCH programs.\nSector Map: choose the next node on the local route graph.\nRepair Ship: spend scrap to clear persistent hull wear before the next jump.",
                    ),
                    TextFont {
                        font: mono_font,
                        font_size: UI_BODY_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
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
                    Text::new(
                        "Dock Controls\nEnter or click to choose a station action\nRefit: open ship editor\nSector Map: choose next route\nRepair: spend scrap to clear hull wear",
                    ),
                    TextFont {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: UI_HELP_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
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

    if let Some((_, existing_signature)) = existing_query.iter().next()
        && existing_signature.0 == ship_signature
    {
        return;
    }

    for (entity, _) in &existing_query {
        commands.entity(entity).despawn();
    }

    spawn_docked_ship_preview(&mut commands, &asset_server, ship, ship_signature);
}

pub(crate) fn rotate_docked_ship_preview(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DockedPreviewRoot>>,
) {
    for mut transform in &mut query {
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
            Option<&RefitButton>,
            Option<&OpenSectorMapButton>,
            Option<&RepairShipButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut editor_session: ResMut<EditorSessionState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background, refit, open_map, repair) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if refit.is_some() {
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
    parent: &mut ChildSpawnerCommands,
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
                font_size: UI_TITLE_FONT_SIZE - 2.0,
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
            Transform::from_xyz(360.0, 10.0, 0.0),
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
                        asset_server.load(docked_sprite_path_for_kind(&module.kind, module.variant)),
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
