use super::{FixedVec2, Fx, WideFx, widen, wrap_radians};

pub(crate) const COLLISION_PUSH_STIFFNESS: Fx = Fx::from_bits(60293);
pub(crate) const COLLISION_RESTITUTION: Fx = Fx::from_bits(14418);
pub(crate) const COLLISION_HEAT_FROM_DAMAGE: Fx = Fx::from_bits(7864);
pub(crate) const COLLISION_MAX_EFFECTIVE_MASS: Fx = Fx::from_bits(512 << 16);
pub(crate) const COLLISION_MAX_EFFECTIVE_SPEED: Fx = Fx::from_bits(320 << 16);
pub(crate) const COLLISION_DAMAGE_ENERGY_DIVISOR: WideFx = WideFx::from_bits(3600i64 << 16);
pub(crate) const COLLISION_DAMAGE_ENERGY_THRESHOLD: WideFx = WideFx::from_bits(1260i64 << 16);
pub(crate) const COMPONENT_COLLIDER_RADIUS: Fx = Fx::from_bits(922_747);
pub(crate) const SHIELD_COLLIDER_RADIUS: Fx = Fx::from_bits(1_593_836);
pub(crate) const FX_QUARTER_MAX_BITS: i32 = i32::MAX / 4;

pub(crate) fn clamp_signed(value: Fx, max_abs: Fx) -> Fx {
    debug_assert!(max_abs >= Fx::from_num(0));
    value.clamp(-max_abs, max_abs)
}

pub(crate) fn clamp_non_negative(value: Fx, max_value: Fx) -> Fx {
    debug_assert!(max_value >= Fx::from_num(0));
    value.clamp(Fx::from_num(0), max_value)
}

pub(crate) fn reduced_mass_wide(mass_a: Fx, mass_b: Fx) -> WideFx {
    let mass_a = widen(clamp_non_negative(
        mass_a.max(Fx::from_num(1)),
        COLLISION_MAX_EFFECTIVE_MASS,
    ));
    let mass_b = widen(clamp_non_negative(
        mass_b.max(Fx::from_num(1)),
        COLLISION_MAX_EFFECTIVE_MASS,
    ));
    debug_assert!(mass_a > WideFx::from_num(0));
    debug_assert!(mass_b > WideFx::from_num(0));
    (mass_a * mass_b) / (mass_a + mass_b).max(WideFx::from_num(1))
}

pub(crate) fn collision_energy_wide(mass_a: Fx, mass_b: Fx, closing_speed: Fx) -> WideFx {
    let reduced_mass = reduced_mass_wide(mass_a, mass_b);
    let clamped_speed = widen(clamp_non_negative(
        closing_speed,
        COLLISION_MAX_EFFECTIVE_SPEED,
    ));
    debug_assert!(clamped_speed >= WideFx::from_num(0));
    (reduced_mass * clamped_speed * clamped_speed) / WideFx::from_num(2)
}

pub(crate) fn collision_damage_from_energy(collision_energy: WideFx) -> i32 {
    let energy = collision_energy.max(WideFx::from_num(0));
    if energy < COLLISION_DAMAGE_ENERGY_THRESHOLD {
        return 0;
    }
    let steps = energy / COLLISION_DAMAGE_ENERGY_DIVISOR;
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
