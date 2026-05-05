mod arena;
mod hud;
mod salvage;

use arena::spawn_test_arena;
use bevy::{log, prelude::*};
use hud::spawn_runtime_hud;
use salvage::spawn_salvage_wreck;

use super::ship::{default_hostile_identity, spawn_hostile_ship, spawn_runtime_ship};
use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{HostileShip, HostileShipModule, HostileTurretPlatform, ShipRoot},
        helpers::{FixedVec2, Fx},
    },
    netcode,
    ship::enemy::EnemyShipEntryValidationStatus,
    state::{DemoProgression, EditorShip, EnemyShipLibraryState, SectorNodeKind, SectorState},
    stations::StationCatalogResource,
};

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    enemy_library_state: Res<EnemyShipLibraryState>,
    station_catalog: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    session_status: Res<netcode::SessionStatus>,
    local_handle: Res<netcode::LocalPlayerHandle>,
    mut player_handle_map: ResMut<netcode::PlayerHandleMap>,
    mut observed_local_player: ResMut<netcode::ObservedLocalPlayer>,
    balance: Res<BalanceConfig>,
) {
    player_handle_map.entities.clear();
    observed_local_player.entity = None;
    observed_local_player.handle = local_handle.0;

    log::info!(
        "Spawning runtime encounter scene: local_handle={:?}, total_players={}, current_node={}, active_node={:?}, selected_node={:?}",
        local_handle.0,
        session_status.total_players,
        sector_state.current_node_id,
        sector_state.active_encounter_node_id,
        sector_state.selected_node_id
    );

    spawn_runtime_hud(&mut commands, &asset_server, &editor_ship.ship);
    let active_node = sector_state
        .active_node()
        .or_else(|| sector_state.selected_node())
        .cloned()
        .unwrap_or_else(|| sector_state.nodes[1].clone());
    log::debug!(
        "Runtime encounter source node: id={}, label='{}', hostile_count={}, enemy_ship_ids={:?}, salvage_value={}, reward_multiplier={}",
        active_node.id,
        active_node.label,
        active_node.encounter.hostile_count,
        active_node.encounter.enemy_ship_ids,
        active_node.encounter.salvage_value,
        active_node.encounter.reward_multiplier
    );
    let platform_hostile_count = if matches!(active_node.kind, SectorNodeKind::TestRange) {
        active_node.encounter.hostile_count
    } else {
        0
    };
    log::info!(
        "Encounter hostile spawn path: node_id={}, node_kind={}, ship_hostiles={}, platform_hostiles={}",
        active_node.id,
        active_node.kind.as_str(),
        active_node.encounter.enemy_ship_ids.len(),
        platform_hostile_count
    );
    spawn_test_arena(
        &mut commands,
        &balance,
        &active_node.encounter.arena_variant,
        platform_hostile_count,
        active_node.encounter.ambient_heat_pressure,
        active_node.encounter.ambient_electrical_pressure,
    );
    spawn_salvage_wreck(&mut commands, active_node.encounter.salvage_value);
    let active_contract = progression
        .active_contract_id
        .as_ref()
        .and_then(|contract_id| station_catalog.0.contract(contract_id));
    let mission_briefing = active_contract
        .as_ref()
        .map(|(_, contract)| contract.briefing.clone());
    let contract_title = active_contract
        .as_ref()
        .map(|(_, contract)| contract.title.clone());
    let opposition_identity = active_node
        .encounter
        .enemy_ship_ids
        .first()
        .and_then(|enemy_id| enemy_library_state.library.find_by_id(enemy_id))
        .map(|entry| {
            if entry.is_crewed {
                format!(
                    "{} | {} | {}",
                    entry.faction_id.as_str(),
                    entry.captain_name.as_deref().unwrap_or("Unknown Captain"),
                    entry
                        .ship_name
                        .as_deref()
                        .unwrap_or(entry.display_name.as_str())
                )
            } else {
                format!(
                    "{} | {}",
                    entry.faction_id.as_str(),
                    entry
                        .ship_name
                        .as_deref()
                        .unwrap_or(entry.display_name.as_str())
                )
            }
        });
    let opposition_comms = active_node
        .encounter
        .enemy_ship_ids
        .first()
        .and_then(|enemy_id| enemy_library_state.library.find_by_id(enemy_id))
        .and_then(|entry| entry.comms_intro.clone());
    spawn_runtime_ship(
        &mut commands,
        &asset_server,
        &editor_ship.ship,
        &(0..session_status.total_players.max(1)).collect::<Vec<_>>(),
        local_handle.0,
        &mut player_handle_map,
        &mut observed_local_player,
        session_status.lobby_snapshot.as_ref(),
        &balance,
        active_node.id,
        &active_node.label,
        active_node.kind.as_str(),
        active_node.encounter.reward_multiplier,
        active_node.encounter.ambient_heat_pressure,
        active_node.encounter.ambient_electrical_pressure,
        progression.hull_wear,
        progression.active_contract_id.clone(),
        contract_title,
        mission_briefing,
        opposition_identity,
        opposition_comms,
    );

    let spawn_points = [
        FixedVec2::from_num(-220.0, 120.0),
        FixedVec2::from_num(210.0, 40.0),
        FixedVec2::from_num(160.0, -150.0),
        FixedVec2::from_num(-120.0, -150.0),
    ];
    for (index, enemy_id) in active_node.encounter.enemy_ship_ids.iter().enumerate() {
        let Some(entry) = enemy_library_state.library.find_by_id(enemy_id) else {
            log::warn!(
                "Encounter node {} references unknown enemy ship id '{}'; skipping hostile spawn",
                active_node.id,
                enemy_id
            );
            continue;
        };
        match enemy_library_state
            .entry_statuses
            .get(enemy_id)
            .copied()
            .unwrap_or(EnemyShipEntryValidationStatus::Valid)
        {
            EnemyShipEntryValidationStatus::Invalid => {
                log::warn!(
                    "Encounter node {} references invalid enemy ship id '{}'; skipping hostile spawn",
                    active_node.id,
                    enemy_id
                );
                continue;
            }
            EnemyShipEntryValidationStatus::RepairedInMemory => {
                log::debug!(
                    "Encounter node {} is using repaired in-memory enemy ship '{}'",
                    active_node.id,
                    enemy_id
                );
            }
            EnemyShipEntryValidationStatus::Valid => {}
        }
        let spawn_position = spawn_points
            .get(index)
            .copied()
            .unwrap_or_else(|| FixedVec2::from_num(180.0 + index as f32 * 40.0, 90.0));
        log::debug!(
            "Spawning hostile ship '{}' (modules={}, threat={}) at {:?}",
            entry.id,
            entry.ship.modules.len(),
            entry.threat_tier,
            spawn_position
        );
        let preferred_range = match entry.behavior_tag.as_str() {
            "brawler" => Fx::from_num(balance.hostile_ai.brawler_preferred_range),
            "skirmisher" => Fx::from_num(balance.hostile_ai.skirmisher_preferred_range),
            _ => Fx::from_num(balance.hostile_ai.default_preferred_range),
        };
        spawn_hostile_ship(
            &mut commands,
            &asset_server,
            &entry.ship,
            &balance,
            spawn_position,
            preferred_range,
            Fx::from_num(balance.hostile_ai.default_aggression),
            balance.hostile_ai.salvage_reward_base
                + u32::from(entry.threat_tier) * balance.hostile_ai.salvage_reward_per_threat,
            Some(if entry.is_crewed {
                crate::gameplay::components::ShipEncounterIdentity {
                    faction_id: entry.faction_id,
                    ship_name: entry
                        .ship_name
                        .clone()
                        .unwrap_or_else(|| entry.display_name.clone()),
                    captain: crate::gameplay::components::CaptainProfile {
                        name: entry
                            .captain_name
                            .clone()
                            .unwrap_or_else(|| "Unknown Captain".to_string()),
                        title: "Commanding".to_string(),
                    },
                    comms: crate::gameplay::components::EncounterCommsScript {
                        intro: entry
                            .comms_intro
                            .clone()
                            .unwrap_or_else(|| "Hostile comms burst detected.".to_string()),
                        outro: entry
                            .comms_outro
                            .clone()
                            .unwrap_or_else(|| "Hostile signal lost.".to_string()),
                    },
                    crewed: true,
                }
            } else {
                default_hostile_identity(
                    entry
                        .ship_name
                        .as_deref()
                        .unwrap_or(entry.display_name.as_str()),
                )
            }),
        );
    }

    if active_node.encounter.enemy_ship_ids.is_empty() && platform_hostile_count == 0 {
        log::warn!(
            "Encounter node {} ('{}') has no ship hostiles and no platform hostiles configured",
            active_node.id,
            active_node.label
        );
    }
}

pub(crate) fn log_runtime_hostile_scene_summary(
    rollback_state: Res<netcode::RollbackGameState>,
    hostile_root_query: Query<Entity, (With<HostileShip>, With<ShipRoot>)>,
    hostile_module_query: Query<Entity, With<HostileShipModule>>,
    hostile_platform_query: Query<Entity, With<HostileTurretPlatform>>,
    mut last_logged_scene_generation: Local<Option<u32>>,
) {
    if rollback_state.phase != netcode::RollbackPhase::Encounter {
        *last_logged_scene_generation = None;
        return;
    }

    if *last_logged_scene_generation == Some(rollback_state.scene_generation) {
        return;
    }

    let hostile_root_count = hostile_root_query.iter().count();
    let hostile_module_count = hostile_module_query.iter().count();
    let hostile_platform_count = hostile_platform_query.iter().count();

    log::info!(
        "Encounter scene {} hostile summary: hostile_roots={}, hostile_modules={}, hostile_platforms={}",
        rollback_state.scene_generation,
        hostile_root_count,
        hostile_module_count,
        hostile_platform_count
    );

    if hostile_root_count == 0 && hostile_platform_count == 0 {
        log::warn!(
            "Encounter scene {} spawned no hostile ships or hostile platforms",
            rollback_state.scene_generation
        );
    }

    *last_logged_scene_generation = Some(rollback_state.scene_generation);
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    mut player_handle_map: ResMut<netcode::PlayerHandleMap>,
    mut observed_local_player: ResMut<netcode::ObservedLocalPlayer>,
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) {
    player_handle_map.entities.clear();
    observed_local_player.entity = None;
    let entity_count = query.iter().count();
    log::info!(
        "Cleaning up runtime encounter scene and {} presentation/runtime entities",
        entity_count
    );
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn runtime_scene_missing(
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) -> bool {
    query.is_empty()
}

pub(crate) fn runtime_scene_present(
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) -> bool {
    !query.is_empty()
}
