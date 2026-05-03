use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{super::helpers::Fx, CarriedItemKind};
use crate::ship::{ModuleKind, ModuleVariant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceKind {
    RawSalvage,
    RepairCharge,
    Fuel,
    Ammunition,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ResourceInventory {
    pub(crate) raw_salvage: u32,
    pub(crate) repair_charge: u32,
    pub(crate) fuel: u32,
    pub(crate) ammunition: u32,
}

impl ResourceInventory {
    pub(crate) fn get(self, kind: ResourceKind) -> u32 {
        match kind {
            ResourceKind::RawSalvage => self.raw_salvage,
            ResourceKind::RepairCharge => self.repair_charge,
            ResourceKind::Fuel => self.fuel,
            ResourceKind::Ammunition => self.ammunition,
        }
    }

    pub(crate) fn add(&mut self, kind: ResourceKind, amount: u32) {
        match kind {
            ResourceKind::RawSalvage => self.raw_salvage += amount,
            ResourceKind::RepairCharge => self.repair_charge += amount,
            ResourceKind::Fuel => self.fuel += amount,
            ResourceKind::Ammunition => self.ammunition += amount,
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
        }
        taken
    }

    pub(crate) fn total_units(self) -> u32 {
        self.raw_salvage + self.repair_charge + self.fuel + self.ammunition
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
}

impl StorageModule {
    pub(crate) fn accepts(&self, kind: ResourceKind) -> bool {
        match kind {
            ResourceKind::Fuel => self.accepts_fuel,
            ResourceKind::Ammunition => self.accepts_ammunition,
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
