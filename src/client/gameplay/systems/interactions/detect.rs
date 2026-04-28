use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        CurrentStation,
        DestroyedModule,
        Integrity,
        ModuleRuntimeState,
        NearbyInteraction,
        RuntimeShipModule,
        ShipboardPlayer,
    },
    helpers::{interaction_for_module, interaction_prompt},
};

pub(crate) fn detect_nearby_interactions(
    module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &Integrity,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    player_query: Single<(&CurrentStation, &mut NearbyInteraction), With<ShipboardPlayer>>,
) {
    let (station, mut nearby) = player_query.into_inner();
    nearby.target = None;
    nearby.kind = None;
    nearby.prompt = None;
    nearby.unavailable_reason = None;

    let Some((entity, _, integrity, runtime_state, destroyed)) = module_query
        .iter()
        .find(|(_, runtime_module, _, _, _)| runtime_module.module_id == station.module_id)
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
