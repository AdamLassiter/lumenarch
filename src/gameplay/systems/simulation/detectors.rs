use bevy::prelude::*;

use crate::gameplay::{
    components::{
        DestroyedModule,
        DetectorKind,
        DetectorModule,
        HostileShip,
        Integrity,
        ModuleRuntimeState,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerShip,
        ProcessorModule,
        RuntimeShipModule,
        ShipDamageSensorState,
        ShipPowerState,
        ShipRoot,
        SimPosition,
        SimRotation,
        StorageModule,
    },
    helpers::{FixedVec2, Fx, fx_from_time_delta},
};

/// Updates detector and monitor modules so ARCH can read live ship, crew, and threat cues.
pub(crate) fn update_detector_modules(
    time: Res<Time>,
    ship_query: Single<
        (
            &SimPosition,
            &SimRotation,
            &ShipPowerState,
            &mut ShipDamageSensorState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    hostile_root_query: Query<Entity, (With<HostileShip>, With<ShipRoot>)>,
    other_ship_query: Query<
        (Entity, &SimPosition, Option<&HostileShip>),
        (With<ShipRoot>, Without<PlayerShip>),
    >,
    player_query: Query<&PlayerMotionState>,
    mut detector_query: Query<
        (
            &RuntimeShipModule,
            &mut DetectorModule,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    module_query: Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&StorageModule>,
            Option<&ProcessorModule>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) {
    let dt = fx_from_time_delta(&time);
    let (ship_position, ship_rotation, ship_power_state, mut damage_state) =
        ship_query.into_inner();

    damage_state.recent_timer = (damage_state.recent_timer - dt).max(Fx::from_num(0));
    if damage_state.recent_timer == Fx::from_num(0) {
        damage_state.recent_direction = FixedVec2::zero();
        damage_state.recent_distance = Fx::from_num(0);
        damage_state.recent_intensity = Fx::from_num(0);
    }

    for (runtime_module, mut detector, destroyed) in &mut detector_query {
        if destroyed.is_some() {
            detector.detected = false;
            detector.secondary_detected = false;
            detector.direction = FixedVec2::zero();
            detector.distance = Fx::from_num(0);
            detector.magnitude = Fx::from_num(0);
            detector.critical = false;
            continue;
        }

        let detector_world =
            ship_position.value + runtime_module.local_position.rotate(ship_rotation.radians);
        let mut direction = FixedVec2::zero();
        let mut distance = Fx::from_num(0);
        let mut magnitude = Fx::from_num(0);
        let mut detected = false;
        let mut secondary_detected = false;
        let mut critical = false;

        match detector.kind {
            DetectorKind::LifeSign => {
                let mut best_distance = detector.range.max(Fx::from_num(1)) + Fx::from_num(1);
                for motion in &player_query {
                    let delta = motion.world_position - detector_world;
                    let candidate_distance = delta.length();
                    if candidate_distance > detector.range {
                        continue;
                    }

                    let hostile = match motion.frame {
                        PlayerReferenceFrame::Ship(entity) => hostile_root_query.contains(entity),
                        PlayerReferenceFrame::World => false,
                    };
                    if hostile {
                        secondary_detected = true;
                    } else {
                        detected = true;
                    }

                    if candidate_distance < best_distance {
                        best_distance = candidate_distance;
                        let local_target = (motion.world_position - ship_position.value)
                            .rotate(-ship_rotation.radians);
                        let delta_local = local_target - runtime_module.local_position;
                        direction = delta_local.normalized_or_zero();
                        distance = delta_local.length();
                    }
                }
            }
            DetectorKind::Ship => {
                let mut best_distance = detector.range.max(Fx::from_num(1)) + Fx::from_num(1);
                for (_entity, other_position, hostile) in &other_ship_query {
                    let delta = other_position.value - detector_world;
                    let candidate_distance = delta.length();
                    if candidate_distance > detector.range {
                        continue;
                    }
                    detected = true;
                    secondary_detected |= hostile.is_some();
                    if candidate_distance < best_distance {
                        best_distance = candidate_distance;
                        let local_target = (other_position.value - ship_position.value)
                            .rotate(-ship_rotation.radians);
                        let delta_local = local_target - runtime_module.local_position;
                        direction = delta_local.normalized_or_zero();
                        distance = delta_local.length();
                    }
                }
            }
            DetectorKind::Damage => {
                detected = damage_state.recent_timer > Fx::from_num(0);
                magnitude = damage_state.recent_intensity;
                direction = damage_state.recent_direction;
                distance = damage_state.recent_distance;

                let mut worst_score = Fx::from_num(0);
                for (module, integrity, runtime_state, _, _, module_destroyed) in &module_query {
                    if module_destroyed.is_some() {
                        critical = true;
                    }
                    let damage_fraction = if integrity.max > 0 {
                        Fx::from_num((integrity.max - integrity.current).max(0))
                            / Fx::from_num(integrity.max)
                    } else {
                        Fx::from_num(0)
                    };
                    let severity = damage_fraction
                        + runtime_state.current_heat * Fx::from_num(0.03)
                        + runtime_state.electrical_instability * Fx::from_num(0.03);
                    if runtime_state.is_disabled || integrity.current <= (integrity.max / 3).max(1)
                    {
                        critical = true;
                    }
                    if severity > worst_score {
                        worst_score = severity;
                        if !detected && detector.tier >= 2 {
                            let delta_local = module.local_position - runtime_module.local_position;
                            direction = delta_local.normalized_or_zero();
                        }
                        if detector.tier >= 3 {
                            distance =
                                (module.local_position - runtime_module.local_position).length();
                        }
                        magnitude = magnitude.max(severity * Fx::from_num(10));
                    }
                }
            }
            DetectorKind::Power => {
                detected = ship_power_state.surplus < Fx::from_num(0);
                secondary_detected = ship_power_state.stored_energy <= Fx::from_num(2);
                magnitude = ship_power_state.stored_energy;
                critical = secondary_detected && detected;
            }
            DetectorKind::Heat => {
                let mut hottest = Fx::from_num(0);
                for (module, _, runtime_state, _, _, destroyed) in &module_query {
                    if destroyed.is_some() {
                        continue;
                    }
                    let severity = runtime_state
                        .current_heat
                        .max(runtime_state.electrical_instability * Fx::from_num(1.2));
                    if severity > hottest {
                        hottest = severity;
                        let delta_local = module.local_position - runtime_module.local_position;
                        direction = delta_local.normalized_or_zero();
                        distance = delta_local.length();
                    }
                    if runtime_state.is_disabled || runtime_state.needs_attention {
                        critical = true;
                    }
                }
                detected = hottest >= Fx::from_num(8);
                magnitude = hottest;
            }
            DetectorKind::Logistics => {
                let mut strongest = Fx::from_num(0);
                for (module, _, _, storage, processor, destroyed) in &module_query {
                    if destroyed.is_some() {
                        continue;
                    }
                    let mut severity = Fx::from_num(0);
                    if let Some(storage) = storage
                        && storage.inventory.total_units() >= storage.capacity
                    {
                        severity = severity.max(Fx::from_num(1));
                        secondary_detected = true;
                    }
                    if let Some(processor) = processor {
                        if processor.blocked_reason.is_some() {
                            severity = severity.max(Fx::from_num(2));
                            critical = true;
                        } else if processor.inventory.raw_salvage < processor.input_required {
                            severity = severity.max(Fx::from_num(1));
                        }
                    }
                    if severity > strongest {
                        strongest = severity;
                        let delta_local = module.local_position - runtime_module.local_position;
                        direction = delta_local.normalized_or_zero();
                        distance = delta_local.length();
                    }
                }
                detected = strongest > Fx::from_num(0);
                magnitude = strongest;
            }
        }

        if detector.tier < 2
            && !matches!(
                detector.kind,
                DetectorKind::Power | DetectorKind::Heat | DetectorKind::Logistics
            )
        {
            direction = FixedVec2::zero();
        }
        if detector.tier < 3 && !matches!(detector.kind, DetectorKind::Power) {
            distance = Fx::from_num(0);
        }

        detector.detected = detected;
        detector.secondary_detected = secondary_detected;
        detector.direction = direction;
        detector.distance = distance;
        detector.magnitude = magnitude.max(Fx::from_num(0));
        detector.critical = critical;
    }
}
