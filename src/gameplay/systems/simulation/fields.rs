use bevy::prelude::*;

use crate::{
    {
        balance::BalanceConfig,
        gameplay::{
            components::{
                DestroyedModule,
                Integrity,
                MissionState,
                ModuleFieldEmitter,
                ModuleRuntimeState,
                PlayerFieldState,
                PlayerMotionState,
                PlayerReferenceFrame,
                PlayerShip,
                ReactorCommandState,
                ResourceKind,
                RuntimeShipModule,
                ShieldCommandState,
                ShipRoot,
                ShipboardPlayer,
                StorageModule,
                TurretCommandState,
            },
            helpers::{
                Fx,
                dynamic_field_output,
                field_attenuation,
                fx_from_time_delta,
                local_field_distance,
                wrap_radians,
            },
        },
    },
    ship::ModuleKind,
};

pub(crate) fn sample_ship_fields(
    balance: Res<BalanceConfig>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &Parent,
        &ModuleFieldEmitter,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<(&PlayerMotionState, &mut PlayerFieldState), With<ShipboardPlayer>>,
) {
    let mission_state = mission_query.into_inner();
    let module_samples: Vec<_> = module_query
        .iter()
        .map(
            |(entity, runtime_module, parent, emitter, integrity, runtime_state, destroyed)| {
                let output = dynamic_field_output(
                    emitter,
                    runtime_state,
                    integrity,
                    destroyed.is_some(),
                    &balance,
                );
                (
                    entity,
                    parent.get(),
                    runtime_module.local_position,
                    output.heat,
                    output.cooling,
                    output.electrical,
                    output.grounding,
                    destroyed.is_some(),
                )
            },
        )
        .collect();

    for (entity, runtime_module, parent, _, _, mut runtime_state, destroyed) in &mut module_query {
        if destroyed.is_some() {
            runtime_state.sampled_heat = Fx::from_num(0);
            runtime_state.sampled_electrical = Fx::from_num(0);
            continue;
        }

        let mut heat = Fx::from_num(0);
        let mut electrical = Fx::from_num(0);
        for (
            source_entity,
            source_parent,
            source_pos,
            source_heat,
            source_cooling,
            source_electrical,
            source_grounding,
            source_destroyed,
        ) in &module_samples
        {
            if *source_destroyed || *source_entity == entity || *source_parent != parent.get() {
                continue;
            }
            let attenuation = field_attenuation(
                local_field_distance(runtime_module.local_position, *source_pos),
                &balance,
            );
            if attenuation <= Fx::from_num(0) {
                continue;
            }
            heat += (*source_heat - *source_cooling) * attenuation;
            electrical += (*source_electrical - *source_grounding) * attenuation;
        }
        runtime_state.sampled_heat =
            (heat + mission_state.ambient_heat_pressure).max(Fx::from_num(0));
        runtime_state.sampled_electrical =
            (electrical + mission_state.ambient_electrical_pressure).max(Fx::from_num(0));
    }

    let (player_motion, mut player_fields) = player_query.into_inner();
    let Some(active_ship) = (match player_motion.frame {
        PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
        PlayerReferenceFrame::World => None,
    }) else {
        player_fields.local_heat = Fx::from_num(0);
        player_fields.local_electrical = Fx::from_num(0);
        player_fields.heat_danger = false;
        player_fields.electrical_danger = false;
        return;
    };
    let player_pos = player_motion.local_position;

    let mut heat = Fx::from_num(0);
    let mut electrical = Fx::from_num(0);
    for (
        _,
        source_parent,
        source_pos,
        source_heat,
        source_cooling,
        source_electrical,
        source_grounding,
        source_destroyed,
    ) in &module_samples
    {
        if *source_destroyed || *source_parent != active_ship {
            continue;
        }
        let attenuation =
            field_attenuation(local_field_distance(player_pos, *source_pos), &balance);
        if attenuation <= Fx::from_num(0) {
            continue;
        }
        heat += (*source_heat - *source_cooling) * attenuation;
        electrical += (*source_electrical - *source_grounding) * attenuation;
    }

    player_fields.local_heat = (heat + mission_state.ambient_heat_pressure).max(Fx::from_num(0));
    player_fields.local_electrical =
        (electrical + mission_state.ambient_electrical_pressure).max(Fx::from_num(0));
    player_fields.heat_danger =
        player_fields.local_heat >= Fx::from_num(balance.fields.player_heat_warning_threshold);
    player_fields.electrical_danger = player_fields.local_electrical
        >= Fx::from_num(balance.fields.player_electrical_warning_threshold);
}

pub(crate) fn update_module_runtime_state(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mut module_query: Query<(
        &RuntimeShipModule,
        &Parent,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&mut ReactorCommandState>,
        Option<&mut ShieldCommandState>,
        Option<&mut TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
    mut storage_query: Query<(&RuntimeShipModule, &Parent, &mut StorageModule)>,
) {
    let dt = fx_from_time_delta(&time);

    for (
        runtime_module,
        parent,
        integrity,
        mut runtime_state,
        reactor_state,
        shield_state,
        turret_state,
        destroyed,
    ) in &mut module_query
    {
        runtime_state.was_disabled_last_frame = runtime_state.is_disabled;
        if destroyed.is_some() || integrity.current <= 0 {
            runtime_state.is_disabled = true;
            continue;
        }

        let base_heat = match runtime_module.kind {
            ModuleKind::Reactor => Fx::from_num(balance.fields.reactor_base_heat),
            ModuleKind::Engine => Fx::from_num(balance.fields.engine_base_heat),
            ModuleKind::Turret => Fx::from_num(balance.fields.turret_base_heat),
            ModuleKind::Battery => Fx::from_num(balance.fields.battery_base_heat),
            ModuleKind::Computer => Fx::from_num(balance.fields.computer_base_heat),
            _ => Fx::from_num(balance.fields.generic_base_heat),
        };
        let damage_factor = Fx::from_num(1)
            - Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
        runtime_state.last_interaction_age += dt;
        let mut reactor_heat_bonus = Fx::from_num(0);
        if let Some(mut reactor_state) = reactor_state {
            if reactor_state.fuel_remaining < Fx::from_num(balance.reactor.starting_fuel * 0.25) {
                for (storage_runtime, storage_parent, mut storage) in &mut storage_query {
                    if storage_parent.get() != parent.get()
                        || !storage.accepts_fuel
                        || local_field_distance(
                            runtime_module.local_position,
                            storage_runtime.local_position,
                        ) > Fx::from_num(64)
                    {
                        continue;
                    }
                    if storage.inventory.remove(ResourceKind::Fuel, 1) > 0 {
                        reactor_state.fuel_remaining +=
                            Fx::from_num(balance.reactor.starting_fuel * 0.5);
                        break;
                    }
                }
            }
            let warmup_threshold = Fx::from_num(balance.reactor.warmup_threshold);
            let full_power_threshold = Fx::from_num(balance.reactor.full_power_threshold);
            if reactor_state.fuel_remaining > Fx::from_num(0) {
                let variant_fuel_scalar =
                    if reactor_state.variant == crate::ship::ModuleVariant::Fusion {
                        Fx::from_num(1.25)
                    } else {
                        Fx::from_num(1)
                    };
                reactor_state.fuel_remaining = (reactor_state.fuel_remaining
                    - reactor_state.reaction_rate
                        * dt
                        * Fx::from_num(balance.reactor.fuel_burn_rate)
                        * variant_fuel_scalar)
                    .max(Fx::from_num(0));
            } else {
                reactor_state.reaction_rate = Fx::from_num(0);
            }
            let heat_span = (full_power_threshold - warmup_threshold).max(Fx::from_num(1));
            let warmup_ratio = if runtime_state.current_heat <= Fx::from_num(0) {
                Fx::from_num(0)
            } else if runtime_state.current_heat < warmup_threshold {
                (runtime_state.current_heat / warmup_threshold)
                    .clamp(Fx::from_num(0), Fx::from_num(1))
                    * Fx::from_num(0.5)
            } else {
                (Fx::from_num(0.5)
                    + ((runtime_state.current_heat - warmup_threshold) / heat_span)
                        .clamp(Fx::from_num(0), Fx::from_num(1))
                        * Fx::from_num(0.5))
                .clamp(Fx::from_num(0), Fx::from_num(1))
            };
            let mut power_output = (reactor_state.reaction_rate
                * Fx::from_num(balance.reactor.reaction_power_factor)
                + reactor_state.turbine_load * Fx::from_num(balance.reactor.turbine_power_factor))
            .min(Fx::from_num(balance.reactor.max_power_output));
            if reactor_state.variant == crate::ship::ModuleVariant::Fusion {
                let confinement = reactor_state
                    .confinement
                    .clamp(Fx::from_num(0), Fx::from_num(1));
                let stability = Fx::from_num(1)
                    - (reactor_state.reaction_rate - confinement)
                        .abs()
                        .min(Fx::from_num(1));
                power_output *= Fx::from_num(0.7) + stability * Fx::from_num(0.9);
                reactor_heat_bonus = reactor_state.reaction_rate
                    * Fx::from_num(balance.reactor.reaction_heat_factor * 1.25)
                    + (Fx::from_num(1) - stability) * Fx::from_num(1.5)
                    - reactor_state.turbine_load
                        * Fx::from_num(balance.reactor.turbine_cooling_factor * 0.85);
            } else {
                reactor_heat_bonus = reactor_state.reaction_rate
                    * Fx::from_num(balance.reactor.reaction_heat_factor)
                    - reactor_state.turbine_load
                        * Fx::from_num(balance.reactor.turbine_cooling_factor);
            }
            reactor_state.power_output = power_output * warmup_ratio;
        }
        if let Some(mut turret_state) = turret_state {
            let angle_delta = wrap_radians(turret_state.desired_angle - turret_state.actual_angle);
            let step = Fx::from_num(balance.combat.turret_rotation_speed) * dt;
            turret_state.actual_angle = if angle_delta > step {
                wrap_radians(turret_state.actual_angle + step)
            } else if angle_delta < -step {
                wrap_radians(turret_state.actual_angle - step)
            } else {
                wrap_radians(turret_state.desired_angle)
            };
        }
        if let Some(mut shield_state) = shield_state {
            let regen_factor = if runtime_state.current_heat
                >= Fx::from_num(balance.fields.degraded_heat_threshold)
            {
                Fx::from_num(0.4)
            } else {
                Fx::from_num(1)
            };
            shield_state.strength = (shield_state.strength
                + shield_state.regen_rate * regen_factor * dt)
                .min(shield_state.max_strength);
        }
        let target_heat = base_heat
            + reactor_heat_bonus.max(Fx::from_num(balance.fields.reactor_heat_min_bonus))
            + runtime_state.sampled_heat * Fx::from_num(balance.fields.sampled_heat_factor)
            + damage_factor;
        runtime_state.current_heat = (runtime_state.current_heat
            + (target_heat - runtime_state.current_heat)
                * Fx::from_num(balance.fields.heat_response_rate)
                * dt)
            .max(Fx::from_num(0));
        runtime_state.electrical_instability = (runtime_state.electrical_instability
            + (runtime_state.sampled_electrical
                * Fx::from_num(balance.fields.sampled_electrical_factor)
                + damage_factor * Fx::from_num(balance.fields.damage_instability_factor))
                * dt
            - Fx::from_num(balance.fields.electrical_decay_rate) * dt)
            .max(Fx::from_num(0));

        runtime_state.needs_attention = runtime_state.current_heat
            >= Fx::from_num(balance.fields.degraded_heat_threshold)
            || runtime_state.electrical_instability
                >= Fx::from_num(balance.fields.degraded_electrical_threshold)
            || integrity.current < integrity.max;
        runtime_state.is_disabled = runtime_state.current_heat
            >= Fx::from_num(balance.fields.disabled_heat_threshold)
            || runtime_state.electrical_instability
                >= Fx::from_num(balance.fields.disabled_electrical_threshold);
    }
}
