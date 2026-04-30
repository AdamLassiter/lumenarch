use bevy::prelude::*;

use crate::{
    gameplay::{
        components::{
            BeginHeldInteraction,
            HeldInteraction,
            InteractWithModule,
            NearbyInteraction,
            ShipboardPlayer,
        },
        helpers::{interaction_hold_duration, is_hold_interaction},
    },
    state::{ConnectionPhase, ConnectionStatus},
};

pub(crate) fn run_shipboard_interaction_input(
    status: Res<ConnectionStatus>,
    keys: Res<ButtonInput<KeyCode>>,
    player_query: Single<(&NearbyInteraction, &HeldInteraction), With<ShipboardPlayer>>,
    mut interact_events: EventWriter<InteractWithModule>,
    mut begin_events: EventWriter<BeginHeldInteraction>,
) {
    if matches!(status.phase, ConnectionPhase::Connected) {
        return;
    }
    let (nearby, held) = player_query.into_inner();
    let (Some(target), Some(kind)) = (nearby.target, nearby.kind) else {
        return;
    };

    if is_hold_interaction(kind) {
        if keys.just_pressed(KeyCode::KeyE) && held.target.is_none() {
            begin_events.send(BeginHeldInteraction {
                target,
                kind,
                required: interaction_hold_duration(kind),
            });
        }
    } else if keys.just_pressed(KeyCode::KeyE) {
        interact_events.send(InteractWithModule { target, kind });
    }
}
