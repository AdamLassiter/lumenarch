use bevy::prelude::*;

use crate::gameplay::{
    components::{
        CurrentStation,
        DestroyedModule,
        Integrity,
        ModuleRuntimeState,
        NearbyInteraction,
        PlayerMotionState,
        PlayerReferenceFrame,
        RuntimeShipModule,
        ShipboardPlayer,
    },
    helpers::{interaction_for_module, interaction_prompt},
};

pub(crate) fn detect_nearby_interactions(
    module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &Parent,
        &Integrity,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    player_query: Single<
        (&CurrentStation, &PlayerMotionState, &mut NearbyInteraction),
        With<ShipboardPlayer>,
    >,
) {
    let (station, player_motion, mut nearby) = player_query.into_inner();
    nearby.target = None;
    nearby.kind = None;
    nearby.prompt = None;
    nearby.unavailable_reason = None;

    let Some(active_ship) = (match player_motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
        PlayerReferenceFrame::World => None,
    }) else {
        nearby.unavailable_reason = Some("EVA: no nearby station".to_string());
        return;
    };

    let Some((entity, _, _, integrity, runtime_state, destroyed)) =
        module_query
            .iter()
            .find(|(_, runtime_module, parent, _, _, _)| {
                parent.get() == active_ship && runtime_module.module_id == station.module_id
            })
    else {
        nearby.unavailable_reason = Some("no reachable station".to_string());
        return;
    };

    if let Some(kind) =
        interaction_for_module(station.kind, integrity, runtime_state, destroyed.is_some())
    {
        nearby.target = Some(entity);
        nearby.kind = Some(kind);
        nearby.prompt = Some(interaction_prompt(kind).to_string());
    } else {
        nearby.unavailable_reason = Some("station is stable".to_string());
    }
}
