use bevy::{log, prelude::*};

use crate::{
    campaign::{CampaignSave, load_campaign, save_campaign},
    netcode,
    ship::{
        ShipDefinition,
        enemy::load_validated_default_enemy_library,
        storage::load_default_ship,
    },
    state::{
        CampaignLoadState,
        DockedState,
        EditorShip,
        EnemyShipLibraryState,
        LastMissionReport,
        Progression,
        SectorState,
    },
    stations::{self, StationCatalogResource},
};

/// Initializes campaign-facing docked state from progression and station data when docking begins.
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

    if let Some(current_node) = sector_state.current_node()
        && docked_state.station_title != current_node.label
    {
        docked_state.station_title = current_node.label.clone();
    }
}

/// Persists docked campaign selections back into resources so station browsing survives updates.
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
