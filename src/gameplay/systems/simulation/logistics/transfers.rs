use super::*;

pub(super) fn find_automation_transfer_task(
    snapshots: &[(
        Entity,
        u64,
        ModuleKind,
        FixedVec2,
        Option<(
            u32,
            crate::gameplay::components::ResourceInventory,
            bool,
            bool,
            bool,
        )>,
        Option<(u32, u32, crate::gameplay::components::ResourceInventory)>,
        bool,
    )],
    in_range: &impl Fn(FixedVec2) -> bool,
    preference: crate::gameplay::components::ArchLogisticsPreference,
) -> Option<(u64, u64, ResourceKind)> {
    for (_, source_id, _, source_pos, storage, _, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, source_inventory, _, _, _)) = storage else {
            continue;
        };
        if source_inventory.raw_salvage == 0 {
            continue;
        }
        for (_, target_id, target_kind, target_pos, _, processor, target_destroyed) in snapshots {
            if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                continue;
            }
            let Some((input_required, _, processor_inventory)) = processor else {
                continue;
            };
            if *target_kind != ModuleKind::Processor
                || processor_inventory.raw_salvage >= input_required * 2
            {
                continue;
            }
            return Some((*source_id, *target_id, ResourceKind::RawSalvage));
        }
    }

    if matches!(
        preference,
        crate::gameplay::components::ArchLogisticsPreference::FeedProcessor
    ) {
        return None;
    }

    for (_, source_id, source_kind, source_pos, _, processor, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, _, processor_inventory)) = processor else {
            continue;
        };
        if *source_kind != ModuleKind::Processor || processor_inventory.repair_charge == 0 {
            continue;
        }
        for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
            if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                continue;
            }
            let Some((capacity, storage_inventory, _, _, accepts_general)) = storage else {
                continue;
            };
            if *target_kind != ModuleKind::Cargo
                || !accepts_general
                || storage_inventory.total_units() >= *capacity
            {
                continue;
            }
            return Some((*source_id, *target_id, ResourceKind::RepairCharge));
        }
    }

    for (_, source_id, source_kind, source_pos, _, processor, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, _, processor_inventory)) = processor else {
            continue;
        };
        let outputs = [
            (ResourceKind::Fuel, processor_inventory.fuel),
            (ResourceKind::Ammunition, processor_inventory.ammunition),
        ];
        for (resource_kind, amount) in outputs {
            if amount == 0 {
                continue;
            }
            for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
                if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                    continue;
                }
                let Some((capacity, storage_inventory, accepts_fuel, accepts_ammunition, _)) =
                    storage
                else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => *accepts_fuel,
                    ResourceKind::Ammunition => *accepts_ammunition,
                    _ => false,
                };
                if !accepts
                    || *target_kind != ModuleKind::Cargo
                    || storage_inventory.total_units() >= *capacity
                {
                    continue;
                }
                return Some((*source_id, *target_id, resource_kind));
            }
        }
        let _ = source_kind;
    }

    None
}

pub(super) fn find_airlock_to_cargo_transfer(
    snapshots: &[(
        Entity,
        u64,
        ModuleKind,
        FixedVec2,
        Option<(
            u32,
            crate::gameplay::components::ResourceInventory,
            bool,
            bool,
            bool,
        )>,
        Option<(u32, u32, crate::gameplay::components::ResourceInventory)>,
        bool,
    )],
    in_range: &impl Fn(FixedVec2) -> bool,
) -> Option<(u64, u64, ResourceKind)> {
    for (_, source_id, source_kind, source_pos, storage, _, source_destroyed) in snapshots {
        if *source_destroyed || !in_range(*source_pos) {
            continue;
        }
        let Some((_, source_inventory, _, _, _)) = storage else {
            continue;
        };
        if *source_kind != ModuleKind::Airlock {
            continue;
        }
        for (resource_kind, amount) in [
            (ResourceKind::RawSalvage, source_inventory.raw_salvage),
            (ResourceKind::RepairCharge, source_inventory.repair_charge),
            (ResourceKind::Fuel, source_inventory.fuel),
            (ResourceKind::Ammunition, source_inventory.ammunition),
        ] {
            if amount == 0 {
                continue;
            }
            for (_, target_id, target_kind, target_pos, storage, _, target_destroyed) in snapshots {
                if *target_destroyed || !in_range(*target_pos) || source_id == target_id {
                    continue;
                }
                let Some((
                    capacity,
                    storage_inventory,
                    accepts_fuel,
                    accepts_ammunition,
                    accepts_general,
                )) = storage
                else {
                    continue;
                };
                let accepts = match resource_kind {
                    ResourceKind::Fuel => *accepts_fuel,
                    ResourceKind::Ammunition => *accepts_ammunition,
                    ResourceKind::RawSalvage | ResourceKind::RepairCharge => *accepts_general,
                };
                if *target_kind != ModuleKind::Cargo
                    || !accepts
                    || storage_inventory.total_units() >= *capacity
                {
                    continue;
                }
                return Some((*source_id, *target_id, resource_kind));
            }
        }
    }

    None
}
