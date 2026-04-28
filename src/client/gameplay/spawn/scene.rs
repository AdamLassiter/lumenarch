mod arena;
mod hud;
mod salvage;

use arena::spawn_test_arena;
use bevy::prelude::*;
use hud::spawn_runtime_hud;
use salvage::spawn_salvage_wreck;

use super::ship::{spawn_hostile_ship, spawn_runtime_ship};
use crate::client::{
    gameplay::helpers::{FixedVec2, Fx},
    state::{DemoProgression, EditorShip, EnemyShipLibraryState, SectorState},
};

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    enemy_library_state: Res<EnemyShipLibraryState>,
    sector_state: Res<SectorState>,
) {
    spawn_runtime_hud(&mut commands, &asset_server, &editor_ship.ship);
    let active_node = sector_state
        .active_node()
        .or_else(|| sector_state.selected_node())
        .cloned()
        .unwrap_or_else(|| sector_state.nodes[1].clone());
    spawn_test_arena(
        &mut commands,
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
            "brawler" => Fx::from_num(120.0),
            "skirmisher" => Fx::from_num(220.0),
            _ => Fx::from_num(180.0),
        };
        spawn_hostile_ship(
            &mut commands,
            &asset_server,
            &entry.ship,
            spawn_position,
            preferred_range,
            Fx::from_num(0.85),
            4 + u32::from(entry.threat_tier) * 3,
        );
    }
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
