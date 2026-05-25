use super::*;

pub(crate) fn spawn_docked_spaceport_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    editor_ship: Res<EditorShip>,
    avatar_memory: Res<DockedAvatarMemory>,
    local_profile: Res<LocalPlayerProfile>,
    local_handle: Option<Res<LocalPlayerHandle>>,
) {
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    let mut scene_state = docked_scene_state(station, &editor_ship.ship);
    restore_docked_avatar_if_walkable(
        station,
        &mut scene_state,
        avatar_memory.restore_for(local_handle.as_deref()),
    );
    let docked_player_ship = docked_player_ship_definition(&editor_ship.ship);
    commands.insert_resource(scene_state.clone());

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            DockedSpaceportRoot,
            Name::new("Docked Spaceport Scene"),
        ))
        .with_children(|root| {
            spawn_docked_ship_tiles(root, &asset_server, &station.ship, 0, 0, 0.0);

            spawn_docked_ship_tiles(
                root,
                &asset_server,
                &docked_player_ship,
                scene_state.ship_offset_x,
                scene_state.ship_offset_y,
                1.5,
            );

            for npc in station_npcs(&station.ship) {
                let sprite_path = npc.service_action.sprite_path();
                root.spawn((
                    Sprite::from_image(asset_server.load(sprite_path)),
                    Transform::from_translation(grid_translation(npc.grid_x, npc.grid_y, 2.0)),
                    GlobalTransform::default(),
                    DockedNpcMarker,
                ))
                .with_child((
                    Text2d::new(npc.name),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.94, 0.91, 0.78)),
                    Transform::from_translation(Vec3::new(0.0, 18.0, 1.0)),
                ));
            }

            root.spawn((
                docked_player_actor_sprite(&asset_server, &local_profile),
                Transform::from_translation(local_translation(
                    scene_state.avatar.local_position,
                    5.0,
                )),
                DockedLocalAvatar,
            ));

            let focus = focused_tile(scene_state.avatar);
            root.spawn((
                Sprite::from_color(Color::srgba(0.95, 0.95, 0.70, 0.28), Vec2::splat(TILE_SIZE)),
                Transform::from_translation(grid_translation(focus.0, focus.1, 4.0)),
                DockedFocusMarker,
            ));
        });
}

pub(super) fn spawn_docked_ship_tiles(
    root: &mut ChildSpawnerCommands<'_>,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    offset_x: i32,
    offset_y: i32,
    z_bias: f32,
) {
    for tile in &ship.foundation_tiles {
        spawn_docked_foundation_tile(root, asset_server, ship, tile, offset_x, offset_y, z_bias);
    }
    for tile in &ship.hull_tiles {
        spawn_docked_foundation_tile(root, asset_server, ship, tile, offset_x, offset_y, z_bias);
    }
    for module in &ship.modules {
        if matches!(
            module.kind,
            ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
        ) {
            continue;
        }
        root.spawn((
            Sprite::from_image(
                asset_server.load(sprite_path_for_kind(&module.kind, module.variant)),
            ),
            Transform {
                translation: grid_translation(
                    module.grid_x + offset_x,
                    module.grid_y + offset_y,
                    z_bias + docked_module_visual_z(module.kind),
                ),
                rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            DockedShipTileMarker,
        ));
    }
}

pub(super) fn docked_player_actor_sprite(
    asset_server: &AssetServer,
    profile: &LocalPlayerProfile,
) -> Sprite {
    let sprite_path = match profile.starting_suit() {
        crate::gameplay::components::PlayerSuit::Standard => "actors/player_default.png",
        crate::gameplay::components::PlayerSuit::Radiation => "actors/player_radiation.png",
        crate::gameplay::components::PlayerSuit::Welder => "actors/player_welder.png",
        crate::gameplay::components::PlayerSuit::Eva => "actors/player_eva.png",
    };
    if Path::new("assets").join(sprite_path).exists() {
        let mut sprite = Sprite::from_image(asset_server.load(sprite_path));
        sprite.color = profile.color();
        sprite.custom_size = Some(Vec2::splat(32.0));
        sprite
    } else {
        Sprite::from_color(profile.color(), Vec2::splat(24.0))
    }
}

pub(super) fn spawn_docked_foundation_tile(
    root: &mut ChildSpawnerCommands<'_>,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    tile: &ShipFoundationTile,
    offset_x: i32,
    offset_y: i32,
    z_bias: f32,
) {
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
    let (path, connection_rotation) =
        sprite_path_for_foundation_connections(tile.kind, north, east, south, west);
    root.spawn((
        Sprite::from_image(asset_server.load(path)),
        Transform {
            translation: grid_translation(
                tile.grid_x + offset_x,
                tile.grid_y + offset_y,
                z_bias + docked_foundation_visual_z(tile.kind),
            ),
            rotation: Quat::from_rotation_z(
                -((tile.rotation_quadrants + connection_rotation) as f32) * FRAC_PI_2,
            ),
            ..default()
        },
        DockedShipTileMarker,
    ));
}

/// Moves the local docked avatar on the station grid and keeps the focus marker in front of them.
pub(crate) fn move_docked_local_avatar(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    scene_state: Option<ResMut<DockedSpaceportState>>,
    mut avatar_memory: ResMut<DockedAvatarMemory>,
    local_handle: Option<Res<LocalPlayerHandle>>,
    mut avatar_query: Query<&mut Transform, (With<DockedLocalAvatar>, Without<DockedFocusMarker>)>,
    mut focus_query: Query<&mut Transform, (With<DockedFocusMarker>, Without<DockedLocalAvatar>)>,
) {
    let Some(mut scene_state) = scene_state else {
        return;
    };
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    let input = Vec2::new(
        i8::from(keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as f32
            - i8::from(keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as f32,
        i8::from(keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) as f32
            - i8::from(keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) as f32,
    )
    .normalize_or_zero();

    let dt = time.delta_secs();
    scene_state.avatar.local_velocity += input * balance.player.walk_acceleration * dt;
    scene_state.avatar.local_velocity = damp_vec2(
        FixedVec2::from_vec2(scene_state.avatar.local_velocity),
        Fx::from_num(balance.player.walk_damping),
        Fx::from_num(dt),
    )
    .to_vec2()
    .clamp_length_max(balance.player.walk_max_speed);

    if !scene_state.avatar.local_velocity.is_finite() {
        scene_state.avatar.local_velocity = Vec2::ZERO;
    }

    let desired = scene_state.avatar.local_position + scene_state.avatar.local_velocity * dt;
    let current = scene_state.avatar.local_position;
    let mut resolved = current;
    let try_x = Vec2::new(desired.x, resolved.y);
    if is_walkable_docked_tile(
        station,
        docked_grid_from_local_position(try_x),
        &scene_state,
    ) {
        resolved.x = desired.x;
    } else {
        scene_state.avatar.local_velocity.x = 0.0;
    }
    let try_y = Vec2::new(resolved.x, desired.y);
    if is_walkable_docked_tile(
        station,
        docked_grid_from_local_position(try_y),
        &scene_state,
    ) {
        resolved.y = desired.y;
    } else {
        scene_state.avatar.local_velocity.y = 0.0;
    }
    scene_state.avatar.local_position = resolved;

    if input.length_squared() > 0.0001 {
        scene_state.avatar.facing_radians = angle_from_vector(FixedVec2::from_vec2(input)).to_num();
        (scene_state.avatar.facing_x, scene_state.avatar.facing_y) =
            docked_facing_offset(scene_state.avatar.facing_radians);
    }
    let grid = docked_grid_from_local_position(scene_state.avatar.local_position);
    scene_state.avatar.grid_x = grid.0;
    scene_state.avatar.grid_y = grid.1;

    if let Ok(mut transform) = avatar_query.single_mut() {
        transform.translation = local_translation(scene_state.avatar.local_position, 5.0);
        transform.rotation = Quat::from_rotation_z(scene_state.avatar.facing_radians - FRAC_PI_2);
    }
    if let Ok(mut transform) = focus_query.single_mut() {
        let focus = focused_tile(scene_state.avatar);
        transform.translation = grid_translation(focus.0, focus.1, 4.0);
    }
    avatar_memory.save_for(local_handle.as_deref(), scene_state.avatar);
}
