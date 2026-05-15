use super::{FixedVec2, Fx, WideFx, widen, wrap_radians};

pub(crate) const FX_QUARTER_MAX_BITS: i32 = i32::MAX / 4;

pub(crate) fn clamp_non_negative(value: Fx, max_value: Fx) -> Fx {
    debug_assert!(max_value >= Fx::from_num(0));
    value.clamp(Fx::from_num(0), max_value)
}

pub(crate) fn reduced_mass_wide(mass_a: Fx, mass_b: Fx, max_effective_mass: Fx) -> WideFx {
    let mass_a = widen(clamp_non_negative(
        mass_a.max(Fx::from_num(1)),
        max_effective_mass,
    ));
    let mass_b = widen(clamp_non_negative(
        mass_b.max(Fx::from_num(1)),
        max_effective_mass,
    ));
    debug_assert!(mass_a > WideFx::from_num(0));
    debug_assert!(mass_b > WideFx::from_num(0));
    (mass_a * mass_b) / (mass_a + mass_b).max(WideFx::from_num(1))
}

pub(crate) fn collision_energy_wide(
    mass_a: Fx,
    mass_b: Fx,
    closing_speed: Fx,
    max_effective_mass: Fx,
    max_effective_speed: Fx,
) -> WideFx {
    let reduced_mass = reduced_mass_wide(mass_a, mass_b, max_effective_mass);
    let clamped_speed = widen(clamp_non_negative(closing_speed, max_effective_speed));
    debug_assert!(clamped_speed >= WideFx::from_num(0));
    (reduced_mass * clamped_speed * clamped_speed) / WideFx::from_num(2)
}

pub(crate) fn collision_damage_from_energy(
    collision_energy: WideFx,
    damage_threshold: WideFx,
    damage_divisor: WideFx,
) -> i32 {
    let energy = collision_energy.max(WideFx::from_num(0));
    if energy < damage_threshold {
        return 0;
    }
    let steps = energy / damage_divisor.max(WideFx::from_num(1));
    let whole = steps.to_num::<i32>();
    if steps > WideFx::from_num(whole) {
        (whole + 1).max(1)
    } else {
        whole.max(1)
    }
}

pub(crate) fn narrow_wide_clamped(value: WideFx) -> Fx {
    let max = Fx::from_bits(FX_QUARTER_MAX_BITS);
    let widened_max = widen(max);
    let clamped = value.clamp(-widened_max, widened_max);
    debug_assert!(clamped >= -widened_max && clamped <= widened_max);
    Fx::from_bits(clamped.to_bits() as i32)
}

pub(crate) fn half_arc_radians_from_degrees(arc_degrees: Fx) -> Fx {
    debug_assert!(arc_degrees >= Fx::from_num(0));
    (arc_degrees * Fx::PI) / Fx::from_num(360)
}

pub(crate) fn shield_accepts_contact(
    ship_rotation: Fx,
    shield_position: FixedVec2,
    other_position: FixedVec2,
    directional: bool,
    desired_angle: Fx,
    arc_degrees: Fx,
    angle_from_vector: fn(FixedVec2) -> Fx,
) -> bool {
    if !directional {
        return true;
    }
    let outward = other_position - shield_position;
    if outward.is_near_zero() {
        return true;
    }
    let local_angle = wrap_radians(angle_from_vector(outward) - ship_rotation);
    let desired = wrap_radians(desired_angle);
    let half_arc = half_arc_radians_from_degrees(arc_degrees);
    wrap_radians(local_angle - desired).abs() <= half_arc
}

pub(crate) fn safe_sqrt_wide(value: WideFx) -> WideFx {
    value.max(WideFx::from_num(0)).sqrt()
}
