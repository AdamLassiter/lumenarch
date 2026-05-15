use super::Fx;
use crate::{
    balance::BalanceConfig,
    gameplay::components::{Integrity, InteractionKind, ModuleRuntimeState, ResourceKind},
    ship::{ModuleKind, ModuleVariant},
};

pub(crate) fn sprite_path_for_kind(kind: &ModuleKind, variant: ModuleVariant) -> String {
    let _ = variant;
    match kind {
        ModuleKind::Turret => "tiles/hardpoint.png".to_string(),
        ModuleKind::Shield => "tiles/battery.png".to_string(),
        ModuleKind::Detector => "tiles/computer.png".to_string(),
        _ => format!("tiles/{}.png", kind.as_str()),
    }
}

pub(crate) fn interaction_for_module(
    kind: ModuleKind,
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> Option<InteractionKind> {
    if destroyed {
        return None;
    }
    if kind == ModuleKind::Computer {
        return Some(InteractionKind::Computer);
    }
    if kind == ModuleKind::Cockpit {
        return Some(InteractionKind::Cockpit);
    }
    if kind == ModuleKind::Cargo || kind == ModuleKind::O2Generator {
        return Some(InteractionKind::Storage);
    }
    if kind == ModuleKind::Airlock {
        return Some(InteractionKind::Manipulator);
    }
    if kind == ModuleKind::Processor {
        return Some(InteractionKind::Processor);
    }
    if kind == ModuleKind::Reactor {
        return Some(InteractionKind::Reactor);
    }
    if kind == ModuleKind::Turret {
        return Some(InteractionKind::Turret);
    }
    if matches!(kind, ModuleKind::JunctionBox | ModuleKind::Valve) {
        return Some(InteractionKind::Manipulator);
    }
    if kind == ModuleKind::Engine && (runtime_state.is_disabled || runtime_state.needs_attention) {
        return Some(InteractionKind::Engine);
    }
    if integrity.current < integrity.max
        || runtime_state.needs_attention
        || runtime_state.is_disabled
    {
        return Some(InteractionKind::Repair);
    }
    None
}

pub(crate) fn module_needs_repair(
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> bool {
    !destroyed
        && (integrity.current < integrity.max
            || runtime_state.needs_attention
            || runtime_state.is_disabled)
}

pub(crate) fn interaction_prompt(kind: InteractionKind) -> &'static str {
    match kind {
        InteractionKind::Cockpit => "E: enter cockpit station",
        InteractionKind::Computer => "E: open computer console",
        InteractionKind::Storage => "E: open storage panel",
        InteractionKind::Manipulator => "E: open manipulator panel",
        InteractionKind::Processor => "E: open processor panel",
        InteractionKind::Reactor => "E: operate reactor",
        InteractionKind::Turret => "E: man turret",
        InteractionKind::Engine => "E: reset engine",
        InteractionKind::Repair => "Hold E: repair module",
        InteractionKind::Extract => "Hold E: extract component",
    }
}

pub(crate) fn is_hold_interaction(kind: InteractionKind) -> bool {
    matches!(kind, InteractionKind::Repair | InteractionKind::Extract)
}

pub(crate) fn interaction_hold_duration(kind: InteractionKind, balance: &BalanceConfig) -> Fx {
    match kind {
        InteractionKind::Cockpit
        | InteractionKind::Computer
        | InteractionKind::Storage
        | InteractionKind::Manipulator
        | InteractionKind::Processor
        | InteractionKind::Turret
        | InteractionKind::Reactor
        | InteractionKind::Engine => Fx::from_num(0),
        InteractionKind::Repair => Fx::from_num(balance.interaction.repair_hold_duration),
        InteractionKind::Extract => Fx::from_num(balance.interaction.extract_hold_duration),
    }
}

pub(crate) fn resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::RawSalvage => "raw salvage",
        ResourceKind::RepairCharge => "repair charge",
        ResourceKind::Fuel => "fuel",
        ResourceKind::Ammunition => "ammunition",
        ResourceKind::Oxygen => "oxygen",
    }
}
