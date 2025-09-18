use rand::prelude::*;

use crate::domain::math::algebra::{Product, TryNormalizeVectorError, UnitVector, Vector};
use crate::domain::math::numeric::Val;

use super::Normal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Direction(UnitVector);

impl Direction {
    #[inline]
    pub fn normalize(vector: Vector) -> Result<Self, TryNormalizeVectorError> {
        UnitVector::normalize(vector).map(Into::into)
    }

    #[inline]
    pub fn random(rng: &mut dyn RngCore) -> Self {
        UnitVector::random(rng).into()
    }

    pub fn random_cosine_hemisphere(normal: Normal, rng: &mut dyn RngCore) -> Self {
        loop {
            let dir = Self::random(rng);
            if let Ok(direction) = Self::normalize(normal + dir) {
                return direction;
            }
        }
    }

    #[inline]
    pub fn x_direction() -> Self {
        UnitVector::x_direction().into()
    }

    #[inline]
    pub fn y_direction() -> Self {
        UnitVector::y_direction().into()
    }

    #[inline]
    pub fn z_direction() -> Self {
        UnitVector::z_direction().into()
    }

    #[inline]
    pub fn norm(&self) -> Val {
        self.0.norm()
    }

    #[inline]
    pub fn norm_squared(&self) -> Val {
        self.0.norm_squared()
    }

    #[inline]
    pub fn to_unit_vector(self) -> UnitVector {
        self.0
    }

    #[inline]
    pub fn to_vector(self) -> Vector {
        self.0.to_vector()
    }
}

impl From<UnitVector> for Direction {
    #[inline]
    fn from(value: UnitVector) -> Self {
        Self(value)
    }
}

impl From<Direction> for UnitVector {
    #[inline]
    fn from(value: Direction) -> Self {
        value.to_unit_vector()
    }
}

impl From<Direction> for Vector {
    #[inline]
    fn from(value: Direction) -> Self {
        value.to_vector()
    }
}

impl From<Normal> for Direction {
    #[inline]
    fn from(value: Normal) -> Self {
        value.to_unit_vector().into()
    }
}

crate::impl_common_methods_for_wrapper_vector!(Direction);

crate::impl_add_for_wrapper_vector!(Direction, Direction);
crate::impl_add_for_wrapper_vector!(Direction, Vector);
crate::impl_add_for_wrapper_vector!(Vector, Direction);
crate::impl_add_for_wrapper_vector!(Direction, UnitVector);
crate::impl_add_for_wrapper_vector!(UnitVector, Direction);
crate::impl_add_for_wrapper_vector!(Direction, Normal);

crate::impl_sub_for_wrapper_vector!(Direction, Direction);
crate::impl_sub_for_wrapper_vector!(Direction, Vector);
crate::impl_sub_for_wrapper_vector!(Vector, Direction);
crate::impl_sub_for_wrapper_vector!(Direction, UnitVector);
crate::impl_sub_for_wrapper_vector!(UnitVector, Direction);
crate::impl_sub_for_wrapper_vector!(Direction, Normal);

crate::impl_product_for_wrapper_vector!(Direction, Direction);
crate::impl_product_for_wrapper_vector!(Direction, Vector);
crate::impl_product_for_wrapper_vector!(Vector, Direction);
crate::impl_product_for_wrapper_vector!(Direction, UnitVector);
crate::impl_product_for_wrapper_vector!(UnitVector, Direction);
crate::impl_product_for_wrapper_vector!(Direction, Normal);

crate::impl_neg_for_wrapper_vector!(Direction);

crate::impl_commutative_mul_for_wrapper_vector_and_scalar!(Direction, Val);

crate::impl_div_for_wrapper_vector_and_scalar!(Direction, Val);

crate::impl_common_transformation_for_wrapper_vector!(Direction);
