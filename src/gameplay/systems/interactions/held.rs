use bevy::prelude::*;

use crate::gameplay::{
    components::{
        BeginHeldInteraction,
        CompleteHeldInteraction,
        HeldInteraction,
        MissionState,
        NearbyInteraction,
        PlayerFieldState,
        PlayerShip,
        ShipRoot,
        ShipboardPlayer,
    },
    helpers::{Fx, fx_from_time_delta},
};

pub(crate) fn begin_held_interactions(
    mut events: EventReader<BeginHeldInteraction>,
    player_query: Single<&mut HeldInteraction, With<ShipboardPlayer>>,
) {
    let mut held = player_query.into_inner();
    for event in events.read() {
        held.target = Some(event.target);
        held.kind = Some(event.kind);
        held.progress = Fx::from_num(0);
        held.required = event.required;
    }
}

pub(crate) fn complete_held_interactions(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    nearby_query: Single<&NearbyInteraction, With<ShipboardPlayer>>,
    player_query: Single<(&PlayerFieldState, &mut HeldInteraction), With<ShipboardPlayer>>,
    mut complete_events: EventWriter<CompleteHeldInteraction>,
) {
    let mission_state = mission_query.into_inner();
    let nearby = nearby_query.into_inner();
    let (player_fields, mut held) = player_query.into_inner();
    let Some(target) = held.target else {
        return;
    };

    if mission_state.failed
        || mission_state.completed
        || player_fields.local_heat >= Fx::from_num(15)
        || player_fields.local_electrical >= Fx::from_num(13)
        || !keys.pressed(KeyCode::KeyE)
        || nearby.target != Some(target)
        || nearby.kind != held.kind
    {
        *held = HeldInteraction::default();
        return;
    }

    held.progress += fx_from_time_delta(&time);
    if held.progress >= held.required {
        if let Some(kind) = held.kind {
            complete_events.send(CompleteHeldInteraction { target, kind });
        }
        *held = HeldInteraction::default();
    }
}
