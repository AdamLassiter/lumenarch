use super::*;

pub(crate) fn toggle_shipboard_control_mode(
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &mut ShipboardControlState,
            &CurrentStation,
        ),
        With<PlayerShipAssignment>,
    >,
    module_query: Query<(Entity, &RuntimeShipModule)>,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (handle, mut control_state, current_station) in players {
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        if !command.raw.pressed(netcode::INPUT_TOGGLE_STATION) {
            continue;
        }
        if control_state.mode == ShipControlMode::Interior {
            if current_station.kind != ModuleKind::Cockpit {
                continue;
            }
            if let Some((entity, runtime_module)) = module_query
                .iter()
                .find(|(_, runtime_module)| runtime_module.module_id == current_station.module_id)
            {
                super::focus_station(
                    &mut control_state,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Cockpit,
                    ShipControlMode::Cockpit,
                );
            }
        }
    }
}

pub(crate) fn exit_focused_station(
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    mut ship_query: Query<(&PlayerHandleComponent, &mut ShipboardControlState)>,
) {
    let mut players: Vec<_> = ship_query.iter_mut().collect();
    players.sort_by_key(|(handle, _)| handle.handle);
    for (handle, mut control_state) in players {
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        if !command.raw.pressed(netcode::INPUT_EXIT_STATION) {
            continue;
        }
        if control_state.focus_mode == StationFocusMode::Focused {
            control_state.mode = ShipControlMode::Interior;
            control_state.focus_mode = StationFocusMode::Internal;
            control_state.focused_entity = None;
            control_state.focused_module_id = None;
            control_state.focused_kind = None;
            control_state.focused_family = None;
        }
    }
}

pub(crate) fn update_player_reference_frame(
    ship_query: Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipInertiaField,
        ),
        With<ShipRoot>,
    >,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &mut PlayerMotionState,
            &mut InternalPosition,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (_, mut motion, mut internal_position) in players {
        let previous_frame = motion.frame;
        let mut best_ship = None;
        let mut best_distance_sq = None;

        for (ship_entity, ship_position, ship_rotation, ship_velocity, inertia_field) in &ship_query
        {
            let offset = motion.world_position - ship_position.value;
            let distance_sq = offset.length_sq();
            let radius_sq = helpers::fixed_square(inertia_field.radius);
            if distance_sq <= radius_sq
                && best_distance_sq.is_none_or(|current| distance_sq < current)
            {
                best_distance_sq = Some(distance_sq);
                let local_position = offset.rotate(-ship_rotation.radians);
                let relative_velocity = motion.world_velocity - ship_velocity.value;
                let local_velocity = relative_velocity.rotate(-ship_rotation.radians);
                best_ship = Some((ship_entity, local_position, local_velocity));
            }
        }

        match best_ship {
            Some((ship_entity, local_position, local_velocity)) => {
                motion.frame = PlayerReferenceFrame::Ship(ship_entity);
                if !matches!(previous_frame, PlayerReferenceFrame::Ship(current) if current == ship_entity)
                {
                    motion.local_position = local_position;
                    motion.local_velocity = local_velocity;
                }
                internal_position.local_position = local_position;
                internal_position.grid_x = (local_position.x / Fx::from_num(32)).to_num::<i32>();
                internal_position.grid_y = (-local_position.y / Fx::from_num(32)).to_num::<i32>();
            }
            None => {
                if let PlayerReferenceFrame::Ship(ship_entity) = previous_frame
                    && let Ok((_, _, ship_rotation, ship_velocity, _)) = ship_query.get(ship_entity)
                {
                    motion.world_velocity =
                        ship_velocity.value + motion.local_velocity.rotate(ship_rotation.radians);
                }
                motion.frame = PlayerReferenceFrame::World;
                motion.local_velocity = FixedVec2::zero();
            }
        }
    }
}

pub(crate) fn move_shipboard_player(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    ship_query: Query<
        (
            Entity,
            &SimPosition,
            &SimRotation,
            &LinearVelocity,
            &ShipAtmosphereState,
        ),
        With<ShipRoot>,
    >,
    module_query: Query<
        (
            &ChildOf,
            &RuntimeShipModule,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &ShipboardControlState,
            &mut PlayerMotionState,
            &mut InternalPosition,
            &PlayerFieldState,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let dt = fx_from_time_delta(&time);
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _, _, _)| handle.handle);
    for (handle, control_state, mut motion, mut position, player_fields) in players {
        if control_state.mode != ShipControlMode::Interior {
            helpers::anchor_player_to_focused_station(
                &mut motion,
                &mut position,
                control_state,
                &ship_query,
                &module_query,
            );
            continue;
        }

        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        let input =
            FixedVec2::from_num(command.move_x as i32, command.move_y as i32).normalized_or_zero();

        match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => {
                let Ok((_, ship_position, ship_rotation, ship_velocity, atmosphere_state)) =
                    ship_query.get(ship_entity)
                else {
                    motion.frame = PlayerReferenceFrame::World;
                    continue;
                };
                let movement_multiplier = if player_fields.oxygen_critical {
                    Fx::from_num(balance.atmosphere.critical_oxygen_speed_multiplier)
                } else if player_fields.oxygen_warning {
                    Fx::from_num(balance.atmosphere.low_oxygen_speed_multiplier)
                } else {
                    Fx::from_num(1)
                };
                motion.local_velocity +=
                    input * Fx::from_num(PLAYER_WALK_ACCELERATION) * movement_multiplier * dt;
                motion.local_velocity =
                    damp_vec2(motion.local_velocity, Fx::from_num(PLAYER_WALK_DAMPING), dt)
                        .clamp_length(Fx::from_num(PLAYER_WALK_MAX_SPEED) * movement_multiplier);
                let local_velocity = motion.local_velocity;
                let desired_local_position = motion.local_position + local_velocity * dt;
                let collision_tiles =
                    helpers::ship_collision_tiles(ship_entity, atmosphere_state, &module_query);
                motion.local_position = helpers::resolve_ship_local_motion(
                    motion.local_position,
                    desired_local_position,
                    &collision_tiles,
                );
                motion.world_position =
                    ship_position.value + motion.local_position.rotate(ship_rotation.radians);
                motion.world_velocity =
                    ship_velocity.value + local_velocity.rotate(ship_rotation.radians);
                position.local_position = motion.local_position;
                position.grid_x = (motion.local_position.x / Fx::from_num(32)).to_num::<i32>();
                position.grid_y = (-motion.local_position.y / Fx::from_num(32)).to_num::<i32>();
            }
            PlayerReferenceFrame::World => {
                motion.world_velocity += input * Fx::from_num(PLAYER_EVA_ACCELERATION) * dt;
                motion.world_velocity =
                    damp_vec2(motion.world_velocity, Fx::from_num(PLAYER_EVA_DAMPING), dt)
                        .clamp_length(Fx::from_num(PLAYER_EVA_MAX_SPEED));
                let world_velocity = motion.world_velocity;
                motion.world_position += world_velocity * dt;
                clamp_position_to_arena(&mut motion.world_position);
            }
        }
    }
}

pub(crate) fn sync_shipboard_player_visual(
    ship_query: Single<&ShipboardControlState, With<ObservedLocalPlayerMarker>>,
    mut player_query: Query<
        (
            Option<&ObservedLocalPlayerMarker>,
            &PlayerMotionState,
            &CarriedResource,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
        ),
        (With<ShipboardMarker>, With<ShipboardPlayer>),
    >,
) {
    let control_state = ship_query.into_inner();
    for (observed, position, carried, mut transform, mut sprite, mut visibility) in
        &mut player_query
    {
        transform.translation = match position.frame {
            PlayerReferenceFrame::Ship(_) => Vec3::new(
                position.local_position.x.to_num::<f32>(),
                position.local_position.y.to_num::<f32>(),
                6.0,
            ),
            PlayerReferenceFrame::World => render_translation(position.world_position, 6.0),
        };
        if observed.is_some() {
            sprite.color = match control_state.mode {
                ShipControlMode::Interior if carried.kind.is_some() => {
                    Color::srgb(0.98, 0.88, 0.52)
                }
                ShipControlMode::Interior => Color::srgb(0.82, 0.96, 0.62),
                ShipControlMode::Cockpit | ShipControlMode::Turret => {
                    Color::srgba(0.82, 0.96, 0.62, 0.20)
                }
                ShipControlMode::Reactor
                | ShipControlMode::Logistics
                | ShipControlMode::Computer => Color::srgb(0.92, 0.96, 0.68),
            };
            *visibility = if matches!(
                control_state.mode,
                ShipControlMode::Cockpit | ShipControlMode::Turret
            ) {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

pub(crate) fn sync_player_reference_frame_parenting(
    mut commands: Commands,
    player_query: Single<
        (Entity, &PlayerMotionState, Option<&ChildOf>),
        With<ObservedLocalPlayerMarker>,
    >,
) {
    let (player_entity, motion, parent) = player_query.into_inner();
    match motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) => {
            if parent.map(ChildOf::get) != Some(ship_entity) {
                commands
                    .entity(player_entity)
                    .set_parent_in_place(ship_entity);
            }
        }
        PlayerReferenceFrame::World => {
            if parent.is_some() {
                commands.entity(player_entity).remove_parent_in_place();
            }
        }
    }
}

pub(crate) fn update_current_station(
    ship_query: Query<(Entity, &SimPosition, &SimRotation), With<ShipRoot>>,
    module_query: Query<(&RuntimeShipModule, &ChildOf)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &PlayerMotionState,
            &mut CurrentStation,
        ),
        With<PlayerShipAssignment>,
    >,
) {
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (_, motion, mut station) in players {
        let Some(active_ship) = (match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
            PlayerReferenceFrame::World => None,
        }) else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
            continue;
        };

        let Ok((_, ship_position, ship_rotation)) = ship_query.get(active_ship) else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
            continue;
        };

        let mut nearest = None;
        let mut nearest_distance_sq = None;
        let max_distance_sq = helpers::fixed_square(Fx::from_num(PLAYER_INTERACT_RADIUS));
        for (runtime_module, parent) in &module_query {
            if parent.get() != active_ship {
                continue;
            }
            let world_position =
                ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
            let distance_sq = motion.world_position.distance_sq(world_position);
            if distance_sq > max_distance_sq {
                continue;
            }
            if nearest_distance_sq.is_none_or(|current| distance_sq < current) {
                nearest_distance_sq = Some(distance_sq);
                nearest = Some((runtime_module.module_id, runtime_module.kind));
            }
        }

        if let Some((module_id, kind)) = nearest {
            station.module_id = module_id;
            station.kind = kind;
        } else {
            station.module_id = 0;
            station.kind = ModuleKind::Interior;
        }
    }
}

pub(crate) fn handle_player_cargo_interaction(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    session_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    local_handle: Option<Res<netcode::LocalPlayerHandle>>,
    player_ship_query: Single<(Entity, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<
        (&PlayerMotionState, &CurrentStation, &mut CarriedResource),
        (With<ObservedLocalPlayerMarker>, With<PlayerShipAssignment>),
    >,
    mut loose_cargo_query: Query<(Entity, &SimPosition, &LooseCargo)>,
    mut storage_query: Query<(&RuntimeShipModule, &ChildOf, &mut StorageModule)>,
) {
    let session_input = netcode::local_player_input(session_inputs, local_handle);
    let pickup_pressed =
        keys.just_pressed(KeyCode::KeyF) || session_input.pressed(netcode::INPUT_PICKUP);
    let drop_pressed =
        keys.just_pressed(KeyCode::KeyG) || session_input.pressed(netcode::INPUT_DROP);
    if !pickup_pressed && !drop_pressed {
        return;
    }

    let (player_ship_entity, mut mission_state) = player_ship_query.into_inner();
    let (motion, current_station, mut carried) = player_query.into_inner();

    if drop_pressed {
        let Some(kind) = carried.kind else {
            return;
        };
        let amount = carried.amount.max(1);
        commands.spawn((
            Sprite::from_color(helpers::cargo_color(kind), Vec2::splat(10.0)),
            Transform::from_translation(render_translation(motion.world_position, 5.2)),
            SimPosition {
                value: motion.world_position,
            },
            LooseCargo { kind, amount },
            PlayingCleanup,
        ));
        mission_state.recent_action =
            Some(format!("Dropped {} {}", amount, helpers::resource_label(kind)));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }

    if carried.kind.is_none() {
        for (entity, position, cargo) in &mut loose_cargo_query {
            if motion.world_position.distance_sq(position.value)
                <= helpers::fixed_square(Fx::from_num(PLAYER_CARGO_PICKUP_RADIUS))
            {
                carried.kind = Some(cargo.kind);
                carried.amount = cargo.amount;
                mission_state.recent_action = Some(format!(
                    "Picked up {} {}",
                    cargo.amount,
                    helpers::resource_label(cargo.kind)
                ));
                mission_state.recent_action_timer = Fx::from_num(1.5);
                commands.entity(entity).despawn();
                return;
            }
        }

        let Some(active_ship) = (match motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) if ship_entity != player_ship_entity => {
                Some(ship_entity)
            }
            _ => None,
        }) else {
            return;
        };

        for (runtime_module, parent, mut storage) in &mut storage_query {
            if parent.get() != active_ship || runtime_module.module_id != current_station.module_id
            {
                continue;
            }
            if let Some((kind, amount)) = helpers::take_first_available(&mut storage.inventory) {
                carried.kind = Some(kind);
                carried.amount = amount;
                mission_state.recent_action = Some(format!(
                    "Extracted {} {} from hostile ship",
                    amount,
                    helpers::resource_label(kind)
                ));
                mission_state.recent_action_timer = Fx::from_num(1.5);
                return;
            }
        }
        return;
    }

    let Some(kind) = carried.kind else {
        return;
    };
    let Some(active_ship) = (match motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) if ship_entity == player_ship_entity => {
            Some(ship_entity)
        }
        _ => None,
    }) else {
        return;
    };

    for (runtime_module, parent, mut storage) in &mut storage_query {
        if parent.get() != active_ship || runtime_module.module_id != current_station.module_id {
            continue;
        }
        if storage.inventory.total_units() + carried.amount > storage.capacity {
            continue;
        }
        storage.inventory.add(kind, carried.amount);
        if kind == ResourceKind::RawSalvage {
            mission_state.recovered_raw_salvage += carried.amount;
            mission_state.salvage_scrap_awarded += carried.amount;
            mission_state.salvage_collected = true;
            mission_state.completion_reason = Some("Cargo recovered aboard".to_string());
        } else {
            mission_state.processed_repair_charge += carried.amount;
        }
        mission_state.recent_action = Some(format!(
            "Deposited {} {} aboard ship",
            carried.amount,
            helpers::resource_label(kind)
        ));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }
}
