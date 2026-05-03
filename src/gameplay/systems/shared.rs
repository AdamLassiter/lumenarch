use super::super::components::{InteractionKind, ModuleCondition};
use crate::ship::ModuleKind;

pub(super) fn module_display_name(kind: ModuleKind) -> String {
    sentence_case(kind.as_str())
}

fn sentence_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
        }
    }
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
        InteractionKind::Extract => "extract component",
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
