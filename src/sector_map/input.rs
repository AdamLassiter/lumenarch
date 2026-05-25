use bevy::{input::mouse::MouseWheel, log, prelude::*, window::PrimaryWindow};

use super::layout::node_button_color;
use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    state::{
        BackToStationButton,
        LaunchEncounterButton,
        SectorMapPanState,
        SectorMapViewState,
        SectorNodeButton,
        SectorNodeKind,
        SectorState,
    },
};

/// Pans and zooms the route map so players can inspect larger sectors without losing context.
pub(crate) fn pan_and_zoom_sector_map(
    window: Single<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut pan_state: ResMut<SectorMapPanState>,
    mut view_state: ResMut<SectorMapViewState>,
) {
    let window = window.into_inner();

    for event in mouse_wheel.read() {
        let zoom_step = (1.0 + event.y * 0.08).clamp(0.75, 1.25);
        view_state.zoom = (view_state.zoom * zoom_step).clamp(0.65, 1.8);
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        if let Some(cursor) = window.cursor_position() {
            if let Some(previous_cursor) = pan_state.last_cursor {
                view_state.offset += cursor - previous_cursor;
            }
            pan_state.last_cursor = Some(cursor);
        } else {
            pan_state.last_cursor = None;
        }
    } else {
        pan_state.last_cursor = None;
    }
}

/// Handles sector-node clicks so the host can inspect and queue reachable encounter destinations.
pub(crate) fn sector_node_button_system(
    mut interaction_query: Query<
        (&Interaction, &SectorNodeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    status: Res<netcode::SessionStatus>,
    mut sector_state: ResMut<SectorState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, button, mut background) in &mut interaction_query {
        let Some(node) = sector_state.node(button.node_id).cloned() else {
            continue;
        };
        let reachable = sector_state.is_reachable(button.node_id);
        let is_current = sector_state.current_node_id == button.node_id;
        match *interaction {
            Interaction::Pressed => {
                if reachable && !matches!(node.kind, SectorNodeKind::HubStation) {
                    sector_state.selected_node_id = Some(button.node_id);
                    log::debug!(
                        "Sector map queued SelectSectorNode meta command for node {}",
                        button.node_id
                    );
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::SelectSectorNode,
                        arg0: button.node_id as i16,
                        ..Default::default()
                    });
                }
                *background = BackgroundColor(PRESSED_BUTTON);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                let is_selected = sector_state.selected_node_id == Some(button.node_id);
                *background = BackgroundColor(node_button_color(
                    node.kind,
                    node.status,
                    reachable,
                    is_current,
                    is_selected,
                ));
            }
        }
    }
}

/// Applies launch and return button presses so sector-map navigation turns into rollback meta commands.
pub(crate) fn sector_navigation_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&LaunchEncounterButton>,
            Option<&BackToStationButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    sector_state: Res<SectorState>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background, launch, back) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if launch.is_some() {
                    *background = BackgroundColor(Color::srgb(0.12, 0.40, 0.24));
                    let queued_selection = pending_meta.0.as_ref().and_then(|command| {
                        matches!(command.op, netcode::RollbackMetaOp::SelectSectorNode)
                            .then_some(command.arg0.max(0) as u32)
                    });
                    if let Some(node_id) = queued_selection.or(sector_state.selected_node_id)
                        && sector_state.is_reachable(node_id)
                        && sector_state
                            .node(node_id)
                            .map(|node| !matches!(node.kind, SectorNodeKind::HubStation))
                            .unwrap_or(false)
                    {
                        log::debug!(
                            "Sector map queued LaunchEncounter meta command for node {}",
                            node_id
                        );
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LaunchEncounter,
                            arg0: node_id as i16,
                            ..Default::default()
                        });
                    }
                } else if back.is_some() {
                    *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                    log::debug!("Sector map queued ReturnToDock meta command");
                    pending_meta.0 = Some(netcode::PendingMetaCommand {
                        op: netcode::RollbackMetaOp::ReturnToDock,
                        ..Default::default()
                    });
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if launch.is_some() {
                    Color::srgb(0.18, 0.50, 0.30)
                } else {
                    Color::srgb(0.32, 0.24, 0.18)
                });
            }
        }
    }
}

/// Mirrors the sector-map back action onto Q/Escape so navigation screens share the same quit shortcut.
pub(crate) fn sector_navigation_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    status: Res<netcode::SessionStatus>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape) {
        log::debug!("Sector map queued ReturnToDock meta command from keyboard");
        pending_meta.0 = Some(netcode::PendingMetaCommand {
            op: netcode::RollbackMetaOp::ReturnToDock,
            ..Default::default()
        });
    }
}
