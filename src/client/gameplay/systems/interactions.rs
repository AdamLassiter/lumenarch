use bevy::prelude::*;

use super::super::{
    components::{
        ArchComputerModule,
        BeginHeldInteraction,
        CompleteHeldInteraction,
        CurrentStation,
        DestroyedModule,
        HeldInteraction,
        Integrity,
        InteractWithModule,
        InteractionKind,
        MissionState,
        ModuleRuntimeState,
        NearbyInteraction,
        PlayerFieldState,
        PlayerShip,
        RuntimeShipModule,
        ShipAutomationMode,
        ShipAutomationState,
        ShipRoot,
        ShipboardPlayer,
    },
    helpers::{
        Fx,
        interaction_for_module,
        interaction_hold_duration,
        interaction_prompt,
        is_hold_interaction,
    },
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
    ship_query: Single<
        (
            &mut super::super::components::ShipboardControlState,
            &mut ShipAutomationState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    mut module_query: Query<(
        &RuntimeShipModule,
        &mut Integrity,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&DestroyedModule>,
    )>,
) {
    let (mut ship_control, mut automation_state, mut mission_state) = ship_query.into_inner();
    for event in instant_events.read() {
        match event.kind {
            InteractionKind::Cockpit => {
                ship_control.mode = super::super::components::ShipControlMode::ShipFlight;
                mission_state.recent_action = Some("Returned to cockpit control".to_string());
                mission_state.recent_action_timer = Fx::from_num(1.5);
            }
            InteractionKind::Computer => {
                if let Ok((_, _, _, computer, destroyed)) = module_query.get_mut(event.target)
                    && computer.is_some()
                    && destroyed.is_none()
                {
                    automation_state.mode = match automation_state.mode {
                        ShipAutomationMode::Off => ShipAutomationMode::ReactorGuard,
                        ShipAutomationMode::ReactorGuard => ShipAutomationMode::Off,
                    };
                    let label = match automation_state.mode {
                        ShipAutomationMode::Off => "Automation offline",
                        ShipAutomationMode::ReactorGuard => "Automation set: reactor guard",
                    };
                    mission_state.recent_action = Some(label.to_string());
                    mission_state.recent_action_timer = Fx::from_num(1.8);
                }
            }
            InteractionKind::Turret => {
                if let Ok((_, _, mut runtime_state, _, destroyed)) =
                    module_query.get_mut(event.target)
                    && destroyed.is_none()
                {
                    runtime_state.is_disabled = false;
                    runtime_state.needs_attention = false;
                    runtime_state.electrical_instability = (runtime_state.electrical_instability
                        - Fx::from_num(4))
                    .max(Fx::from_num(0));
                    runtime_state.last_interaction_age = Fx::from_num(0);
                    mission_state.recent_action = Some("Turret reset complete".to_string());
                    mission_state.recent_action_timer = Fx::from_num(1.5);
                }
            }
            InteractionKind::Engine => {
                if let Ok((_, _, mut runtime_state, _, destroyed)) =
                    module_query.get_mut(event.target)
                    && destroyed.is_none()
                {
                    runtime_state.current_heat =
                        (runtime_state.current_heat - Fx::from_num(3)).max(Fx::from_num(0));
                    runtime_state.electrical_instability = (runtime_state.electrical_instability
                        - Fx::from_num(2))
                    .max(Fx::from_num(0));
                    runtime_state.is_disabled = false;
                    runtime_state.needs_attention = false;
                    runtime_state.last_interaction_age = Fx::from_num(0);
                    mission_state.recent_action = Some("Engine reset complete".to_string());
                    mission_state.recent_action_timer = Fx::from_num(1.5);
                }
            }
            _ => {}
        }
    }

    for event in complete_events.read() {
        if let Ok((runtime_module, mut integrity, mut runtime_state, _, destroyed)) =
            module_query.get_mut(event.target)
        {
            if destroyed.is_some() {
                continue;
            }
            match event.kind {
                InteractionKind::Reactor => {
                    runtime_state.current_heat =
                        (runtime_state.current_heat - Fx::from_num(6)).max(Fx::from_num(0));
                    runtime_state.electrical_instability = (runtime_state.electrical_instability
                        - Fx::from_num(5))
                    .max(Fx::from_num(0));
                    runtime_state.needs_attention = false;
                    runtime_state.is_disabled = false;
                    mission_state.stabilizations_performed += 1;
                    mission_state.recent_action = Some("Reactor stabilized".to_string());
                    mission_state.recent_action_timer = Fx::from_num(2);
                }
                InteractionKind::Repair => {
                    integrity.current = (integrity.current + 3).min(integrity.max);
                    runtime_state.current_heat =
                        (runtime_state.current_heat - Fx::from_num(3)).max(Fx::from_num(0));
                    runtime_state.electrical_instability = (runtime_state.electrical_instability
                        - Fx::from_num(3))
                    .max(Fx::from_num(0));
                    runtime_state.needs_attention = integrity.current < integrity.max;
                    runtime_state.is_disabled = false;
                    mission_state.repairs_performed += 1;
                    mission_state.recent_action =
                        Some(format!("{} repaired", runtime_module.kind.as_str()));
                    mission_state.recent_action_timer = Fx::from_num(2);
                }
                _ => {}
            }
            runtime_state.last_interaction_age = Fx::from_num(0);
        }
    }
}
