use bevy::prelude::*;

use super::{
    DockedPreviewRoot,
    DockedPreviewSignature,
    DockedRoot,
    preview_helpers::{docked_preview_signature, spawn_docked_ship_preview},
};
use crate::{TOOLBOX_WIDTH, state::EditorShip};

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

    let existing = existing_query.iter().collect::<Vec<_>>();
    let matching_count = existing
        .iter()
        .filter(|(_, signature)| signature.0 == ship_signature)
        .count();

    if matching_count == 1 && existing.len() == 1 {
        return;
    }

    for (entity, _) in existing {
        commands.entity(entity).despawn();
    }

    spawn_docked_ship_preview(&mut commands, &asset_server, ship, ship_signature);
}

pub(crate) fn rotate_docked_ship_preview(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DockedPreviewRoot>>,
) {
    let panel_width = TOOLBOX_WIDTH + 80.0;
    let desired_x = panel_width * 0.5;
    let desired_y = 0.0f32;
    for mut transform in &mut query {
        transform.translation.x = desired_x;
        transform.translation.y = desired_y;
        transform.rotate_z(0.12 * time.delta_secs());
    }
}

pub(crate) fn docked_ui_missing(query: Query<Entity, With<DockedRoot>>) -> bool {
    query.is_empty()
}

pub(crate) fn docked_ui_present(query: Query<Entity, With<DockedRoot>>) -> bool {
    !query.is_empty()
}
