mod combat;
mod fields;
mod interactions;
mod math;
mod ship;
mod status;

pub(crate) use combat::{
    angle_from_vector,
    clamp_position_to_arena,
    damp_scalar,
    damp_vec2,
    facing_vector,
    fixed_radius_sq,
    is_inside_arena,
    render_translation,
    spawn_player_projectile,
    spawn_projectile_entity,
};
pub(crate) use fields::{
    dynamic_field_output,
    field_attenuation,
    local_field_distance,
    module_condition,
    module_condition_label,
    module_effectiveness,
};
pub(crate) use interactions::{
    interaction_for_module,
    interaction_hold_duration,
    interaction_prompt,
    is_hold_interaction,
    resource_kind_label,
    sprite_path_for_kind,
};
pub(crate) use math::{
    FieldOutput,
    FixedVec2,
    Fx,
    WideFx,
    format_fx0,
    format_fx1,
    format_fx2,
    fx_from_time_delta,
    fx_ratio,
    widen,
    wrap_radians,
};
pub(crate) use ship::{
    count_modules,
    module_integrity,
    module_local_position,
    module_local_translation,
    ship_movement_model,
    ship_movement_model_with_effective,
    ship_power_model,
    ship_power_model_with_effective,
    update_ship_power_state,
};
pub(crate) use status::{
    danger_level,
    gameplay_status_line,
    meter_bar,
    mission_return_line,
    mission_status_line,
};
