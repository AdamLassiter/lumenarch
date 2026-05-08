use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use super::{DockedPreviewRoot, DockedPreviewSignature, DockedPreviewTile};
use crate::{
    TILE_SIZE,
    TOOLBOX_WIDTH,
    netcode,
    ship::{ModuleKind, ModuleVariant, ShipDefinition, storage::load_default_ship},
    state::EditorShip,
};

pub(super) fn spawn_docked_ship_preview(
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
            Transform::from_xyz((TOOLBOX_WIDTH + 80.0) * 0.5, 0.0, 0.0),
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
                        asset_server
                            .load(docked_sprite_path_for_kind(&module.kind, module.variant)),
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

pub(super) fn docked_preview_ship(
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

pub(super) fn docked_preview_signature(ship: &ShipDefinition) -> u128 {
    let encoded = serde_json::to_vec(ship).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    encoded.hash(&mut hasher);
    hasher.finish() as u128
}
