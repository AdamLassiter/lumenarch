use bevy::prelude::*;

use crate::{
    gameplay::{
        components::{
            BeginHeldInteraction,
            HeldInteraction,
            InteractWithModule,
            NearbyInteraction,
            PlayerHandleComponent,
            ShipboardPlayer,
        },
        helpers::{interaction_hold_duration, is_hold_interaction},
    },
    netcode::{self, LumenGgrsConfig},
};
use bevy_ggrs::PlayerInputs;

pub(crate) fn run_shipboard_interaction_input(
    session_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    player_query: Query<
        (
            Entity,
            &PlayerHandleComponent,
            &NearbyInteraction,
            &HeldInteraction,
        ),
        With<ShipboardPlayer>,
    >,
    mut interact_events: EventWriter<InteractWithModule>,
    mut begin_events: EventWriter<BeginHeldInteraction>,
) {
    let mut players: Vec<_> = player_query.iter().collect();
    players.sort_by_key(|(_, handle, _, _)| handle.handle);
    for (player, handle, nearby, held) in players {
        let input = session_inputs
            .as_ref()
            .and_then(|inputs| inputs.get(handle.handle))
            .map(|(input, _)| *input)
            .unwrap_or_default();
        let (Some(target), Some(kind)) = (nearby.target, nearby.kind) else {
            continue;
        };

        if is_hold_interaction(kind) {
            if input.pressed(netcode::INPUT_INTERACT) && held.target.is_none() {
                begin_events.send(BeginHeldInteraction {
                    player,
                    target,
                    kind,
                    required: interaction_hold_duration(kind),
                });
            }
        } else if input.pressed(netcode::INPUT_INTERACT) {
            interact_events.send(InteractWithModule {
                player,
                target,
                kind,
            });
        }
    }
}
