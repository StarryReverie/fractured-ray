use std::ops::{Add, Mul, Sub};

use snafu::prelude::*;

use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Rotation, Scaling, Transform, Translation};

use super::Point;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Distance(Val);

impl Distance {
    #[inline]
    pub fn new(value: Val) -> Result<Self, TryNewDistanceError> {
        ensure!(
            value.is_sign_positive() || value == Val(0.0),
            NegativeValueSnafu
        );
        Ok(Self(value.abs()))
    }

    #[inline]
    pub fn clamp(value: Val) -> Self {
        Self(value.max(Val(0.0)))
    }

    #[inline]
    pub fn along(displacement: Vector) -> Self {
        Self(displacement.norm())
    }

    #[inline]
    pub fn between(from: Point, to: Point) -> Self {
        Self::along(to - from)
    }

    #[inline]
    pub fn zero() -> Self {
        Self(Val(0.0))
    }

    #[inline]
    pub fn infinity() -> Self {
        Self(Val::INFINITY)
    }

    #[inline]
    pub fn value(&self) -> Val {
        self.0
    }
}

impl Add for Distance {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

macro_rules! impl_trait_for_distance {
    ($trait:ident, $method:ident, $lhs_type:ty, $rhs_type:ty) => {
        impl $trait<$rhs_type> for $lhs_type {
            type Output = Val;

            #[inline]
            fn $method(self, rhs: $rhs_type) -> Self::Output {
                $trait::$method(Val::from(self.0), Val::from(rhs.0))
            }
        }
    };
}

impl_trait_for_distance!(Add, add, Distance, Val);
impl_trait_for_distance!(Add, add, Val, Distance);

impl_trait_for_distance!(Sub, sub, Distance, Distance);
impl_trait_for_distance!(Sub, sub, Distance, Val);
impl_trait_for_distance!(Sub, sub, Val, Distance);

impl_trait_for_distance!(Mul, mul, Distance, Distance);
impl_trait_for_distance!(Mul, mul, Distance, Val);
impl_trait_for_distance!(Mul, mul, Val, Distance);

impl From<Distance> for Val {
    #[inline]
    fn from(value: Distance) -> Self {
        value.value()
    }
}

impl TryFrom<Val> for Distance {
    type Error = TryNewDistanceError;

    #[inline]
    fn try_from(value: Val) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Transform<Rotation> for Distance {
    #[inline]
    fn transform_impl(self, _transformation: &Rotation) -> Self {
        self
    }
}

impl Transform<Scaling> for Distance {
    #[inline]
    fn transform_impl(self, transformation: &Scaling) -> Self {
        Self(self.0 * transformation.scale())
    }
}

impl Transform<Translation> for Distance {
    #[inline]
    fn transform_impl(self, _transformation: &Translation) -> Self {
        self
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewDistanceError {
    #[snafu(display("distance should be non-negative"))]
    NegativeValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_new_fails_when_argument_is_negative() {
        assert!(matches!(
            Distance::new(Val(-1.0)),
            Err(TryNewDistanceError::NegativeValue)
        ));
    }
}
