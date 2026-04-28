use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        CurrentStation,
        DestroyedModule,
        Integrity,
        ModuleFieldEmitter,
        ModuleRuntimeState,
        PlayerFieldState,
        RuntimeShipModule,
        ShipboardPlayer,
    },
    helpers::{
        dynamic_field_output,
        field_attenuation,
        local_field_distance,
        fx_from_time_delta,
        Fx,
    },
};
use crate::ship::ModuleKind;

pub(crate) fn sample_ship_fields(
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &ModuleFieldEmitter,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
    player_query: Single<(&CurrentStation, &mut PlayerFieldState), With<ShipboardPlayer>>,
) {
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
        runtime_state.sampled_heat = heat.max(Fx::from_num(0));
        runtime_state.sampled_electrical = electrical.max(Fx::from_num(0));
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

    player_fields.local_heat = heat.max(Fx::from_num(0));
    player_fields.local_electrical = electrical.max(Fx::from_num(0));
    player_fields.heat_danger = player_fields.local_heat >= Fx::from_num(8);
    player_fields.electrical_danger = player_fields.local_electrical >= Fx::from_num(7);
}

pub(crate) fn update_module_runtime_state(
    time: Res<Time>,
    mut module_query: Query<(
        &RuntimeShipModule,
        &Integrity,
        &mut ModuleRuntimeState,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);

    for (runtime_module, integrity, mut runtime_state, destroyed) in &mut module_query {
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
        let target_heat =
            base_heat + runtime_state.sampled_heat * Fx::from_num(0.45) + damage_factor;
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
