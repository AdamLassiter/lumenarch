use std::{
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use bevy::prelude::*;
use cordic::{cos, sin};
use fixed::{types::extra::U16, FixedI32, FixedI64};

pub(crate) type Fx = FixedI32<U16>;
pub(crate) type WideFx = FixedI64<U16>;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FieldOutput {
    pub(crate) heat: Fx,
    pub(crate) cooling: Fx,
    pub(crate) electrical: Fx,
    pub(crate) grounding: Fx,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FixedVec2 {
    pub(crate) x: Fx,
    pub(crate) y: Fx,
}

impl FixedVec2 {
    pub(crate) fn new(x: Fx, y: Fx) -> Self {
        Self { x, y }
    }

    pub(crate) fn zero() -> Self {
        Self::default()
    }

    pub(crate) fn from_num(x: impl ToFixed, y: impl ToFixed) -> Self {
        Self::new(Fx::from_num(x.to_f64()), Fx::from_num(y.to_f64()))
    }

    pub(crate) fn from_vec2(value: Vec2) -> Self {
        Self::new(Fx::from_num(value.x), Fx::from_num(value.y))
    }

    pub(crate) fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x.to_num::<f32>(), self.y.to_num::<f32>())
    }

    pub(crate) fn length_sq(self) -> WideFx {
        widen(self.x) * widen(self.x) + widen(self.y) * widen(self.y)
    }

    pub(crate) fn length(self) -> Fx {
        Fx::from_num(self.length_sq().to_num::<f64>().sqrt())
    }

    pub(crate) fn distance_sq(self, other: Self) -> WideFx {
        (self - other).length_sq()
    }

    pub(crate) fn is_near_zero(self) -> bool {
        self.length_sq() <= wide_ratio(1, 4096)
    }

    pub(crate) fn normalized_or_zero(self) -> Self {
        let length = self.length();
        if length <= Fx::from_num(0) {
            return Self::zero();
        }
        self * (Fx::from_num(1) / length)
    }

    pub(crate) fn clamp_length(self, max_length: Fx) -> Self {
        let length_sq = self.length_sq();
        let max_sq = widen(max_length) * widen(max_length);
        if length_sq <= max_sq {
            return self;
        }

        self.normalized_or_zero() * max_length
    }

    pub(crate) fn rotate(self, radians: Fx) -> Self {
        let radians = wrap_radians(radians);
        let sin_theta = sin(radians);
        let cos_theta = cos(radians);
        Self::new(
            self.x * cos_theta - self.y * sin_theta,
            self.x * sin_theta + self.y * cos_theta,
        )
    }
}

impl Add for FixedVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for FixedVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for FixedVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for FixedVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Fx> for FixedVec2 {
    type Output = Self;

    fn mul(self, rhs: Fx) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<Fx> for FixedVec2 {
    fn mul_assign(&mut self, rhs: Fx) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

pub(crate) trait ToFixed {
    fn to_f64(self) -> f64;
}

impl ToFixed for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToFixed for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToFixed for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

pub(crate) fn fx_ratio(num: i32, den: i32) -> Fx {
    Fx::from_num(num) / Fx::from_num(den.max(1))
}

pub(crate) fn wide_ratio(num: i32, den: i32) -> WideFx {
    WideFx::from_num(num) / WideFx::from_num(den.max(1))
}

pub(crate) fn widen(value: Fx) -> WideFx {
    WideFx::from_num(value)
}

pub(crate) fn fx_from_time_delta(time: &Time) -> Fx {
    Fx::from_num(time.delta_secs())
}

pub(crate) fn wrap_radians(phi: Fx) -> Fx {
    let pi = Fx::PI;
    let tau = 2 * Fx::PI;
    let mut angle = phi;

    while angle < -pi {
        angle += tau;
    }
    while pi <= angle {
        angle -= tau;
    }
    angle
}

pub(crate) fn format_fx0(value: Fx) -> String {
    format!("{:.0}", value.to_num::<f32>())
}

pub(crate) fn format_fx1(value: Fx) -> String {
    format!("{:.1}", value.to_num::<f32>())
}

pub(crate) fn format_fx2(value: Fx) -> String {
    format!("{:.2}", value.to_num::<f32>())
}
