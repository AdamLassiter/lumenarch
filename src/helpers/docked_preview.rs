use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use crate::{
    TOOLBOX_WIDTH,
    docked::{DockedPreviewRoot, DockedPreviewSignature, DockedPreviewTile},
    editor::normalize_editor_ship_layers,
    gameplay::{ship_visual_center, spawn_ship_layer_visuals},
    netcode,
    ship::{ShipDefinition, storage::load_default_ship},
    state::EditorShip,
};

pub(crate) fn spawn_docked_ship_preview(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: ShipDefinition,
    ship_signature: u128,
) {
    if ship.modules.is_empty() && ship.foundation_tiles.is_empty() && ship.hull_tiles.is_empty() {
        return;
    }
    let (center_x, center_y) = ship_visual_center(&ship).unwrap_or((0.0, 0.0));

    let root_entity = commands
        .spawn((
            Transform::from_xyz((TOOLBOX_WIDTH + 80.0) * 0.5, 0.0, 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            DockedPreviewRoot,
            DockedPreviewSignature(ship_signature),
        ))
        .id();

    let children = spawn_ship_layer_visuals(
        commands,
        asset_server,
        &ship,
        center_x,
        center_y,
        DockedPreviewTile,
    );
    commands.entity(root_entity).add_children(&children);
}

pub(crate) fn docked_preview_ship(
    editor_ship: &EditorShip,
    status: &netcode::SessionStatus,
) -> ShipDefinition {
    let mut ship = if !editor_ship.ship.modules.is_empty() {
        editor_ship.ship.clone()
    } else if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
        snapshot.clone()
    } else {
        match load_default_ship() {
            Ok(Some(ship)) => ship,
            Ok(None) | Err(_) => ShipDefinition::empty("Untitled Knot"),
        }
    };
    normalize_editor_ship_layers(&mut ship);
    ship
}

pub(crate) fn docked_preview_signature(ship: &ShipDefinition) -> u128 {
    let encoded = serde_json::to_vec(ship).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    encoded.hash(&mut hasher);
    hasher.finish() as u128
}
