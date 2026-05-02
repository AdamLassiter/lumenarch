mod arena;
mod hud;
mod salvage;

use arena::spawn_test_arena;
use bevy::prelude::*;
use hud::spawn_runtime_hud;
use salvage::spawn_salvage_wreck;

use super::ship::{spawn_hostile_ship, spawn_runtime_ship};
use crate::{
    balance::BalanceConfig,
    gameplay::helpers::{FixedVec2, Fx},
    netcode,
    state::{DemoProgression, EditorShip, EnemyShipLibraryState, SectorState},
};

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    enemy_library_state: Res<EnemyShipLibraryState>,
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

    spawn_runtime_hud(&mut commands, &asset_server, &editor_ship.ship);
    let active_node = sector_state
        .active_node()
        .or_else(|| sector_state.selected_node())
        .cloned()
        .unwrap_or_else(|| sector_state.nodes[1].clone());
    spawn_test_arena(
        &mut commands,
        &balance,
        &active_node.encounter.arena_variant,
        if active_node.encounter.enemy_ship_ids.is_empty() {
            active_node.encounter.hostile_count
        } else {
            0
        },
        active_node.encounter.ambient_heat_pressure,
        active_node.encounter.ambient_electrical_pressure,
    );
    spawn_salvage_wreck(&mut commands, active_node.encounter.salvage_value);
    spawn_runtime_ship(
        &mut commands,
        &asset_server,
        &editor_ship.ship,
        &(0..session_status.total_players.max(1)).collect::<Vec<_>>(),
        local_handle.0,
        &mut player_handle_map,
        &mut observed_local_player,
        &balance,
        active_node.id,
        &active_node.label,
        active_node.kind.as_str(),
        active_node.encounter.reward_multiplier,
        active_node.encounter.ambient_heat_pressure,
        active_node.encounter.ambient_electrical_pressure,
        progression.hull_wear,
    );

    let spawn_points = [
        FixedVec2::from_num(-220.0, 120.0),
        FixedVec2::from_num(210.0, 40.0),
        FixedVec2::from_num(160.0, -150.0),
        FixedVec2::from_num(-120.0, -150.0),
    ];
    for (index, enemy_id) in active_node.encounter.enemy_ship_ids.iter().enumerate() {
        let Some(entry) = enemy_library_state.library.find_by_id(enemy_id) else {
            continue;
        };
        let spawn_position = spawn_points
            .get(index)
            .copied()
            .unwrap_or_else(|| FixedVec2::from_num(180.0 + index as f32 * 40.0, 90.0));
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
        );
    }
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    mut player_handle_map: ResMut<netcode::PlayerHandleMap>,
    mut observed_local_player: ResMut<netcode::ObservedLocalPlayer>,
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) {
    player_handle_map.entities.clear();
    observed_local_player.entity = None;
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
