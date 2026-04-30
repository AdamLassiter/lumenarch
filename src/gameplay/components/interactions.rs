use bevy::prelude::*;

use super::super::helpers::Fx;

#[derive(Component, Default)]
pub(crate) struct NearbyInteraction {
    pub(crate) target: Option<Entity>,
    pub(crate) kind: Option<InteractionKind>,
    pub(crate) prompt: Option<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Component, Default)]
pub(crate) struct HeldInteraction {
    pub(crate) target: Option<Entity>,
    pub(crate) kind: Option<InteractionKind>,
    pub(crate) progress: Fx,
    pub(crate) required: Fx,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InteractionKind {
    Cockpit,
    Computer,
    Storage,
    Manipulator,
    Processor,
    Reactor,
    Turret,
    Engine,
    Repair,
}

#[derive(Component)]
pub(crate) struct Interactable;

#[derive(Event)]
pub(crate) struct InteractWithModule {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}

#[derive(Event)]
pub(crate) struct BeginHeldInteraction {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
    pub(crate) required: Fx,
}

#[derive(Event)]
pub(crate) struct CompleteHeldInteraction {
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}
