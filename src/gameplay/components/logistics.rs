use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use super::{
    super::helpers::{FixedVec2, Fx},
    CarriedItemKind,
    DroneTask,
};
use crate::ship::{ModuleKind, ModuleVariant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResourceKind {
    RawSalvage,
    RepairCharge,
    Fuel,
    Ammunition,
    Oxygen,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ResourceInventory {
    pub(crate) raw_salvage: u32,
    pub(crate) repair_charge: u32,
    pub(crate) fuel: u32,
    pub(crate) ammunition: u32,
    pub(crate) oxygen: u32,
}

impl ResourceInventory {
    pub(crate) fn get(self, kind: ResourceKind) -> u32 {
        match kind {
            ResourceKind::RawSalvage => self.raw_salvage,
            ResourceKind::RepairCharge => self.repair_charge,
            ResourceKind::Fuel => self.fuel,
            ResourceKind::Ammunition => self.ammunition,
            ResourceKind::Oxygen => self.oxygen,
        }
    }

    pub(crate) fn add(&mut self, kind: ResourceKind, amount: u32) {
        match kind {
            ResourceKind::RawSalvage => self.raw_salvage += amount,
            ResourceKind::RepairCharge => self.repair_charge += amount,
            ResourceKind::Fuel => self.fuel += amount,
            ResourceKind::Ammunition => self.ammunition += amount,
            ResourceKind::Oxygen => self.oxygen += amount,
        }
    }

    pub(crate) fn remove(&mut self, kind: ResourceKind, amount: u32) -> u32 {
        let available = self.get(kind);
        let taken = available.min(amount);
        match kind {
            ResourceKind::RawSalvage => self.raw_salvage -= taken,
            ResourceKind::RepairCharge => self.repair_charge -= taken,
            ResourceKind::Fuel => self.fuel -= taken,
            ResourceKind::Ammunition => self.ammunition -= taken,
            ResourceKind::Oxygen => self.oxygen -= taken,
        }
        taken
    }

    pub(crate) fn total_units(self) -> u32 {
        self.raw_salvage + self.repair_charge + self.fuel + self.ammunition + self.oxygen
    }
}

#[derive(Component, Clone)]
pub(crate) struct StorageModule {
    pub(crate) capacity: u32,
    pub(crate) inventory: ResourceInventory,
    pub(crate) damaged_components: Vec<StoredDamagedComponent>,
    pub(crate) accepts_fuel: bool,
    pub(crate) accepts_ammunition: bool,
    pub(crate) accepts_general: bool,
    pub(crate) accepts_oxygen: bool,
}

impl StorageModule {
    pub(crate) fn accepts(&self, kind: ResourceKind) -> bool {
        match kind {
            ResourceKind::Fuel => self.accepts_fuel,
            ResourceKind::Ammunition => self.accepts_ammunition,
            ResourceKind::Oxygen => self.accepts_oxygen,
            ResourceKind::RawSalvage | ResourceKind::RepairCharge => self.accepts_general,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredDamagedComponent {
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
    pub(crate) amount: u32,
}

impl StorageModule {
    pub(crate) fn add_damaged_component(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        if let Some(entry) = self
            .damaged_components
            .iter_mut()
            .find(|entry| entry.kind == kind && entry.variant == variant)
        {
            entry.amount += amount;
        } else {
            self.damaged_components.push(StoredDamagedComponent {
                kind,
                variant,
                amount,
            });
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct ManipulatorModule {
    pub(crate) transfer_progress: Fx,
    pub(crate) transfer_duration: Fx,
    pub(crate) active: bool,
    pub(crate) source_module_id: Option<u64>,
    pub(crate) target_module_id: Option<u64>,
    pub(crate) resource_kind: Option<ResourceKind>,
    pub(crate) blocked_reason: Option<String>,
}

#[derive(Component, Clone)]
pub(crate) struct ProcessorModule {
    pub(crate) progress: Fx,
    pub(crate) duration: Fx,
    pub(crate) active: bool,
    pub(crate) blocked_reason: Option<String>,
    pub(crate) inventory: ResourceInventory,
    pub(crate) input_required: u32,
    pub(crate) output_amount: u32,
}

#[derive(Component, Clone)]
pub(crate) struct LooseCargo {
    pub(crate) kind: CarriedItemKind,
    pub(crate) amount: u32,
}

#[derive(Component, Clone)]
pub(crate) struct DroneStationModule {
    pub(crate) max_drones: u32,
    pub(crate) operational_range: Fx,
    pub(crate) active_drones: u32,
    pub(crate) active_tasks: u32,
    pub(crate) queued_tasks: u32,
    pub(crate) idle_drones: u32,
    pub(crate) power_draw: Fx,
    pub(crate) last_status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DronePhase {
    Idle,
    Assigned,
    TravelingToSource,
    PickingUp,
    TravelingToDestination,
    Delivering,
    Returning,
}

impl DronePhase {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Assigned => "Assigned",
            Self::TravelingToSource => "Traveling to Source",
            Self::PickingUp => "Picking Up",
            Self::TravelingToDestination => "Traveling to Destination",
            Self::Delivering => "Delivering",
            Self::Returning => "Returning",
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct DroneUnit {
    pub(crate) station_entity: Entity,
    pub(crate) station_module_id: u64,
    pub(crate) home_local_position: FixedVec2,
    pub(crate) local_position: FixedVec2,
    pub(crate) phase: DronePhase,
    pub(crate) mode: DroneTask,
    pub(crate) source_module_id: Option<u64>,
    pub(crate) target_module_id: Option<u64>,
    pub(crate) resource_kind: Option<ResourceKind>,
    pub(crate) payload_amount: u32,
    pub(crate) reserved_amount: u32,
}

impl MapEntities for DroneUnit {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.station_entity = entity_mapper.get_mapped(self.station_entity);
    }
}
