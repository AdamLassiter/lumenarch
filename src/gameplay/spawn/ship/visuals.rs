use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::{
    TILE_SIZE,
    gameplay::helpers::sprite_path_for_kind,
    ship::{ModuleKind, ShipDefinition, ShipFoundationKind, ShipFoundationTile, ShipModule},
};

fn foundation_sprite_with_connections(
    ship: &ShipDefinition,
    tile: &ShipFoundationTile,
) -> (String, u8) {
    if !tile.kind.is_route() {
        return (format!("tiles/{}.png", tile.kind.as_str()), 0);
    }
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
    let count = [north, east, south, west]
        .into_iter()
        .filter(|connected| *connected)
        .count();
    let base = tile.kind.as_str();
    match count {
        4 => (format!("tiles/{base}_cross.png"), 0),
        3 => {
            let missing = if !north {
                2
            } else if !east {
                3
            } else if !south {
                0
            } else {
                1
            };
            (format!("tiles/{base}_tee.png"), missing)
        }
        2 if (north && south) || (east && west) => {
            let rotation = if east && west { 1 } else { 0 };
            (format!("tiles/{base}_straight.png"), rotation)
        }
        2 => {
            let rotation = match (north, east, south, west) {
                (true, true, false, false) => 0,
                (false, true, true, false) => 1,
                (false, false, true, true) => 2,
                (true, false, false, true) => 3,
                _ => 0,
            };
            (format!("tiles/{base}_corner.png"), rotation)
        }
        1 => {
            let rotation = if east {
                1
            } else if south {
                2
            } else if west {
                3
            } else {
                0
            };
            (format!("tiles/{base}_straight.png"), rotation)
        }
        _ => (format!("tiles/{base}_straight.png"), 0),
    }
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

pub(crate) fn spawn_ship_layer_visuals<B: Bundle + Clone>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    center_x: f32,
    center_y: f32,
    extra: B,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    for tile in &ship.foundation_tiles {
        entities.push(spawn_foundation_visual(
            commands,
            asset_server,
            ship,
            tile,
            center_x,
            center_y,
            extra.clone(),
        ));
    }
    for tile in &ship.hull_tiles {
        entities.push(spawn_foundation_visual(
            commands,
            asset_server,
            ship,
            tile,
            center_x,
            center_y,
            extra.clone(),
        ));
    }
    for module in ship.modules.iter().filter(|module| {
        !matches!(
            module.kind,
            ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
        )
    }) {
        entities.push(spawn_module_visual(
            commands,
            asset_server,
            module,
            center_x,
            center_y,
            extra.clone(),
        ));
    }
    entities
}

pub(crate) fn spawn_module_visual<B: Bundle>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    module: &ShipModule,
    center_x: f32,
    center_y: f32,
    extra: B,
) -> Entity {
    commands
        .spawn((
            Sprite::from_image(
                asset_server.load(sprite_path_for_kind(&module.kind, module.variant)),
            ),
            Transform {
                translation: Vec3::new(
                    (module.grid_x as f32 - center_x) * TILE_SIZE,
                    -((module.grid_y as f32) - center_y) * TILE_SIZE,
                    module_visual_z(module.kind),
                ),
                rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            extra,
        ))
        .id()
}
