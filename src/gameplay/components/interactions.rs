use bevy::{ecs::entity::{EntityMapper, MapEntities}, prelude::*};

use super::super::helpers::Fx;

#[derive(Component, Default, Clone)]
pub(crate) struct NearbyInteraction {
    pub(crate) target: Option<Entity>,
    pub(crate) kind: Option<InteractionKind>,
    pub(crate) prompt: Option<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Component, Default, Clone)]
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
    pub(crate) player: Entity,
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}

#[derive(Event)]
pub(crate) struct BeginHeldInteraction {
    pub(crate) player: Entity,
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
    pub(crate) required: Fx,
}

#[derive(Event)]
pub(crate) struct CompleteHeldInteraction {
    pub(crate) player: Entity,
    pub(crate) target: Entity,
    pub(crate) kind: InteractionKind,
}

impl MapEntities for NearbyInteraction {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.target {
            self.target = Some(entity_mapper.map_entity(entity));
        }
    }
}

impl MapEntities for HeldInteraction {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.target {
            self.target = Some(entity_mapper.map_entity(entity));
        }
    }
}
