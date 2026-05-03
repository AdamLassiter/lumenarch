use super::*;

pub(crate) fn return_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<AbortEncounterButton>,
        ),
    >,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.44, 0.20, 0.14));
                pending_meta.0 = Some(netcode::PendingMetaCommand {
                    op: netcode::RollbackMetaOp::ReturnToDock,
                    ..Default::default()
                });
                log::info!("Abort Encounter button pressed");
                log::info!("Returning to Docked state");
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.64, 0.34, 0.22));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.52, 0.27, 0.18));
            }
        }
    }
}

pub(crate) fn return_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        pending_meta.0 = Some(netcode::PendingMetaCommand {
            op: netcode::RollbackMetaOp::ReturnToDock,
            ..Default::default()
        });
        log::info!("Tab key pressed");
        log::info!("Returning to Docked state");
    }
}

pub(crate) fn camera_follow_player_ship(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    ship_query: Single<&ShipboardControlState, With<ObservedLocalPlayerMarker>>,
    player_query: Single<(&PlayerMotionState, &CurrentStation), With<ObservedLocalPlayerMarker>>,
    ship_frame_query: Query<(Entity, &SimPosition, &SimRotation), With<ShipRoot>>,
    module_query: Query<(&RuntimeShipModule, &ChildOf)>,
    camera_query: Single<(&mut Transform, &mut Projection), (With<Camera2d>, With<MainCamera>)>,
) {
    let control_state = ship_query.into_inner();
    let (player_motion, current_station) = player_query.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();

    let Projection::Orthographic(projection) = projection.deref_mut() else {
        return;
    };

    let active_ship = match player_motion.frame {
        PlayerReferenceFrame::World => None,
        PlayerReferenceFrame::Ship(ship_entity) => ship_frame_query.get(ship_entity).ok(),
    };
    let player_world = player_motion.world_position;
    let (desired_center, desired_rotation, desired_scale) = match control_state.mode {
        ShipControlMode::Interior => {
            if let Some((_, _, ship_rotation)) = active_ship {
                (
                    player_world,
                    ship_rotation.radians.to_num::<f32>(),
                    balance.player.interior_camera_scale,
                )
            } else {
                (player_world, 0.0, balance.player.interior_camera_scale)
            }
        }
        ShipControlMode::Reactor | ShipControlMode::Logistics | ShipControlMode::Computer => {
            let focus_pos = control_state
                .focused_entity
                .and_then(|entity| module_query.get(entity).ok())
                .and_then(|(runtime_module, parent)| {
                    ship_frame_query
                        .get(parent.get())
                        .ok()
                        .map(|(_, ship_pos, ship_rot)| {
                            ship_pos.value + runtime_module.local_position.rotate(ship_rot.radians)
                        })
                })
                .unwrap_or(player_world);
            let desired_rotation = active_ship
                .map(|(_, _, ship_rotation)| ship_rotation.radians.to_num::<f32>())
                .unwrap_or(0.0);
            (
                focus_pos,
                desired_rotation,
                balance.player.interior_camera_scale,
            )
        }
        ShipControlMode::Cockpit => {
            if let Some(ship_entity) = match player_motion.frame {
                PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
                PlayerReferenceFrame::World => None,
            } {
                if let Ok((_, ship_position, _)) = ship_frame_query.get(ship_entity) {
                    (
                        ship_position.value,
                        0.0,
                        balance.player.exterior_camera_scale,
                    )
                } else {
                    (player_world, 0.0, balance.player.exterior_camera_scale)
                }
            } else if let Some((_, parent)) = module_query
                .iter()
                .find(|(runtime_module, _)| runtime_module.module_id == current_station.module_id)
            {
                if let Ok((_, ship_position, _)) = ship_frame_query.get(parent.get()) {
                    (
                        ship_position.value,
                        0.0,
                        balance.player.exterior_camera_scale,
                    )
                } else {
                    (player_world, 0.0, balance.player.exterior_camera_scale)
                }
            } else {
                (player_world, 0.0, balance.player.exterior_camera_scale)
            }
        }
        ShipControlMode::Turret => {
            let focus_pos = control_state
                .focused_entity
                .and_then(|entity| module_query.get(entity).ok())
                .and_then(|(runtime_module, parent)| {
                    ship_frame_query
                        .get(parent.get())
                        .ok()
                        .map(|(_, ship_pos, ship_rot)| {
                            ship_pos.value + runtime_module.local_position.rotate(ship_rot.radians)
                        })
                })
                .or_else(|| match player_motion.frame {
                    PlayerReferenceFrame::Ship(ship_entity) => ship_frame_query
                        .get(ship_entity)
                        .ok()
                        .map(|(_, ship_position, _)| ship_position.value),
                    PlayerReferenceFrame::World => None,
                })
                .unwrap_or(player_world);
            (focus_pos, 0.0, balance.player.exterior_camera_scale)
        }
    };

    let blend = 1.0 - (-balance.combat.camera_follow_lerp_rate * time.delta_secs()).exp();
    let desired_center = desired_center.to_vec2();
    camera_transform.translation.x += (desired_center.x - camera_transform.translation.x) * blend;
    camera_transform.translation.y += (desired_center.y - camera_transform.translation.y) * blend;
    let current_angle = camera_transform.rotation.to_euler(EulerRot::XYZ).2;
    let angle_delta = helpers::wrap_angle_f32(desired_rotation - current_angle);
    camera_transform.rotation = Quat::from_rotation_z(current_angle + angle_delta * blend);
    projection.scale += (desired_scale - projection.scale) * blend;
}
