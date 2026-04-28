use super::Fx;
use crate::client::gameplay::components::{
    Integrity,
    InteractionKind,
    ModuleRuntimeState,
    ResourceKind,
};
use crate::ship::ModuleKind;

pub(crate) fn sprite_path_for_kind(kind: &ModuleKind) -> String {
    match kind {
        ModuleKind::Computer => "tiles/battery.png".to_string(),
        ModuleKind::Turret => "tiles/hardpoint.png".to_string(),
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
    if kind == ModuleKind::Cargo {
        return Some(InteractionKind::Storage);
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

pub(crate) fn interaction_prompt(kind: InteractionKind) -> &'static str {
    match kind {
        InteractionKind::Cockpit => "E: return to flight controls",
        InteractionKind::Computer => "E: cycle automation mode",
        InteractionKind::Storage => "E: inspect cargo state",
        InteractionKind::Processor => "E: inspect processor state",
        InteractionKind::Reactor => "Hold E: stabilize reactor",
        InteractionKind::Turret => "E: reset turret",
        InteractionKind::Engine => "E: reset engine",
        InteractionKind::Repair => "Hold E: repair module",
    }
}

pub(crate) fn is_hold_interaction(kind: InteractionKind) -> bool {
    matches!(kind, InteractionKind::Reactor | InteractionKind::Repair)
}

pub(crate) fn interaction_hold_duration(kind: InteractionKind) -> Fx {
    match kind {
        InteractionKind::Cockpit
        | InteractionKind::Computer
        | InteractionKind::Storage
        | InteractionKind::Processor
        | InteractionKind::Turret
        | InteractionKind::Engine => Fx::from_num(0),
        InteractionKind::Reactor => Fx::from_num(1.2),
        InteractionKind::Repair => Fx::from_num(1.8),
    }
}

pub(crate) fn resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::RawSalvage => "raw salvage",
        ResourceKind::RepairCharge => "repair charge",
    }
}
