use bevy::prelude::*;

use crate::{
    client::gameplay::{
        components::{
            CurrentStation,
            DestroyedModule,
            Integrity,
            MissionState,
            ModuleFieldEmitter,
            ModuleRuntimeState,
            PlayerFieldState,
            PlayerShip,
            ReactorCommandState,
            RuntimeShipModule,
            ShipRoot,
            ShipboardPlayer,
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
    ship::ModuleKind,
};

pub(crate) fn sample_ship_fields(
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    mission_query: Single<&MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    player_query: Single<(&CurrentStation, &mut PlayerFieldState), With<ShipboardPlayer>>,
) {
    let mission_state = mission_query.into_inner();
    let module_samples: Vec<_> = module_query
        .iter()
        .map(
            |(entity, runtime_module, emitter, integrity, runtime_state, destroyed)| {
                let output =
                    dynamic_field_output(emitter, runtime_state, integrity, destroyed.is_some());
                (
                    entity,
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

    for (entity, runtime_module, _, _, mut runtime_state, destroyed) in &mut module_query {
        if destroyed.is_some() {
            runtime_state.sampled_heat = Fx::from_num(0);
            runtime_state.sampled_electrical = Fx::from_num(0);
            continue;
        }

        let mut heat = Fx::from_num(0);
        let mut electrical = Fx::from_num(0);
        for (
            source_entity,
            source_pos,
            source_heat,
            source_cooling,
            source_electrical,
            source_grounding,
            source_destroyed,
        ) in &module_samples
        {
            if *source_destroyed || *source_entity == entity {
                continue;
            }
            let attenuation = field_attenuation(local_field_distance(
                runtime_module.local_position,
                *source_pos,
            ));
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

    let (station, mut player_fields) = player_query.into_inner();
    let Some(player_pos) = module_query
        .iter()
        .find(|(_, runtime_module, _, _, _, _)| runtime_module.module_id == station.module_id)
        .map(|(_, runtime_module, _, _, _, _)| runtime_module.local_position)
    else {
        return;
    };

    let mut heat = Fx::from_num(0);
    let mut electrical = Fx::from_num(0);
    for (
        _,
        source_pos,
        source_heat,
        source_cooling,
        source_electrical,
        source_grounding,
        source_destroyed,
    ) in &module_samples
    {
        if *source_destroyed {
            continue;
        }
        let attenuation = field_attenuation(local_field_distance(player_pos, *source_pos));
        if attenuation <= Fx::from_num(0) {
            continue;
        }
        heat += (*source_heat - *source_cooling) * attenuation;
        electrical += (*source_electrical - *source_grounding) * attenuation;
    }

    player_fields.local_heat = (heat + mission_state.ambient_heat_pressure).max(Fx::from_num(0));
    player_fields.local_electrical =
        (electrical + mission_state.ambient_electrical_pressure).max(Fx::from_num(0));
    player_fields.heat_danger = player_fields.local_heat >= Fx::from_num(8);
    player_fields.electrical_danger = player_fields.local_electrical >= Fx::from_num(7);
}

pub(crate) fn update_module_runtime_state(
    time: Res<Time>,
    mut module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&mut ReactorCommandState>,
        Option<&mut TurretCommandState>,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);

    for (runtime_module, integrity, mut runtime_state, reactor_state, turret_state, destroyed) in
        &mut module_query
    {
        runtime_state.was_disabled_last_frame = runtime_state.is_disabled;
        if destroyed.is_some() || integrity.current <= 0 {
            runtime_state.is_disabled = true;
            continue;
        }

        let base_heat = match runtime_module.kind {
            ModuleKind::Reactor => Fx::from_num(0.9),
            ModuleKind::Engine => Fx::from_num(0.45),
            ModuleKind::Turret => Fx::from_num(0.3),
            ModuleKind::Battery => Fx::from_num(0.2),
            ModuleKind::Computer => Fx::from_num(0.15),
            _ => Fx::from_num(0.05),
        };
        let damage_factor = Fx::from_num(1)
            - Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
        runtime_state.last_interaction_age += dt;
        let mut reactor_heat_bonus = Fx::from_num(0);
        if let Some(mut reactor_state) = reactor_state {
            let warmup_threshold = Fx::from_num(3.0);
            let full_power_threshold = Fx::from_num(6.0);
            if reactor_state.fuel_remaining > Fx::from_num(0) {
                reactor_state.fuel_remaining = (reactor_state.fuel_remaining
                    - reactor_state.reaction_rate * dt * Fx::from_num(0.22))
                .max(Fx::from_num(0));
            } else {
                reactor_state.reaction_rate = Fx::from_num(0);
            }
            if runtime_state.current_heat < warmup_threshold {
                reactor_state.reaction_rate =
                    (reactor_state.reaction_rate - dt * Fx::from_num(0.35)).max(Fx::from_num(0));
                reactor_state.power_output = Fx::from_num(0);
            } else {
                let heat_span = (full_power_threshold - warmup_threshold).max(Fx::from_num(1));
                let warmup_ratio = ((runtime_state.current_heat - warmup_threshold) / heat_span)
                    .clamp(Fx::from_num(0), Fx::from_num(1));
                reactor_state.power_output = ((reactor_state.reaction_rate * Fx::from_num(3)
                    + reactor_state.turbine_load * Fx::from_num(9))
                .min(Fx::from_num(12)))
                    * warmup_ratio;
            }
            reactor_heat_bonus = reactor_state.reaction_rate * Fx::from_num(4.8)
                - reactor_state.turbine_load * Fx::from_num(2.2);
        }
        if let Some(mut turret_state) = turret_state {
            let angle_delta = wrap_radians(turret_state.desired_angle - turret_state.actual_angle);
            let step = Fx::from_num(2.6) * dt;
            turret_state.actual_angle = if angle_delta > step {
                wrap_radians(turret_state.actual_angle + step)
            } else if angle_delta < -step {
                wrap_radians(turret_state.actual_angle - step)
            } else {
                wrap_radians(turret_state.desired_angle)
            };
        }
        let target_heat = base_heat
            + reactor_heat_bonus.max(Fx::from_num(-1.5))
            + runtime_state.sampled_heat * Fx::from_num(0.45)
            + damage_factor;
        runtime_state.current_heat = (runtime_state.current_heat
            + (target_heat - runtime_state.current_heat) * Fx::from_num(1.35) * dt)
            .max(Fx::from_num(0));
        runtime_state.electrical_instability = (runtime_state.electrical_instability
            + (runtime_state.sampled_electrical * Fx::from_num(0.08)
                + damage_factor * Fx::from_num(0.45))
                * dt
            - Fx::from_num(0.5) * dt)
            .max(Fx::from_num(0));

        runtime_state.needs_attention = runtime_state.current_heat >= Fx::from_num(9)
            || runtime_state.electrical_instability >= Fx::from_num(8)
            || integrity.current < integrity.max;
        runtime_state.is_disabled = runtime_state.current_heat >= Fx::from_num(16)
            || runtime_state.electrical_instability >= Fx::from_num(14);
    }
}
