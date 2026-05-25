use super::*;

pub(crate) fn cleanup_docked_spaceport_scene(
    mut commands: Commands,
    scene_query: Query<Entity, With<DockedSpaceportRoot>>,
    dialogue_query: Query<Entity, With<DockedDialogueRoot>>,
    runner_query: Query<Entity, With<DockedYarnRunner>>,
    scene_state: Option<Res<DockedSpaceportState>>,
    mut avatar_memory: ResMut<DockedAvatarMemory>,
    local_handle: Option<Res<LocalPlayerHandle>>,
) {
    if let Some(scene_state) = scene_state {
        avatar_memory.save_for(local_handle.as_deref(), scene_state.avatar);
    }
    for entity in &scene_query {
        commands.entity(entity).despawn();
    }
    for entity in &dialogue_query {
        commands.entity(entity).despawn();
    }
    for entity in &runner_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DockedSpaceportState>();
}

pub(crate) fn docked_spaceport_scene_missing(
    query: Query<Entity, With<DockedSpaceportRoot>>,
) -> bool {
    query.is_empty()
}

pub(crate) fn docked_spaceport_scene_present(
    query: Query<Entity, With<DockedSpaceportRoot>>,
) -> bool {
    !query.is_empty()
}

pub(super) fn docked_scene_state(
    station: &StationDefinition,
    ship: &ShipDefinition,
) -> DockedSpaceportState {
    let docked_ship = docked_player_ship_definition(ship);
    let airlock = docked_ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Airlock);
    let station_airlock = station
        .ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Airlock);
    let station_airlock_tile = station_airlock
        .map(|module| (module.grid_x, module.grid_y))
        .unwrap_or((-5, 0));
    let station_airlock_exit = station_airlock
        .map(|module| airlock_facing_offset(module.rotation_quadrants))
        .unwrap_or((-1, 0));
    let ship_offset_x = airlock
        .map(|module| station_airlock_tile.0 + station_airlock_exit.0 - module.grid_x)
        .unwrap_or(station_airlock_tile.0 - 3);
    let ship_offset_y = airlock
        .map(|module| station_airlock_tile.1 + station_airlock_exit.1 - module.grid_y)
        .unwrap_or(station_airlock_tile.1);
    let cockpit_tile = docked_ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Cockpit)
        .map(|module| (module.grid_x + ship_offset_x, module.grid_y + ship_offset_y));
    let ship_walkable_tiles = docked_ship
        .foundation_tiles
        .iter()
        .map(|tile| (tile.grid_x + ship_offset_x, tile.grid_y + ship_offset_y))
        .chain(
            docked_ship
                .modules
                .iter()
                .filter(|module| {
                    !matches!(
                        module.kind,
                        ModuleKind::Hull
                            | ModuleKind::HullInnerCorner
                            | ModuleKind::HullOuterCorner
                            | ModuleKind::InteriorWall
                    )
                })
                .map(|module| (module.grid_x + ship_offset_x, module.grid_y + ship_offset_y)),
        )
        .collect::<Vec<_>>();

    DockedSpaceportState {
        avatar: DockedAvatarState {
            grid_x: station_airlock_tile.0,
            grid_y: station_airlock_tile.1,
            facing_x: 1,
            facing_y: 0,
            local_position: grid_position(station_airlock_tile.0, station_airlock_tile.1),
            local_velocity: Vec2::ZERO,
            facing_radians: 0.0,
        },
        ship_offset_x,
        ship_offset_y,
        ship_walkable_tiles,
        cockpit_tile,
    }
}

pub(super) fn restore_docked_avatar_if_walkable(
    station: &StationDefinition,
    scene_state: &mut DockedSpaceportState,
    saved_avatar: Option<DockedAvatarState>,
) {
    let Some(mut avatar) = saved_avatar else {
        return;
    };
    let tile = docked_grid_from_local_position(avatar.local_position);
    if !is_walkable_docked_tile_without_avatar(station, tile, scene_state) {
        return;
    }
    avatar.grid_x = tile.0;
    avatar.grid_y = tile.1;
    avatar.local_velocity = Vec2::ZERO;
    scene_state.avatar = avatar;
}

pub(super) fn is_walkable_docked_tile(
    station: &StationDefinition,
    tile: (i32, i32),
    scene_state: &DockedSpaceportState,
) -> bool {
    if tile == (scene_state.avatar.grid_x, scene_state.avatar.grid_y) {
        return true;
    }
    if station_ship_walkable_tiles(&station.ship).contains(&tile) {
        return true;
    }
    scene_state.ship_walkable_tiles.contains(&tile)
}

pub(super) fn is_walkable_docked_tile_without_avatar(
    station: &StationDefinition,
    tile: (i32, i32),
    scene_state: &DockedSpaceportState,
) -> bool {
    station_ship_walkable_tiles(&station.ship).contains(&tile)
        || scene_state.ship_walkable_tiles.contains(&tile)
}

pub(super) fn docked_avatar_memory_handle(
    local_handle: Option<&LocalPlayerHandle>,
) -> PlayerHandle {
    local_handle.and_then(|handle| handle.0).unwrap_or(0)
}

pub(super) fn clear_docked_dialogue_state(dialogue_state: &mut DockedDialogueState) {
    dialogue_state.active_npc_id = None;
    dialogue_state.selected_surface_hint = None;
    dialogue_state.active_yarn_node = None;
    dialogue_state.runner_entity = None;
    dialogue_state.yarn_speaker = None;
    dialogue_state.yarn_line = None;
    dialogue_state.yarn_options.clear();
}

pub(super) fn opening_contact_line_matches(
    stations: &StationCatalogResource,
    sector_state: &SectorState,
    dialogue_state: &DockedDialogueState,
    line: &str,
) -> bool {
    let Some(npc_id) = dialogue_state.active_npc_id.as_deref() else {
        return false;
    };
    stations::current_station(&stations.0, sector_state)
        .and_then(|station| station.contacts.iter().find(|contact| contact.id == npc_id))
        .is_some_and(|contact| contact.brief == line)
}

pub(super) fn station_ship_walkable_tiles(ship: &ShipDefinition) -> Vec<(i32, i32)> {
    let blocked_module_tiles = ship
        .modules
        .iter()
        .filter(|module| station_module_blocks_docked_walk(module))
        .map(|module| (module.grid_x, module.grid_y))
        .collect::<Vec<_>>();
    ship.foundation_tiles
        .iter()
        .filter(|tile| !blocked_module_tiles.contains(&(tile.grid_x, tile.grid_y)))
        .map(|tile| (tile.grid_x, tile.grid_y))
        .chain(ship.modules.iter().filter_map(|module| {
            (!station_module_blocks_docked_walk(module)).then_some((module.grid_x, module.grid_y))
        }))
        .collect()
}

pub(super) fn station_module_blocks_docked_walk(module: &crate::ship::ShipModule) -> bool {
    matches!(
        module.kind,
        ModuleKind::Hull
            | ModuleKind::HullInnerCorner
            | ModuleKind::HullOuterCorner
            | ModuleKind::InteriorWall
            | ModuleKind::Engine
            | ModuleKind::Turret
    ) || station_editor::station_npc_service_action_for_module(module).is_some()
}

pub(super) fn focused_tile(avatar: DockedAvatarState) -> (i32, i32) {
    (
        avatar.grid_x + avatar.facing_x,
        avatar.grid_y + avatar.facing_y,
    )
}

pub(super) fn grid_translation(grid_x: i32, grid_y: i32, z: f32) -> Vec3 {
    Vec3::new(grid_x as f32 * TILE_SIZE, -(grid_y as f32) * TILE_SIZE, z)
}

pub(super) fn grid_position(grid_x: i32, grid_y: i32) -> Vec2 {
    Vec2::new(grid_x as f32 * TILE_SIZE, -(grid_y as f32) * TILE_SIZE)
}

pub(super) fn local_translation(local_position: Vec2, z: f32) -> Vec3 {
    Vec3::new(local_position.x, local_position.y, z)
}

pub(super) fn docked_grid_from_local_position(local_position: Vec2) -> (i32, i32) {
    let tile_half = TILE_SIZE * 0.5;
    (
        ((local_position.x + tile_half) / TILE_SIZE).floor() as i32,
        ((-local_position.y + tile_half) / TILE_SIZE).floor() as i32,
    )
}

pub(super) fn docked_facing_offset(facing_radians: f32) -> (i32, i32) {
    let facing = Vec2::from_angle(facing_radians);
    let threshold = 0.382_683_43;
    let x = if facing.x > threshold {
        1
    } else if facing.x < -threshold {
        -1
    } else {
        0
    };
    let y = if facing.y > threshold {
        -1
    } else if facing.y < -threshold {
        1
    } else {
        0
    };
    (x, y)
}

pub(super) fn airlock_facing_offset(rotation_quadrants: u8) -> (i32, i32) {
    match rotation_quadrants % 4 {
        0 => (0, -1),
        1 => (1, 0),
        2 => (0, 1),
        _ => (-1, 0),
    }
}

pub(super) fn docked_player_ship_definition(ship: &ShipDefinition) -> ShipDefinition {
    let mut rotated = ship.clone();
    for tile in &mut rotated.foundation_tiles {
        rotate_docked_tile(
            &mut tile.grid_x,
            &mut tile.grid_y,
            &mut tile.rotation_quadrants,
        );
    }
    for tile in &mut rotated.hull_tiles {
        rotate_docked_tile(
            &mut tile.grid_x,
            &mut tile.grid_y,
            &mut tile.rotation_quadrants,
        );
    }
    for module in &mut rotated.modules {
        rotate_docked_tile(
            &mut module.grid_x,
            &mut module.grid_y,
            &mut module.rotation_quadrants,
        );
    }
    rotated
}

pub(super) fn rotate_docked_tile(grid_x: &mut i32, grid_y: &mut i32, rotation_quadrants: &mut u8) {
    let original_x = *grid_x;
    let original_y = *grid_y;
    *grid_x = -original_y;
    *grid_y = original_x;
    *rotation_quadrants = (*rotation_quadrants + 1) % 4;
}

pub(super) fn docked_module_visual_z(kind: ModuleKind) -> f32 {
    if matches!(
        kind,
        ModuleKind::Airlock | ModuleKind::Engine | ModuleKind::Turret
    ) {
        0.75
    } else {
        1.0
    }
}

pub(super) fn docked_foundation_visual_z(kind: ShipFoundationKind) -> f32 {
    if matches!(
        kind,
        ShipFoundationKind::Hull
            | ShipFoundationKind::HullInnerCorner
            | ShipFoundationKind::HullOuterCorner
    ) {
        0.5
    } else {
        0.25
    }
}

pub(super) fn station_npcs(ship: &ShipDefinition) -> Vec<DockedStationNpc> {
    ship.modules
        .iter()
        .filter_map(|module| {
            let service_action = station_editor::station_npc_service_action_for_module(module)?;
            let (id, name, role, node) = station_npc_dialogue_defaults(service_action);
            Some(DockedStationNpc {
                id: id.to_string(),
                name: name.to_string(),
                role: role.to_string(),
                grid_x: module.grid_x,
                grid_y: module.grid_y,
                dialogue_start_node: node.to_string(),
                service_action,
            })
        })
        .collect()
}

pub(super) fn station_npc_dialogue_defaults(
    service_action: StationNpcServiceAction,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match service_action {
        StationNpcServiceAction::Engineer => (
            "ivra_quell",
            "Ivra Quell",
            "Harbormaster Engineer",
            "IvraQuell",
        ),
        StationNpcServiceAction::Contracts => {
            ("sable_ren", "Sable Ren", "Contract Broker", "SableRen")
        }
        StationNpcServiceAction::Archives => (
            "peregrine_cho",
            "Peregrine Cho",
            "Archive Custodian",
            "PeregrineCho",
        ),
    }
}
