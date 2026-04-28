mod arena;
mod hud;
mod salvage;

use bevy::prelude::*;

use super::{
    super::super::state::EditorShip,
    ship::spawn_runtime_ship,
};
use arena::spawn_test_arena;
use hud::spawn_runtime_hud;
use salvage::spawn_salvage_wreck;

pub(crate) fn spawn_runtime_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
) {
    spawn_runtime_hud(&mut commands, &asset_server, &editor_ship.ship);
    spawn_test_arena(&mut commands);
    spawn_salvage_wreck(&mut commands);
    spawn_runtime_ship(&mut commands, &asset_server, &editor_ship.ship);
}

pub(crate) fn cleanup_runtime_entities(
    mut commands: Commands,
    query: Query<Entity, With<super::super::super::state::PlayingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
