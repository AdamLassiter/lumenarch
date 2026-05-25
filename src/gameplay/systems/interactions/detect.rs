use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::gameplay::{
    components::{
        CurrentStation,
        DestroyedModule,
        EquippedSuit,
        HostileShip,
        Integrity,
        InteractionKind,
        ModuleRuntimeState,
        NearbyInteraction,
        PlayerHandleComponent,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerSuit,
        RuntimeShipModule,
        ShipRoot,
        ShipboardPlayer,
    },
    helpers::{
        interaction_for_module,
        interaction_prompt,
        module_can_be_extracted,
        module_needs_repair,
    },
};

/// Finds the nearest valid module interaction for each player so contextual prompts feel immediate.
pub(crate) fn detect_nearby_interactions(
    ship_query: Query<Option<&HostileShip>, With<ShipRoot>>,
    module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ChildOf,
        &Integrity,
        &ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &CurrentStation,
            &PlayerMotionState,
            &EquippedSuit,
            &mut NearbyInteraction,
        ),
        With<ShipboardPlayer>,
    >,
) {
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _, _, _)| handle.handle);
    for (_, station, player_motion, equipped_suit, mut nearby) in players {
        nearby.target = None;
        nearby.kind = None;
        nearby.prompt = None;
        nearby.unavailable_reason = None;

        let Some(active_ship) = (match player_motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
            PlayerReferenceFrame::World => None,
        }) else {
            nearby.unavailable_reason = Some("EVA: no nearby station".to_string());
            continue;
        };
        let on_hostile_ship = ship_query.get(active_ship).ok().flatten().is_some();

        let Some((entity, runtime_module, _, integrity, runtime_state, destroyed)) = module_query
            .iter()
            .find(|(_, runtime_module, parent, _, _, _)| {
                parent.get() == active_ship && runtime_module.module_id == station.module_id
            })
        else {
            nearby.unavailable_reason = Some("no reachable station".to_string());
            continue;
        };

        let needs_repair = module_needs_repair(integrity, runtime_state, destroyed.is_some());

        if on_hostile_ship
            && module_can_be_extracted(
                runtime_module.kind,
                integrity,
                runtime_state,
                destroyed.is_some(),
            )
        {
            if equipped_suit.suit != PlayerSuit::Welder {
                nearby.unavailable_reason = Some("need welder suit for extraction".to_string());
            } else {
                nearby.target = Some(entity);
                nearby.kind = Some(InteractionKind::Extract);
                nearby.prompt = Some(interaction_prompt(InteractionKind::Extract).to_string());
            }
        } else if needs_repair {
            if equipped_suit.suit != PlayerSuit::Welder {
                nearby.unavailable_reason = Some("need welder suit for repairs".to_string());
            } else {
                nearby.target = Some(entity);
                nearby.kind = Some(InteractionKind::Repair);
                nearby.prompt = Some(interaction_prompt(InteractionKind::Repair).to_string());
            }
        } else if let Some(kind) =
            interaction_for_module(station.kind, integrity, runtime_state, destroyed.is_some())
        {
            if kind == InteractionKind::Repair && equipped_suit.suit != PlayerSuit::Welder {
                nearby.unavailable_reason = Some("need welder suit for repairs".to_string());
            } else {
                nearby.target = Some(entity);
                nearby.kind = Some(kind);
                nearby.prompt = Some(interaction_prompt(kind).to_string());
            }
        } else {
            nearby.unavailable_reason = Some("station is stable".to_string());
        }
    }
}
