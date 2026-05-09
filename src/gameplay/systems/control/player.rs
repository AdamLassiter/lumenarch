use super::*;
use crate::{
    gameplay::{components::PlayerConditionState, helpers::sampled_decompression_pull},
    ship::ModuleVariant,
};

pub(crate) fn toggle_shipboard_control_mode(
    decoded_commands: Res<netcode::DecodedPlayerCommands>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &mut ShipboardControlState,
            &PlayerConditionState,
            &CurrentStation,
            &PlayerMotionState,
        ),
        With<PlayerShipAssignment>,
    >,
    module_query: Query<(Entity, &RuntimeShipModule, &ChildOf)>,
) {
    let mission_state = mission_query.into_inner();
    if mission_state.failed || mission_state.completed {
        return;
    }

    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _, _, _)| handle.handle);
    for (handle, mut control_state, condition, current_station, motion) in players {
        if condition.control_disabled() {
            continue;
        }
        let command = netcode::command_for_handle(&decoded_commands, handle.handle);
        if !command.raw.pressed(netcode::INPUT_TOGGLE_STATION) {
            continue;
        }
        if control_state.mode == ShipControlMode::Interior {
            if current_station.kind != ModuleKind::Cockpit {
                continue;
            }
            let Some(active_ship) = (match motion.frame {
                PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
                PlayerReferenceFrame::World => None,
            }) else {
                continue;
            };
            if let Some((entity, runtime_module, _)) =
                module_query.iter().find(|(_, runtime_module, parent)| {
                    parent.get() == active_ship
                        && runtime_module.module_id == current_station.module_id
                        && runtime_module.kind == ModuleKind::Cockpit
                })
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
    mut ship_query: Query<(
        &PlayerHandleComponent,
        &mut ShipboardControlState,
        &PlayerConditionState,
    )>,
) {
    let mut players: Vec<_> = ship_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _)| handle.handle);
    for (handle, mut control_state, condition) in players {
        if condition.control_disabled() {
            continue;
        }
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
            &EquippedSuit,
            &PlayerFieldState,
            &PlayerConditionState,
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
    players.sort_by_key(|(handle, _, _, _, _, _, _)| handle.handle);
    for (
        handle,
        control_state,
        mut motion,
        mut position,
        equipped_suit,
        player_fields,
        condition,
    ) in players
    {
        if condition.control_disabled() {
            motion.local_velocity = FixedVec2::zero();
            motion.world_velocity = FixedVec2::zero();
            continue;
        }
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
                let decompression_pull =
                    sampled_decompression_pull(motion.local_position, atmosphere_state);
                motion.local_velocity += input
                    * Fx::from_num(balance.player.walk_acceleration)
                    * movement_multiplier
                    * dt;
                motion.local_velocity += decompression_pull * dt;
                motion.local_velocity = damp_vec2(
                    motion.local_velocity,
                    Fx::from_num(balance.player.walk_damping),
                    dt,
                )
                .clamp_length(Fx::from_num(balance.player.walk_max_speed) * movement_multiplier);
                let local_velocity = motion.local_velocity;
                let desired_local_position = motion.local_position + local_velocity * dt;
                let collision_tiles =
                    helpers::ship_collision_tiles(ship_entity, atmosphere_state, &module_query);
                motion.local_position = helpers::resolve_ship_local_motion(
                    motion.local_position,
                    desired_local_position,
                    &collision_tiles,
                    Fx::from_num(balance.player.collision_radius),
                );
                motion.world_position =
                    ship_position.value + motion.local_position.rotate(ship_rotation.radians);
                motion.world_velocity =
                    ship_velocity.value + local_velocity.rotate(ship_rotation.radians);
                if !local_velocity.is_near_zero() {
                    motion.facing_radians = angle_from_vector(local_velocity);
                }
                position.local_position = motion.local_position;
                position.grid_x = (motion.local_position.x / Fx::from_num(32)).to_num::<i32>();
                position.grid_y = (-motion.local_position.y / Fx::from_num(32)).to_num::<i32>();
            }
            PlayerReferenceFrame::World => {
                let eva_multiplier = equipped_suit.suit.eva_speed_multiplier(&balance.player);
                motion.world_velocity +=
                    input * Fx::from_num(balance.player.eva_acceleration) * eva_multiplier * dt;
                motion.world_velocity = damp_vec2(
                    motion.world_velocity,
                    Fx::from_num(balance.player.eva_damping),
                    dt,
                )
                .clamp_length(Fx::from_num(balance.player.eva_max_speed) * eva_multiplier);
                let world_velocity = motion.world_velocity;
                if !world_velocity.is_near_zero() {
                    motion.facing_radians = angle_from_vector(world_velocity);
                }
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
            &EquippedSuit,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
        ),
        (With<ShipboardMarker>, With<ShipboardPlayer>),
    >,
) {
    let control_state = ship_query.into_inner();
    for (observed, position, carried, equipped_suit, mut transform, mut sprite, mut visibility) in
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
        transform.rotation = Quat::from_rotation_z(
            position.facing_radians.to_num::<f32>() - std::f32::consts::FRAC_PI_2,
        );
        if observed.is_some() {
            sprite.color = match control_state.mode {
                ShipControlMode::Interior if carried.kind.is_some() => {
                    Color::srgb(0.98, 0.88, 0.52)
                }
                ShipControlMode::Interior => equipped_suit.suit.color(),
                ShipControlMode::Cockpit | ShipControlMode::Turret => {
                    Color::srgba(0.82, 0.96, 0.62, 0.20)
                }
                ShipControlMode::Reactor
                | ShipControlMode::Logistics
                | ShipControlMode::Computer => equipped_suit.suit.color(),
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

pub(crate) fn sync_crew_name_labels(
    camera_query: Single<&GlobalTransform, (With<Camera2d>, With<MainCamera>)>,
    player_query: Query<
        (&PlayerMotionState, &Visibility),
        (
            With<ShipboardPlayer>,
            Without<CrewNameLabel>,
            Without<CrewNameBackdrop>,
        ),
    >,
    mut label_query: Query<
        (&CrewNameLabel, &mut Transform, &mut Visibility),
        (Without<ShipboardPlayer>, Without<CrewNameBackdrop>),
    >,
    mut backdrop_query: Query<
        (&CrewNameBackdrop, &mut Transform, &mut Visibility),
        (Without<ShipboardPlayer>, Without<CrewNameLabel>),
    >,
) {
    let camera_transform = camera_query.into_inner();
    let label_rotation = camera_transform.compute_transform().rotation;
    for (label, mut transform, mut visibility) in &mut label_query {
        let Ok((motion, player_visibility)) = player_query.get(label.player_entity) else {
            continue;
        };
        transform.translation =
            render_translation(motion.world_position, 20.0) + Vec3::new(0.0, 15.0, 0.0);
        transform.rotation = label_rotation;
        *visibility = *player_visibility;
    }
    for (backdrop, mut transform, mut visibility) in &mut backdrop_query {
        let Ok((motion, player_visibility)) = player_query.get(backdrop.player_entity) else {
            continue;
        };
        transform.translation =
            render_translation(motion.world_position, 19.5) + Vec3::new(0.0, 15.0, 0.0);
        transform.rotation = label_rotation;
        *visibility = *player_visibility;
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
    balance: Res<BalanceConfig>,
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
        let max_distance_sq = helpers::fixed_square(Fx::from_num(balance.player.interact_radius));
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
    balance: Res<BalanceConfig>,
    player_ship_query: Single<(Entity, &mut MissionState), (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<
        (
            &PlayerMotionState,
            &CurrentStation,
            &mut CarriedResource,
            &mut EquippedSuit,
            &PlayerConditionState,
        ),
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
    let (motion, current_station, mut carried, mut equipped_suit, condition) =
        player_query.into_inner();
    if condition.control_disabled() {
        return;
    }

    if drop_pressed {
        let Some(kind) = carried.kind else {
            return;
        };
        let amount = carried.amount.max(1);
        commands.spawn((
            Sprite::from_color(kind.color(), Vec2::splat(16.0)),
            Transform::from_translation(render_translation(motion.world_position, 5.2)),
            SimPosition {
                value: motion.world_position,
            },
            LooseCargo { kind, amount },
            PlayingCleanup,
        ));
        mission_state.recent_action = Some(format!("Dropped {} {}", amount, kind.label()));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }

    if carried.kind.is_none() {
        for (entity, position, cargo) in &mut loose_cargo_query {
            if motion.world_position.distance_sq(position.value)
                <= helpers::fixed_square(Fx::from_num(balance.player.cargo_pickup_radius))
            {
                carried.kind = Some(cargo.kind);
                carried.amount = cargo.amount;
                mission_state.recent_action =
                    Some(format!("Picked up {} {}", cargo.amount, cargo.kind.label()));
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
                carried.kind = Some(CarriedItemKind::Resource(kind));
                carried.amount = amount;
                mission_state.recent_action = Some(format!(
                    "Extracted {} {} from hostile ship",
                    amount,
                    CarriedItemKind::Resource(kind).label()
                ));
                mission_state.recent_action_timer = Fx::from_num(1.5);
                return;
            }
        }
        return;
    }

    if let Some(CarriedItemKind::Suit(new_suit)) = carried.kind {
        let old_suit = equipped_suit.suit;
        equipped_suit.suit = new_suit;
        if old_suit == PlayerSuit::Standard {
            carried.kind = None;
            carried.amount = 0;
        } else {
            carried.kind = Some(CarriedItemKind::Suit(old_suit));
            carried.amount = 1;
        }
        mission_state.recent_action = Some(format!("Equipped {}", new_suit.as_str()));
        mission_state.recent_action_timer = Fx::from_num(1.5);
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
        let Some(deposit) = carried_item_deposit(kind, carried.amount) else {
            mission_state.recent_action =
                Some("This item cannot be stowed in ship storage".to_string());
            mission_state.recent_action_timer = Fx::from_num(1.5);
            return;
        };
        if storage.inventory.total_units() + deposit.resource_amount > storage.capacity {
            continue;
        }
        if let Some((component_kind, component_variant, component_amount)) = deposit.component {
            storage.add_damaged_component(component_kind, component_variant, component_amount);
        }
        if let Some(resource_kind) = deposit.resource_kind {
            storage
                .inventory
                .add(resource_kind, deposit.resource_amount);
        }
        if deposit.resource_kind == Some(ResourceKind::RawSalvage) {
            mission_state.recovered_raw_salvage += deposit.resource_amount;
            mission_state.salvage_scrap_awarded += deposit.resource_amount;
            mission_state.salvage_collected = true;
            mission_state.completion_reason = Some("Cargo recovered aboard".to_string());
        } else if deposit.resource_kind.is_some() {
            mission_state.processed_repair_charge += deposit.resource_amount;
        }
        mission_state.recent_action = Some(format!("Deposited {} aboard ship", deposit.label));
        mission_state.recent_action_timer = Fx::from_num(1.5);
        carried.kind = None;
        carried.amount = 0;
        return;
    }
}

struct CarriedItemDeposit {
    resource_kind: Option<ResourceKind>,
    resource_amount: u32,
    component: Option<(ModuleKind, ModuleVariant, u32)>,
    label: String,
}

fn carried_item_deposit(kind: CarriedItemKind, amount: u32) -> Option<CarriedItemDeposit> {
    match kind {
        CarriedItemKind::Resource(resource_kind) => Some(CarriedItemDeposit {
            resource_kind: Some(resource_kind),
            resource_amount: amount,
            component: None,
            label: format!(
                "{} {}",
                amount,
                CarriedItemKind::Resource(resource_kind).label()
            ),
        }),
        CarriedItemKind::ExtractedComponent { kind, variant } => Some(CarriedItemDeposit {
            resource_kind: None,
            resource_amount: 0,
            component: Some((kind, variant, amount.max(1))),
            label: format!(
                "damaged {} {} component",
                variant.display_name(),
                kind.as_str()
            ),
        }),
        CarriedItemKind::Suit(_) => None,
    }
}
