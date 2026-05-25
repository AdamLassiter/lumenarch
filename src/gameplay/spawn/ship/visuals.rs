use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::{
    TILE_SIZE,
    helpers::sprite_path_for_foundation_connections,
    ship::{ModuleKind, ShipDefinition, ShipFoundationKind, ShipFoundationTile},
};

fn foundation_sprite_with_connections(
    ship: &ShipDefinition,
    tile: &ShipFoundationTile,
) -> (String, u8) {
    let north = ship
        .logistics_at(tile.grid_x, tile.grid_y - 1)
        .is_some_and(|other| other.kind == tile.kind);
    let east = ship
        .logistics_at(tile.grid_x + 1, tile.grid_y)
        .is_some_and(|other| other.kind == tile.kind);
    let south = ship
        .logistics_at(tile.grid_x, tile.grid_y + 1)
        .is_some_and(|other| other.kind == tile.kind);
    let west = ship
        .logistics_at(tile.grid_x - 1, tile.grid_y)
        .is_some_and(|other| other.kind == tile.kind);
    sprite_path_for_foundation_connections(tile.kind, north, east, south, west)
}

pub(crate) fn ship_visual_center(ship: &ShipDefinition) -> Option<(f32, f32)> {
    ship.bounds().map(|(min_x, max_x, min_y, max_y)| {
        ((min_x + max_x) as f32 * 0.5, (min_y + max_y) as f32 * 0.5)
    })
}

pub(crate) fn is_hull_foundation_kind(kind: ShipFoundationKind) -> bool {
    matches!(
        kind,
        ShipFoundationKind::Hull
            | ShipFoundationKind::HullInnerCorner
            | ShipFoundationKind::HullOuterCorner
    )
}

pub(crate) fn module_visual_z(kind: ModuleKind) -> f32 {
    if matches!(
        kind,
        ModuleKind::Airlock | ModuleKind::Engine | ModuleKind::Turret
    ) {
        0.75
    } else {
        1.0
    }
}

pub(crate) fn foundation_visual_z(kind: ShipFoundationKind) -> f32 {
    if is_hull_foundation_kind(kind) {
        0.5
    } else {
        0.25
    }
}

pub(crate) fn spawn_foundation_visual<B: Bundle>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    tile: &ShipFoundationTile,
    center_x: f32,
    center_y: f32,
    extra: B,
) -> Entity {
    let (path, connection_rotation) = foundation_sprite_with_connections(ship, tile);
    commands
        .spawn((
            Sprite::from_image(asset_server.load(path)),
            Transform {
                translation: Vec3::new(
                    (tile.grid_x as f32 - center_x) * TILE_SIZE,
                    -((tile.grid_y as f32) - center_y) * TILE_SIZE,
                    foundation_visual_z(tile.kind),
                ),
                rotation: Quat::from_rotation_z(
                    -((tile.rotation_quadrants + connection_rotation) as f32) * FRAC_PI_2,
                ),
                ..default()
            },
            extra,
        ))
        .id()
}
