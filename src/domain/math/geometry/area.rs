use std::ops::{Add, Mul, Sub};

use snafu::prelude::*;

use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Rotation, Scaling, Transform, Translation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Area(Val);

impl Area {
    #[inline]
    pub fn new(value: Val) -> Result<Self, TryNewAreaError> {
        ensure!(value >= Val(0.0), NegativeValueSnafu);
        Ok(Self(value.abs()))
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

    #[inline]
    pub fn recip(&self) -> Val {
        self.0.recip()
    }
}

impl From<Area> for Val {
    #[inline]
    fn from(value: Area) -> Self {
        value.0
    }
}

impl TryFrom<Val> for Area {
    type Error = TryNewAreaError;

    fn try_from(value: Val) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Add for Area {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

macro_rules! impl_trait_for_area {
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

impl_trait_for_area!(Add, add, Area, Val);
impl_trait_for_area!(Add, add, Val, Area);

impl_trait_for_area!(Sub, sub, Area, Area);
impl_trait_for_area!(Sub, sub, Area, Val);
impl_trait_for_area!(Sub, sub, Val, Area);

impl_trait_for_area!(Mul, mul, Area, Area);
impl_trait_for_area!(Mul, mul, Area, Val);
impl_trait_for_area!(Mul, mul, Val, Area);

impl Transform<Rotation> for Area {
    #[inline]
    fn transform_impl(self, _transformation: &Rotation) -> Self {
        self
    }
}

impl Transform<Scaling> for Area {
    #[inline]
    fn transform_impl(self, transformation: &Scaling) -> Self {
        Self(self.0 * transformation.scale().powi(2))
    }
}

impl Transform<Translation> for Area {
    #[inline]
    fn transform_impl(self, _transformation: &Translation) -> Self {
        self
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewAreaError {
    #[snafu(display("area should be non-negative"))]
    NegativeValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_new_fails_when_value_is_not_positive() {
        assert!(matches!(
            Area::new(Val(-1.0)),
            Err(TryNewAreaError::NegativeValue),
        ));
    }
}
