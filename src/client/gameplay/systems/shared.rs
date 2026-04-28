use super::super::components::{InteractionKind, ModuleCondition};
use crate::ship::ModuleKind;

pub(super) fn module_display_name(kind: ModuleKind) -> &'static str {
    kind.as_str()
}

pub(super) fn interaction_label(kind: InteractionKind) -> &'static str {
    match kind {
        InteractionKind::Cockpit => "enter cockpit station",
        InteractionKind::Computer => "open computer console",
        InteractionKind::Storage => "open storage panel",
        InteractionKind::Manipulator => "open manipulator panel",
        InteractionKind::Processor => "open processor panel",
        InteractionKind::Reactor => "operate reactor",
        InteractionKind::Turret => "man turret",
        InteractionKind::Engine => "reset engine",
        InteractionKind::Repair => "repair module",
    }
}

pub(super) fn condition_severity(condition: ModuleCondition) -> i32 {
    match condition {
        ModuleCondition::Healthy => 0,
        ModuleCondition::Degraded => 1,
        ModuleCondition::Disabled => 2,
        ModuleCondition::Destroyed => 3,
    }
}
