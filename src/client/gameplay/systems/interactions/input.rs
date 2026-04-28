use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        BeginHeldInteraction,
        HeldInteraction,
        InteractWithModule,
        NearbyInteraction,
        ShipboardPlayer,
    },
    helpers::{interaction_hold_duration, is_hold_interaction},
};

pub(crate) fn run_shipboard_interaction_input(
    keys: Res<ButtonInput<KeyCode>>,
    player_query: Single<(&NearbyInteraction, &HeldInteraction), With<ShipboardPlayer>>,
    mut interact_events: EventWriter<InteractWithModule>,
    mut begin_events: EventWriter<BeginHeldInteraction>,
) {
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
