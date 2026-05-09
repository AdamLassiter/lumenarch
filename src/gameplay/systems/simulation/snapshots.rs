use bevy::prelude::*;

use crate::{
    gameplay::{
        components::{
            ArchComputerModule,
            DestroyedModule,
            DetectorKind,
            DetectorModule,
            ModuleRuntimeState,
            ProcessorModule,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipDamageSensorState,
            ShipPowerState,
            ShipWeaponState,
            StorageModule,
        },
        helpers::Fx,
        systems::simulation::arch::{
            ArchSnapshot,
            LumenSnapshot,
            bool_fx,
            detector_axis,
            detector_metric,
        },
    },
    ship::ModuleKind,
};

pub(super) fn build_lumen_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    _damage_state: &ShipDamageSensorState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DetectorModule>,
        Option<&DestroyedModule>,
    )>,
) -> LumenSnapshot {
    let mut snapshot = LumenSnapshot {
        low_power: ship_power_state.stored_energy <= Fx::from_num(2),
        threat_present,
        ..Default::default()
    };

    for child in children.iter() {
        let Ok((
            _,
            runtime_module,
            runtime_state,
            computer,
            _,
            storage,
            processor,
            detector,
            destroyed,
        )) = module_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        match runtime_module.kind {
            ModuleKind::Reactor => snapshot.reactor_count += 1,
            ModuleKind::Turret => snapshot.turret_count += 1,
            ModuleKind::Cargo | ModuleKind::Airlock if storage.is_some() => {
                snapshot.cargo_count += 1;
            }
            ModuleKind::Processor if processor.is_some() => {
                snapshot.processor_count += 1;
            }
            ModuleKind::Computer if computer.is_some() => snapshot.computer_count += 1,
            ModuleKind::Detector if detector.is_some() => {}
            _ => {}
        }
        if runtime_state.current_heat >= Fx::from_num(8)
            || runtime_state.electrical_instability >= Fx::from_num(6)
        {
            snapshot.hot_module_count += 1;
        }
    }

    snapshot
}

pub(super) fn build_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    ship_weapon_state: &ShipWeaponState,
    damage_state: &ShipDamageSensorState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DetectorModule>,
        Option<&DestroyedModule>,
    )>,
) -> ArchSnapshot {
    let mut heat_sum = Fx::from_num(0);
    let mut live_modules = 0i32;
    let mut reactor_heat = Fx::from_num(0);
    let mut reactor_instability = Fx::from_num(0);
    let mut storage_raw = Fx::from_num(0);
    let mut storage_charge = Fx::from_num(0);
    let mut processor_raw = Fx::from_num(0);
    let mut processor_charge = Fx::from_num(0);
    let mut life_detector: Option<&DetectorModule> = None;
    let mut ship_detector: Option<&DetectorModule> = None;
    let mut damage_detector: Option<&DetectorModule> = None;
    let mut power_detector: Option<&DetectorModule> = None;
    let mut heat_detector: Option<&DetectorModule> = None;
    let mut logistics_detector: Option<&DetectorModule> = None;

    for child in children.iter() {
        let Ok((_, runtime_module, runtime_state, _, _, storage, processor, detector, destroyed)) =
            module_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        heat_sum += runtime_state.current_heat;
        live_modules += 1;
        match runtime_module.kind {
            ModuleKind::Reactor => {
                reactor_heat = reactor_heat.max(runtime_state.current_heat);
                reactor_instability = reactor_instability.max(runtime_state.electrical_instability);
            }
            ModuleKind::Cargo | ModuleKind::Airlock => {
                if let Some(storage) = storage {
                    storage_raw += Fx::from_num(storage.inventory.raw_salvage as i32);
                    storage_charge += Fx::from_num(storage.inventory.repair_charge as i32);
                }
            }
            ModuleKind::Processor => {
                if let Some(processor) = processor {
                    processor_raw += Fx::from_num(processor.inventory.raw_salvage as i32);
                    processor_charge += Fx::from_num(processor.inventory.repair_charge as i32);
                }
            }
            ModuleKind::Detector => {
                if let Some(detector) = detector {
                    let slot = match detector.kind {
                        DetectorKind::LifeSign => &mut life_detector,
                        DetectorKind::Ship => &mut ship_detector,
                        DetectorKind::Damage => &mut damage_detector,
                        DetectorKind::Power => &mut power_detector,
                        DetectorKind::Heat => &mut heat_detector,
                        DetectorKind::Logistics => &mut logistics_detector,
                    };
                    if slot.is_none_or(|current| detector.tier >= current.tier) {
                        *slot = Some(detector);
                    }
                }
            }
            _ => {}
        }
    }

    let avg_heat = if live_modules > 0 {
        heat_sum / Fx::from_num(live_modules)
    } else {
        Fx::from_num(0)
    };

    // These values are later overwritten by system-specific queries in execute_program context.
    ArchSnapshot {
        ship_power_reserve: ship_power_state.stored_energy,
        ship_average_heat: avg_heat,
        mission_threat: if threat_present {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        },
        reactor_heat,
        reactor_instability,
        storage_raw_salvage: storage_raw,
        storage_repair_charge: storage_charge,
        processor_raw_salvage: processor_raw,
        processor_repair_charge: processor_charge,
        turret_ready: if ship_weapon_state.turret_count > 0 {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        },
        turret_cooldown: ship_weapon_state.cooldown_remaining.max(Fx::from_num(0)),
        life_friendly_present: bool_fx(life_detector.is_some_and(|detector| detector.detected)),
        life_hostile_present: bool_fx(
            life_detector.is_some_and(|detector| detector.secondary_detected),
        ),
        life_dir_x: detector_axis(life_detector, |detector| detector.direction.x),
        life_dir_y: detector_axis(life_detector, |detector| detector.direction.y),
        life_distance: detector_metric(life_detector, |detector| detector.distance),
        ship_nearby_present: bool_fx(ship_detector.is_some_and(|detector| detector.detected)),
        ship_hostile_present: bool_fx(
            ship_detector.is_some_and(|detector| detector.secondary_detected),
        ),
        ship_dir_x: detector_axis(ship_detector, |detector| detector.direction.x),
        ship_dir_y: detector_axis(ship_detector, |detector| detector.direction.y),
        ship_distance: detector_metric(ship_detector, |detector| detector.distance),
        damage_incoming: bool_fx(
            damage_state.recent_timer > Fx::from_num(0)
                || damage_detector.is_some_and(|detector| detector.detected),
        ),
        damage_critical: bool_fx(damage_detector.is_some_and(|detector| detector.critical)),
        damage_dir_x: detector_axis(damage_detector, |detector| detector.direction.x),
        damage_dir_y: detector_axis(damage_detector, |detector| detector.direction.y),
        damage_intensity: detector_metric(damage_detector, |detector| detector.magnitude)
            .max(damage_state.recent_intensity),
        power_deficit: bool_fx(power_detector.is_some_and(|detector| detector.detected)),
        power_low_battery: bool_fx(
            power_detector.is_some_and(|detector| detector.secondary_detected),
        ),
        heat_alert: bool_fx(heat_detector.is_some_and(|detector| detector.detected)),
        heat_dir_x: detector_axis(heat_detector, |detector| detector.direction.x),
        heat_dir_y: detector_axis(heat_detector, |detector| detector.direction.y),
        heat_severity: detector_metric(heat_detector, |detector| detector.magnitude),
        logistics_demand: bool_fx(logistics_detector.is_some_and(|detector| detector.detected)),
        logistics_dir_x: detector_axis(logistics_detector, |detector| detector.direction.x),
        logistics_dir_y: detector_axis(logistics_detector, |detector| detector.direction.y),
        logistics_severity: detector_metric(logistics_detector, |detector| detector.magnitude),
    }
}
