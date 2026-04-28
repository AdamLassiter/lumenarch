use super::{FieldOutput, FixedVec2, Fx, fx_ratio};
use crate::client::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::components::{Integrity, ModuleCondition, ModuleFieldEmitter, ModuleRuntimeState},
};

pub(crate) fn module_effectiveness(
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
) -> Fx {
    if destroyed || integrity.current <= 0 || runtime_state.is_disabled {
        return Fx::from_num(0);
    }

    let mut effectiveness =
        Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
    if runtime_state.needs_attention {
        effectiveness *= fx_ratio(3, 4);
    }
    effectiveness -= runtime_state.current_heat * fx_ratio(1, 48);
    effectiveness -= runtime_state.electrical_instability * fx_ratio(1, 40);
    effectiveness.clamp(Fx::from_num(0), Fx::from_num(1))
}

pub(crate) fn module_condition(
    integrity: &Integrity,
    runtime_state: &ModuleRuntimeState,
    destroyed: bool,
    balance: &BalanceConfig,
) -> ModuleCondition {
    if destroyed || integrity.current <= 0 {
        ModuleCondition::Destroyed
    } else if runtime_state.is_disabled {
        ModuleCondition::Disabled
    } else if runtime_state.needs_attention
        || runtime_state.current_heat >= Fx::from_num(balance.fields.degraded_heat_threshold)
        || runtime_state.electrical_instability
            >= Fx::from_num(balance.fields.degraded_electrical_threshold)
        || integrity.current * 2 <= integrity.max
    {
        ModuleCondition::Degraded
    } else {
        ModuleCondition::Healthy
    }
}

pub(crate) fn module_condition_label(condition: ModuleCondition) -> &'static str {
    match condition {
        ModuleCondition::Healthy => "healthy",
        ModuleCondition::Degraded => "degraded",
        ModuleCondition::Disabled => "disabled",
        ModuleCondition::Destroyed => "destroyed",
    }
}

pub(crate) fn dynamic_field_output(
    emitter: &ModuleFieldEmitter,
    runtime_state: &ModuleRuntimeState,
    integrity: &Integrity,
    destroyed: bool,
    balance: &BalanceConfig,
) -> FieldOutput {
    if destroyed || integrity.current <= 0 {
        return FieldOutput::default();
    }

    let damage_factor = Fx::from_num(1)
        - Fx::from_num(integrity.current.max(0)) / Fx::from_num(integrity.max.max(1));
    let heat_attention_bonus = if runtime_state.needs_attention {
        Fx::from_num(balance.fields.attention_heat_multiplier)
    } else {
        Fx::from_num(1)
    };
    let grounding_penalty = if runtime_state.needs_attention {
        Fx::from_num(balance.fields.attention_grounding_penalty)
    } else {
        Fx::from_num(0)
    };
    let heat = emitter.heat_output * heat_attention_bonus
        + damage_factor * Fx::from_num(balance.fields.damage_heat_bonus);
    let cooling = emitter.cooling_output;
    let electrical = emitter.electrical_output
        + damage_factor * Fx::from_num(balance.fields.damage_electrical_bonus)
        + runtime_state.electrical_instability
            * Fx::from_num(balance.fields.instability_leak_factor);
    let grounding = (emitter.grounding_output
        - damage_factor * Fx::from_num(balance.fields.damage_grounding_loss)
        - grounding_penalty)
        .max(Fx::from_num(0));

    FieldOutput {
        heat,
        cooling,
        electrical,
        grounding,
    }
}

pub(crate) fn local_field_distance(a: FixedVec2, b: FixedVec2) -> Fx {
    (a - b).length()
}

pub(crate) fn field_attenuation(distance: Fx, balance: &BalanceConfig) -> Fx {
    let radius = Fx::from_num(TILE_SIZE * balance.fields.attenuation_radius_tiles);
    if distance >= radius {
        Fx::from_num(0)
    } else {
        Fx::from_num(1) - distance / radius
    }
}
