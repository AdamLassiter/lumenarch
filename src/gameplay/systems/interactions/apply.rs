use bevy::prelude::*;

use super::super::control::focus_station;
use crate::gameplay::{
    components::{
        ArchComputerModule,
        CarriedItemKind,
        CarriedResource,
        CompleteHeldInteraction,
        DestroyedModule,
        Integrity,
        InteractWithModule,
        InteractionKind,
        ManipulatorModule,
        MissionState,
        ModuleRuntimeState,
        PlayerShip,
        ProcessorModule,
        ResourceKind,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipControlMode,
        ShipRoot,
        ShipboardControlState,
        ShipboardPlayer,
        StationFamily,
        StorageModule,
    },
    helpers::Fx,
};

pub(crate) fn apply_module_interactions(
    mut instant_events: MessageReader<InteractWithModule>,
    mut complete_events: MessageReader<CompleteHeldInteraction>,
    mut player_query: Query<&mut ShipboardControlState, With<ShipboardPlayer>>,
    mut player_cargo_query: Query<&mut CarriedResource, With<ShipboardPlayer>>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &mut Integrity,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&DestroyedModule>,
    )>,
    mut logistics_query: Query<(
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&ManipulatorModule>,
        Option<&DestroyedModule>,
    )>,
) {
    let mut mission_state = mission_query.into_inner();
    for event in instant_events.read() {
        let Ok(mut ship_control) = player_query.get_mut(event.player) else {
            continue;
        };
        apply_instant_interaction(
            event,
            &mut ship_control,
            &mut mission_state,
            &mut module_query,
        );
    }

    for event in complete_events.read() {
        apply_completed_interaction(
            event,
            &mut player_cargo_query,
            &mut mission_state,
            &mut module_query,
            &mut logistics_query,
        );
    }
}

fn apply_instant_interaction(
    event: &InteractWithModule,
    ship_control: &mut ShipboardControlState,
    mission_state: &mut MissionState,
    module_query: &mut Query<(
        Entity,
        &RuntimeShipModule,
        &mut Integrity,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&DestroyedModule>,
    )>,
) {
    match event.kind {
        InteractionKind::Cockpit => {
            if let Ok((entity, runtime_module, _, _, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Cockpit,
                    ShipControlMode::Cockpit,
                );
                set_recent_action(mission_state, "Entered cockpit station", 1.5);
            }
        }
        InteractionKind::Computer => {
            if let Ok((entity, runtime_module, _, _, computer, arch_runtime, destroyed)) =
                module_query.get_mut(event.target)
                && computer.is_some()
                && destroyed.is_none()
                && let Some(arch_runtime) = arch_runtime
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Computer,
                    ShipControlMode::Computer,
                );
                set_recent_action(
                    mission_state,
                    &format!("Opened {} console", arch_runtime.program.name),
                    1.8,
                );
            }
        }
        InteractionKind::Storage => {
            if let Ok((entity, runtime_module, _, _, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Storage,
                    ShipControlMode::Logistics,
                );
                set_recent_action(mission_state, "Opened storage panel", 1.2);
            }
        }
        InteractionKind::Manipulator => {
            if let Ok((entity, runtime_module, _, _, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Manipulator,
                    ShipControlMode::Logistics,
                );
                set_recent_action(mission_state, "Opened manipulator panel", 1.2);
            }
        }
        InteractionKind::Processor => {
            if let Ok((entity, runtime_module, _, _, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Processor,
                    ShipControlMode::Logistics,
                );
                set_recent_action(mission_state, "Opened processor panel", 1.2);
            }
        }
        InteractionKind::Reactor => {
            if let Ok((entity, runtime_module, _, _, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Reactor,
                    ShipControlMode::Reactor,
                );
                set_recent_action(mission_state, "Opened reactor controls", 1.5);
            }
        }
        InteractionKind::Turret => {
            if let Ok((entity, runtime_module, _, mut runtime_state, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                runtime_state.is_disabled = false;
                runtime_state.needs_attention = false;
                focus_station(
                    ship_control,
                    entity,
                    runtime_module.module_id,
                    runtime_module.kind,
                    StationFamily::Turret,
                    ShipControlMode::Turret,
                );
                set_recent_action(mission_state, "Manned turret station", 1.5);
            }
        }
        InteractionKind::Engine => {
            if let Ok((_, _, _, mut runtime_state, _, _, destroyed)) =
                module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                runtime_state.current_heat =
                    (runtime_state.current_heat - Fx::from_num(3)).max(Fx::from_num(0));
                runtime_state.electrical_instability =
                    (runtime_state.electrical_instability - Fx::from_num(2)).max(Fx::from_num(0));
                runtime_state.is_disabled = false;
                runtime_state.needs_attention = false;
                runtime_state.last_interaction_age = Fx::from_num(0);
                set_recent_action(mission_state, "Engine reset complete", 1.5);
            }
        }
        _ => {}
    }
}

fn apply_completed_interaction(
    event: &CompleteHeldInteraction,
    player_cargo_query: &mut Query<&mut CarriedResource, With<ShipboardPlayer>>,
    mission_state: &mut MissionState,
    module_query: &mut Query<(
        Entity,
        &RuntimeShipModule,
        &mut Integrity,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&DestroyedModule>,
    )>,
    logistics_query: &mut Query<(
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&ManipulatorModule>,
        Option<&DestroyedModule>,
    )>,
) {
    if let Ok((_, runtime_module, mut integrity, mut runtime_state, _, _, destroyed)) =
        module_query.get_mut(event.target)
    {
        if event.kind == InteractionKind::Extract {
            let Ok(mut carried) = player_cargo_query.get_mut(event.player) else {
                return;
            };
            if carried.kind.is_some() {
                set_recent_action(mission_state, "Hands full - drop cargo first", 1.8);
                return;
            }
            carried.kind = Some(CarriedItemKind::ExtractedComponent {
                kind: runtime_module.kind,
                variant: runtime_module.variant,
            });
            carried.amount = 1;
            integrity.current = 0;
            runtime_state.is_disabled = true;
            runtime_state.needs_attention = false;
            runtime_state.extracted = true;
            set_recent_action(
                mission_state,
                &format!("Extracted {} component", runtime_module.kind.as_str()),
                2.0,
            );
            return;
        }
        if destroyed.is_some() {
            return;
        }
        if event.kind == InteractionKind::Repair {
            let used_repair_charge = try_consume_repair_charge(logistics_query);
            if used_repair_charge {
                integrity.current = (integrity.current + 6).min(integrity.max);
                runtime_state.current_heat =
                    (runtime_state.current_heat - Fx::from_num(5)).max(Fx::from_num(0));
                runtime_state.electrical_instability =
                    (runtime_state.electrical_instability - Fx::from_num(5)).max(Fx::from_num(0));
                mission_state.consumed_repair_charge += 1;
            } else {
                integrity.current = (integrity.current + 3).min(integrity.max);
                runtime_state.current_heat =
                    (runtime_state.current_heat - Fx::from_num(3)).max(Fx::from_num(0));
                runtime_state.electrical_instability =
                    (runtime_state.electrical_instability - Fx::from_num(3)).max(Fx::from_num(0));
            }
            runtime_state.needs_attention = integrity.current < integrity.max;
            runtime_state.is_disabled = false;
            mission_state.repairs_performed += 1;
            set_recent_action(
                mission_state,
                &if used_repair_charge {
                    format!("{} repaired with charge", runtime_module.kind.as_str())
                } else {
                    format!("{} field-patched", runtime_module.kind.as_str())
                },
                2.0,
            );
        }
        runtime_state.last_interaction_age = Fx::from_num(0);
    }
}

fn try_consume_repair_charge(
    logistics_query: &mut Query<(
        &RuntimeShipModule,
        Option<&mut StorageModule>,
        Option<&mut ProcessorModule>,
        Option<&ManipulatorModule>,
        Option<&DestroyedModule>,
    )>,
) -> bool {
    for (_, storage, processor, _, destroyed) in logistics_query.iter_mut() {
        if destroyed.is_some() {
            continue;
        }
        if let Some(mut storage) = storage
            && storage.inventory.remove(ResourceKind::RepairCharge, 1) > 0
        {
            return true;
        }
        if let Some(mut processor) = processor
            && processor.inventory.remove(ResourceKind::RepairCharge, 1) > 0
        {
            return true;
        }
    }
    false
}

fn set_recent_action(mission_state: &mut MissionState, label: &str, seconds: f32) {
    mission_state.recent_action = Some(label.to_string());
    mission_state.recent_action_timer = Fx::from_num(seconds);
}
