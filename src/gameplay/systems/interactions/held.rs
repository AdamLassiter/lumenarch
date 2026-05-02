use bevy::prelude::*;
use bevy_ggrs::PlayerInputs;

use crate::{
    gameplay::{
        components::{
            BeginHeldInteraction,
            CompleteHeldInteraction,
            HeldInteraction,
            MissionState,
            NearbyInteraction,
            PlayerFieldState,
            PlayerHandleComponent,
            PlayerShip,
            ShipRoot,
            ShipboardPlayer,
        },
        helpers::{Fx, fx_from_time_delta},
    },
    netcode::{self, LumenGgrsConfig},
};

pub(crate) fn begin_held_interactions(
    mut events: MessageReader<BeginHeldInteraction>,
    mut player_query: Query<&mut HeldInteraction, With<ShipboardPlayer>>,
) {
    for event in events.read() {
        let Ok(mut held) = player_query.get_mut(event.player) else {
            continue;
        };
        held.target = Some(event.target);
        held.kind = Some(event.kind);
        held.progress = Fx::from_num(0);
        held.required = event.required;
    }
}

pub(crate) fn complete_held_interactions(
    time: Res<Time>,
    session_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut player_query: Query<
        (
            Entity,
            &PlayerHandleComponent,
            &NearbyInteraction,
            &PlayerFieldState,
            &mut HeldInteraction,
        ),
        With<ShipboardPlayer>,
    >,
    mut complete_events: MessageWriter<CompleteHeldInteraction>,
) {
    let mission_state = mission_query.into_inner();
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(_, handle, _, _, _)| handle.handle);
    for (player, handle, nearby, player_fields, mut held) in players {
        let Some(target) = held.target else {
            continue;
        };
        let input_held = session_inputs
            .as_ref()
            .and_then(|inputs| inputs.get(handle.handle))
            .is_some_and(|(input, _)| input.pressed(netcode::INPUT_INTERACT));

        if mission_state.failed
            || mission_state.completed
            || player_fields.local_heat >= Fx::from_num(15)
            || player_fields.local_electrical >= Fx::from_num(13)
            || !input_held
            || nearby.target != Some(target)
            || nearby.kind != held.kind
        {
            *held = HeldInteraction::default();
            continue;
        }

        held.progress += fx_from_time_delta(&time);
        if held.progress >= held.required {
            if let Some(kind) = held.kind {
                complete_events.write(CompleteHeldInteraction {
                    player,
                    target,
                    kind,
                });
            }
            *held = HeldInteraction::default();
        }
    }
}
