mod arena;
mod hud;
mod salvage;

use arena::spawn_test_arena;
use bevy::prelude::*;
use hud::spawn_runtime_hud;
use salvage::spawn_salvage_wreck;

use super::ship::spawn_runtime_ship;
use crate::client::state::{DemoProgression, EditorShip, SectorState};

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
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
        active_node.encounter.hostile_count,
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
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
