mod atmosphere;
mod collision_math;
mod combat;
pub(crate) mod control;
pub(crate) mod editor;
mod fields;
mod interactions;
mod math;
mod ship;
pub(crate) mod simulation;
mod status;

pub(crate) use atmosphere::{
    breach_leak_multiplier,
    decompression_signature,
    recompute_decompression_vectors,
    sampled_decompression_pull,
};
pub(crate) use collision_math::{
    clamp_non_negative,
    collision_damage_from_energy,
    collision_energy_wide,
    narrow_wide_clamped,
    safe_sqrt_wide,
    shield_accepts_contact,
};
pub(crate) use combat::{
    angle_from_vector,
    clamp_position_to_arena,
    damp_scalar,
    damp_vec2,
    facing_vector,
    fixed_radius_sq,
    is_inside_arena,
    render_translation,
    spawn_projectile_entity,
};
pub(crate) use editor::*;
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
    module_can_be_extracted,
    module_needs_repair,
    resource_kind_label,
    sprite_path_for_kind,
};
pub(crate) use math::{
    FieldOutput,
    FixedVec2,
    Fx,
    WideFx,
    fixed_square,
    format_fx0,
    format_fx1,
    format_fx2,
    fx_from_time_delta,
    fx_ratio,
    widen,
    wrap_angle_f32,
    wrap_radians,
};
#[allow(unused_imports)]
pub(crate) use ship::ship_grid_facing_offset;
#[allow(unused_imports)]
pub(crate) use ship::ship_grid_from_local_position_with_origin;
pub(crate) use ship::{
    cardinal_neighbors,
    component_service_coords,
    focused_ship_grid_tile,
    focused_ship_grid_tile_with_origin,
    module_integrity,
    module_local_position,
    ship_grid_from_local_position,
    ship_movement_model_with_effective,
    ship_power_model_with_effective,
    ship_tile_contains_point,
    ship_tile_overlaps_point,
    sprite_path_for_foundation,
    sprite_path_for_foundation_connections,
};
pub(crate) use simulation::{
    absorb_hostile_shield_hit,
    absorb_player_shield_hit,
    consume_ship_resource,
    spawn_hostile_salvage,
};
pub(crate) use status::{
    condition_severity,
    danger_level,
    gameplay_status_line,
    interaction_label,
    meter_bar,
    mission_return_line,
    mission_status_line,
    module_display_name,
};
