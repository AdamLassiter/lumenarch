use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        ArchComputerModule,
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
        RuntimeShipModule,
        RuntimeArchComputer,
        ShipAutomationMode,
        ShipAutomationState,
        ShipRoot,
        ShipboardControlState,
        StorageModule,
    },
    helpers::Fx,
};

pub(crate) fn apply_module_interactions(
    mut instant_events: EventReader<InteractWithModule>,
    mut complete_events: EventReader<CompleteHeldInteraction>,
    ship_query: Single<
        (
            &mut ShipboardControlState,
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
    let (mut ship_control, mut automation_state, mut mission_state) = ship_query.into_inner();
    for event in instant_events.read() {
        apply_instant_interaction(
            event,
            &mut ship_control,
            &mut automation_state,
            &mut mission_state,
            &mut module_query,
        );
    }

    for event in complete_events.read() {
        apply_completed_interaction(
            event,
            &mut mission_state,
            &mut module_query,
            &mut logistics_query,
        );
    }
}

fn apply_instant_interaction(
    event: &InteractWithModule,
    ship_control: &mut ShipboardControlState,
    automation_state: &mut ShipAutomationState,
    mission_state: &mut MissionState,
    module_query: &mut Query<(
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
            ship_control.mode = crate::client::gameplay::components::ShipControlMode::ShipFlight;
            set_recent_action(mission_state, "Returned to cockpit control", 1.5);
        }
        InteractionKind::Computer => {
            if let Ok((_, _, _, computer, arch_runtime, destroyed)) = module_query.get_mut(event.target)
                && computer.is_some()
                && destroyed.is_none()
            {
                automation_state.mode = ShipAutomationMode::Mixed;
                let label = if let Some(mut arch_runtime) = arch_runtime {
                    arch_runtime.enabled = !arch_runtime.enabled;
                    if arch_runtime.enabled {
                        format!("{} online", arch_runtime.program.name)
                    } else {
                        format!("{} offline", arch_runtime.program.name)
                    }
                } else {
                    "Computer toggled".to_string()
                };
                set_recent_action(mission_state, &label, 1.8);
            }
        }
        InteractionKind::Storage => {
            set_recent_action(mission_state, "Cargo hold inspected", 1.2);
        }
        InteractionKind::Processor => {
            set_recent_action(mission_state, "Processor inspected", 1.2);
        }
        InteractionKind::Turret => {
            if let Ok((_, _, mut runtime_state, _, _, destroyed)) = module_query.get_mut(event.target)
                && destroyed.is_none()
            {
                runtime_state.is_disabled = false;
                runtime_state.needs_attention = false;
                runtime_state.electrical_instability =
                    (runtime_state.electrical_instability - Fx::from_num(4)).max(Fx::from_num(0));
                runtime_state.last_interaction_age = Fx::from_num(0);
                set_recent_action(mission_state, "Turret reset complete", 1.5);
            }
        }
        InteractionKind::Engine => {
            if let Ok((_, _, mut runtime_state, _, _, destroyed)) = module_query.get_mut(event.target)
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
    mission_state: &mut MissionState,
    module_query: &mut Query<(
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
    if let Ok((runtime_module, mut integrity, mut runtime_state, _, _, destroyed)) =
        module_query.get_mut(event.target)
    {
        if destroyed.is_some() {
            return;
        }
        match event.kind {
            InteractionKind::Reactor => {
                runtime_state.current_heat =
                    (runtime_state.current_heat - Fx::from_num(6)).max(Fx::from_num(0));
                runtime_state.electrical_instability =
                    (runtime_state.electrical_instability - Fx::from_num(5)).max(Fx::from_num(0));
                runtime_state.needs_attention = false;
                runtime_state.is_disabled = false;
                mission_state.stabilizations_performed += 1;
                set_recent_action(mission_state, "Reactor stabilized", 2.0);
            }
            InteractionKind::Repair => {
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
            _ => {}
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
