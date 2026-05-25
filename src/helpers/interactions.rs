use super::Fx;
use crate::{
    balance::BalanceConfig,
    gameplay::components::{Integrity, InteractionKind, ModuleRuntimeState, ResourceKind},
    ship::{ModuleKind, ModuleVariant},
};

pub(crate) fn sprite_path_for_kind(kind: &ModuleKind, variant: ModuleVariant) -> String {
    let _ = variant;
    match kind {
        ModuleKind::Airlock | ModuleKind::Engine => format!("tiles/hull/{}.png", kind.as_str()),
        ModuleKind::Turret => "tiles/hull/hardpoint.png".to_string(),
        ModuleKind::Shield => "tiles/components/battery.png".to_string(),
        ModuleKind::Detector => "tiles/components/computer.png".to_string(),
        _ => format!("tiles/components/{}.png", kind.as_str()),
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

pub(crate) fn module_can_be_extracted(
    kind: ModuleKind,
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> bool {
    module_is_salvageable(kind)
        && !runtime_state.extracted
        && (destroyed
            || integrity.current < integrity.max
            || runtime_state.needs_attention
            || runtime_state.is_disabled)
}

pub(crate) fn module_is_salvageable(kind: ModuleKind) -> bool {
    !matches!(
        kind,
        ModuleKind::Core
            | ModuleKind::Hull
            | ModuleKind::HullInnerCorner
            | ModuleKind::HullOuterCorner
            | ModuleKind::Interior
            | ModuleKind::InteriorWall
            | ModuleKind::Cockpit
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn integrity(current: i32, max: i32) -> Integrity {
        Integrity { current, max }
    }

    fn runtime_state() -> ModuleRuntimeState {
        ModuleRuntimeState {
            current_heat: Fx::from_num(0),
            electrical_instability: Fx::from_num(0),
            sampled_heat: Fx::from_num(0),
            sampled_electrical: Fx::from_num(0),
            is_disabled: false,
            was_disabled_last_frame: false,
            needs_attention: false,
            extracted: false,
            last_interaction_age: Fx::from_num(0),
        }
    }

    #[test]
    fn damaged_generic_module_uses_repair_interaction() {
        let runtime_state = runtime_state();

        assert!(
            interaction_for_module(
                ModuleKind::Battery,
                &integrity(3, 10),
                &runtime_state,
                false
            ) == Some(InteractionKind::Repair)
        );
    }

    #[test]
    fn station_interactions_win_when_station_is_healthy() {
        let runtime_state = runtime_state();
        let integrity = integrity(10, 10);

        assert!(
            interaction_for_module(ModuleKind::Computer, &integrity, &runtime_state, false)
                == Some(InteractionKind::Computer)
        );
        assert!(
            interaction_for_module(ModuleKind::Cockpit, &integrity, &runtime_state, false)
                == Some(InteractionKind::Cockpit)
        );
        assert!(
            interaction_for_module(ModuleKind::Reactor, &integrity, &runtime_state, false)
                == Some(InteractionKind::Reactor)
        );
    }

    #[test]
    fn destroyed_modules_are_not_interactable() {
        let runtime_state = runtime_state();

        assert!(
            interaction_for_module(
                ModuleKind::Computer,
                &integrity(10, 10),
                &runtime_state,
                true
            )
            .is_none()
        );
        assert!(!module_needs_repair(
            &integrity(1, 10),
            &runtime_state,
            true
        ));
    }

    #[test]
    fn extraction_requires_salvageable_damaged_or_destroyed_modules() {
        let runtime_state = runtime_state();

        assert!(module_can_be_extracted(
            ModuleKind::Turret,
            &integrity(4, 10),
            &runtime_state,
            false
        ));
        assert!(module_can_be_extracted(
            ModuleKind::Engine,
            &integrity(10, 10),
            &runtime_state,
            true
        ));
        assert!(!module_can_be_extracted(
            ModuleKind::Turret,
            &integrity(10, 10),
            &runtime_state,
            false
        ));
        assert!(!module_can_be_extracted(
            ModuleKind::Core,
            &integrity(4, 10),
            &runtime_state,
            false
        ));
        assert!(!module_can_be_extracted(
            ModuleKind::Hull,
            &integrity(4, 10),
            &runtime_state,
            false
        ));
        assert!(!module_can_be_extracted(
            ModuleKind::Cockpit,
            &integrity(4, 10),
            &runtime_state,
            false
        ));
    }
}
