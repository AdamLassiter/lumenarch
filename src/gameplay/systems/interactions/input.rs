use bevy::prelude::*;
use bevy_ggrs::PlayerInputs;

use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            BeginHeldInteraction,
            HeldInteraction,
            InteractWithModule,
            NearbyInteraction,
            PlayerConditionState,
            PlayerHandleComponent,
            ShipboardPlayer,
        },
        helpers::{interaction_hold_duration, is_hold_interaction},
    },
    netcode::{self, LumenGgrsConfig},
};

/// Converts local interaction input into begin/complete messages so shipboard actions stay rollback-safe.
pub(crate) fn run_shipboard_interaction_input(
    balance: Res<BalanceConfig>,
    session_inputs: Option<Res<PlayerInputs<LumenGgrsConfig>>>,
    player_query: Query<
        (
            Entity,
            &PlayerHandleComponent,
            &PlayerConditionState,
            &NearbyInteraction,
            &HeldInteraction,
        ),
        With<ShipboardPlayer>,
    >,
    mut interact_events: MessageWriter<InteractWithModule>,
    mut begin_events: MessageWriter<BeginHeldInteraction>,
) {
    let mut players: Vec<_> = player_query.iter().collect();
    players.sort_by_key(|(_, handle, _, _, _)| handle.handle);
    for (player, handle, condition, nearby, held) in players {
        if condition.control_disabled() {
            continue;
        }
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
                begin_events.write(BeginHeldInteraction {
                    player,
                    target,
                    kind,
                    required: interaction_hold_duration(kind, &balance),
                });
            }
        } else if input.pressed(netcode::INPUT_INTERACT) {
            interact_events.write(InteractWithModule {
                player,
                target,
                kind,
            });
        }
    }
}
