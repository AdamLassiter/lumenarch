use bevy::prelude::*;

use super::super::{
    components::{
        BeginHeldInteraction, CompleteHeldInteraction, CurrentStation, DestroyedModule, HeldInteraction,
        Integrity, InteractWithModule, InteractionKind, ModuleRuntimeState, NearbyInteraction,
        PlayerShip, RuntimeShipModule, ShipRoot, ShipboardPlayer,
    },
    helpers::{
        interaction_for_module, interaction_hold_duration, interaction_prompt, is_hold_interaction,
        Fx,
    },
};

pub(crate) fn detect_nearby_interactions(
    module_query: Query<
        (
            Entity,
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&DestroyedModule>,
        ),
    >,
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

    if let Some(kind) = interaction_for_module(station.kind, integrity, runtime_state, destroyed.is_some()) {
        nearby.target = Some(entity);
        nearby.kind = Some(kind);
        nearby.prompt = Some(interaction_prompt(kind).to_string());
    } else {
        nearby.unavailable_reason = Some("station is stable".to_string());
    }
}

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
    nearby_query: Single<&NearbyInteraction, With<ShipboardPlayer>>,
    player_query: Single<&mut HeldInteraction, With<ShipboardPlayer>>,
    mut complete_events: EventWriter<CompleteHeldInteraction>,
) {
    let nearby = nearby_query.into_inner();
    let mut held = player_query.into_inner();
    let Some(target) = held.target else {
        return;
    };

    if !keys.pressed(KeyCode::KeyE) || nearby.target != Some(target) || nearby.kind != held.kind {
        *held = HeldInteraction::default();
        return;
    }

    held.progress += super::super::helpers::fx_from_time_delta(&time);
    if held.progress >= held.required {
        if let Some(kind) = held.kind {
            complete_events.send(CompleteHeldInteraction { target, kind });
        }
        *held = HeldInteraction::default();
    }
}

pub(crate) fn apply_module_interactions(
    mut instant_events: EventReader<InteractWithModule>,
    mut complete_events: EventReader<CompleteHeldInteraction>,
    ship_query: Single<&mut super::super::components::ShipboardControlState, (With<PlayerShip>, With<ShipRoot>)>,
    mut module_query: Query<(&mut Integrity, &mut ModuleRuntimeState, Option<&DestroyedModule>)>,
) {
    let mut ship_control = ship_query.into_inner();
    for event in instant_events.read() {
        match event.kind {
            InteractionKind::Cockpit => {
                ship_control.mode = super::super::components::ShipControlMode::ShipFlight;
            }
            InteractionKind::Turret => {
                if let Ok((_, mut runtime_state, destroyed)) = module_query.get_mut(event.target) {
                    if destroyed.is_none() {
                        runtime_state.is_disabled = false;
                        runtime_state.needs_attention = false;
                        runtime_state.electrical_instability =
                            (runtime_state.electrical_instability - Fx::from_num(4)).max(Fx::from_num(0));
                        runtime_state.last_interaction_age = Fx::from_num(0);
                    }
                }
            }
            _ => {}
        }
    }

    for event in complete_events.read() {
        if let Ok((mut integrity, mut runtime_state, destroyed)) = module_query.get_mut(event.target) {
            if destroyed.is_some() {
                continue;
            }
            match event.kind {
                InteractionKind::Reactor => {
                    runtime_state.current_heat =
                        (runtime_state.current_heat - Fx::from_num(6)).max(Fx::from_num(0));
                    runtime_state.electrical_instability =
                        (runtime_state.electrical_instability - Fx::from_num(5)).max(Fx::from_num(0));
                    runtime_state.needs_attention = false;
                    runtime_state.is_disabled = false;
                }
                InteractionKind::Repair => {
                    integrity.current = (integrity.current + 3).min(integrity.max);
                    runtime_state.current_heat =
                        (runtime_state.current_heat - Fx::from_num(3)).max(Fx::from_num(0));
                    runtime_state.electrical_instability =
                        (runtime_state.electrical_instability - Fx::from_num(3)).max(Fx::from_num(0));
                    runtime_state.needs_attention = integrity.current < integrity.max;
                    runtime_state.is_disabled = false;
                }
                _ => {}
            }
            runtime_state.last_interaction_age = Fx::from_num(0);
        }
    }
}
