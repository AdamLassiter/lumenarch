use super::*;
use crate::{
    ship::{ShipFoundationKind, ShipFoundationTile, ShipModule},
    stations::StationCatalog,
};

#[test]
fn docked_scene_places_ship_airlock_at_station_anchor() {
    let mut ship = ShipDefinition::empty("Airlock Test");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Airlock, 0, 0, 0));
    let station = StationDefinition {
        id: "test".to_string(),
        name: "Test".to_string(),
        faction: crate::stations::FactionId::ContinuantGuild,
        flavor: String::new(),
        ship: {
            let mut station_ship = ShipDefinition::empty("Station");
            station_ship.replace_module(ShipModule::new(1, ModuleKind::Airlock, -5, 2, 3));
            station_ship
        },
        contacts: Vec::new(),
        services: Vec::new(),
        contracts: Vec::new(),
        lore_entries: Vec::new(),
    };

    let scene = docked_scene_state(&station, &ship);

    assert_eq!(scene.ship_offset_x, -6);
    assert_eq!(scene.ship_offset_y, 2);
}

#[test]
fn bottom_station_airlock_aligns_to_rotated_player_airlock() {
    let mut ship = ShipDefinition::empty("Right Edge Airlock");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        2,
        0,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Airlock, 2, 0, 1));
    let station = StationDefinition {
        id: "test".to_string(),
        name: "Test".to_string(),
        faction: crate::stations::FactionId::ContinuantGuild,
        flavor: String::new(),
        ship: {
            let mut station_ship = ShipDefinition::empty("Station");
            station_ship.replace_module(ShipModule::new(1, ModuleKind::Airlock, 5, 5, 2));
            station_ship
        },
        contacts: Vec::new(),
        services: Vec::new(),
        contracts: Vec::new(),
        lore_entries: Vec::new(),
    };

    let scene = docked_scene_state(&station, &ship);

    assert_eq!(
        (scene.ship_offset_x, scene.ship_offset_y),
        (5, 4),
        "rotated ship airlock should sit on the station airlock docking side"
    );
    assert!(scene.ship_walkable_tiles.contains(&(5, 6)));
}

#[test]
fn saved_player_ship_docks_to_saved_needle_rest_without_overlap() {
    let ship: ShipDefinition =
        serde_json::from_str(include_str!("../../../saves/editor_ship.json"))
            .expect("saved editor ship should deserialize");
    let stations: StationCatalog =
        serde_json::from_str(include_str!("../../../saves/stations.json"))
            .expect("saved stations should deserialize");
    let station = stations
        .station("needle_rest")
        .expect("saved station catalog should contain Needle Rest");

    let scene = docked_scene_state(station, &ship);
    let docked_ship = docked_player_ship_definition(&ship);
    let station_airlock = station
        .ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Airlock)
        .expect("Needle Rest should have a docking airlock");
    let ship_airlock = ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Airlock)
        .expect("saved player ship should have a docking airlock");
    let docked_ship_airlock = docked_ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Airlock)
        .expect("rotated saved player ship should retain its docking airlock");

    assert_eq!(
        ship_airlock.rotation_quadrants, 1,
        "saved player ship fixture should keep its right-facing airlock"
    );
    assert_eq!(
        station_airlock.rotation_quadrants, 0,
        "saved Needle Rest fixture should keep its downward-facing airlock"
    );

    let placed_ship_airlock = (
        docked_ship_airlock.grid_x + scene.ship_offset_x,
        docked_ship_airlock.grid_y + scene.ship_offset_y,
    );
    let station_airlock_exit = airlock_facing_offset(station_airlock.rotation_quadrants);
    let expected_ship_airlock = (
        station_airlock.grid_x + station_airlock_exit.0,
        station_airlock.grid_y + station_airlock_exit.1,
    );
    let overlaps = docked_tile_overlaps(station, &docked_ship, &scene);
    let station_screen_rotation =
        effective_screen_rotation_quadrants(station_airlock.rotation_quadrants);
    let ship_screen_rotation =
        effective_screen_rotation_quadrants(docked_ship_airlock.rotation_quadrants);

    assert_eq!(
        ship_screen_rotation,
        (station_screen_rotation + 2) % 4,
        "saved ship airlock should render 180 degrees opposite the saved station airlock"
    );

    assert_eq!(
        placed_ship_airlock, expected_ship_airlock,
        "saved ship airlock should dock on the saved station airlock connector side"
    );
    assert!(
        overlaps.is_empty(),
        "saved docked scene should not overlap station and ship tiles, found {overlaps:?}"
    );
}

#[test]
fn focused_docked_tile_uses_facing_direction() {
    let avatar = DockedAvatarState {
        grid_x: 0,
        grid_y: 0,
        facing_x: 0,
        facing_y: -1,
        local_position: grid_position(0, 0),
        local_velocity: Vec2::ZERO,
        facing_radians: FRAC_PI_2,
    };

    assert_eq!(focused_tile(avatar), (0, -1));
}

#[test]
fn docked_scene_rotates_player_ship_counterclockwise() {
    let mut ship = ShipDefinition::empty("Rotation Test");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        2,
        1,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Cockpit, 2, 1, 3));

    let rotated = docked_player_ship_definition(&ship);

    assert_eq!(
        rotated.foundation_tiles[0].grid_x, -1,
        "x should become negative original y"
    );
    assert_eq!(
        rotated.foundation_tiles[0].grid_y, 2,
        "y should become original x"
    );
    assert_eq!(rotated.modules[0].rotation_quadrants, 0);
}

#[test]
fn station_npc_tiles_are_interacted_with_from_adjacent_focus() {
    let mut ship = ShipDefinition::empty("Station");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    ship.replace_module(ShipModule {
        variant: crate::ship::ModuleVariant::StationEngineerNpc,
        ..ShipModule::new(1, ModuleKind::Interior, 0, 0, 0)
    });

    assert!(!station_ship_walkable_tiles(&ship).contains(&(0, 0)));
}

#[test]
fn docked_station_airlock_tiles_are_walkable() {
    let mut ship = ShipDefinition::empty("Station Airlock");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Airlock, 0, 0, 0));

    assert!(station_ship_walkable_tiles(&ship).contains(&(0, 0)));
}

fn docked_tile_overlaps(
    station: &StationDefinition,
    docked_ship: &ShipDefinition,
    scene: &DockedSpaceportState,
) -> Vec<(i32, i32)> {
    let station_tiles = occupied_tiles(&station.ship, 0, 0);
    occupied_tiles(docked_ship, scene.ship_offset_x, scene.ship_offset_y)
        .into_iter()
        .filter(|tile| station_tiles.contains(tile))
        .collect()
}

fn occupied_tiles(ship: &ShipDefinition, offset_x: i32, offset_y: i32) -> Vec<(i32, i32)> {
    ship.foundation_tiles
        .iter()
        .map(|tile| (tile.grid_x + offset_x, tile.grid_y + offset_y))
        .chain(
            ship.hull_tiles
                .iter()
                .map(|tile| (tile.grid_x + offset_x, tile.grid_y + offset_y)),
        )
        .chain(
            ship.modules
                .iter()
                .map(|module| (module.grid_x + offset_x, module.grid_y + offset_y)),
        )
        .collect()
}

fn effective_screen_rotation_quadrants(rotation_quadrants: u8) -> u8 {
    (4 - rotation_quadrants % 4) % 4
}
