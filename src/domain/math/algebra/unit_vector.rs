use std::ops::{Add, Div, Mul, Neg, Sub};

use rand::prelude::*;
use rand_distr::UnitSphere;
use snafu::prelude::*;

use crate::domain::math::geometry::{Rotation, Transform, Translation};
use crate::domain::math::numeric::Val;

use super::{Product, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitVector(Vector);

impl UnitVector {
    pub fn random(rng: &mut dyn RngCore) -> Self {
        let [x, y, z] = UnitSphere.sample(rng);
        Self(Vector::new(Val(x), Val(y), Val(z)))
    }

    pub fn x_direction() -> Self {
        Self(Vector::new(Val(1.0), Val(0.0), Val(0.0)))
    }

    pub fn y_direction() -> Self {
        Self(Vector::new(Val(0.0), Val(1.0), Val(0.0)))
    }

    pub fn z_direction() -> Self {
        Self(Vector::new(Val(0.0), Val(0.0), Val(1.0)))
    }

    pub fn x(&self) -> Val {
        self.0.x()
    }

    pub fn y(&self) -> Val {
        self.0.y()
    }

    pub fn z(&self) -> Val {
        self.0.z()
    }

    pub fn norm(&self) -> Val {
        Val(1.0)
    }

    pub fn norm_squared(&self) -> Val {
        Val(1.0)
    }

    pub fn to_vector(&self) -> Vector {
        self.0
    }

    pub fn is_perpendicular_to<V>(&self, rhs: V) -> bool
    where
        Self: Product<V, Output = Self>,
    {
        self.dot(rhs) == Val(0.0)
    }

    pub fn is_parallel_to<V>(&self, rhs: V) -> bool
    where
        Self: Product<V, Output = Self>,
    {
        self.cross(rhs).norm_squared() == Val(0.0)
    }
}

impl TryFrom<Vector> for UnitVector {
    type Error = TryIntoUnitVectorError;

    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        let norm = value.norm();
        ensure!(norm > Val(0.0), ZeroVectorSnafu);
        Ok(UnitVector(value / norm))
    }
}

impl From<UnitVector> for Vector {
    fn from(value: UnitVector) -> Self {
        value.0
    }
}

macro_rules! impl_operations {
    ($lhs_type:ty, $rhs_type:ty) => {
        impl Add<$rhs_type> for $lhs_type {
            type Output = Vector;

            fn add(self, rhs: $rhs_type) -> Self::Output {
                Vector::from(self) + Vector::from(rhs)
            }
        }

        impl Sub<$rhs_type> for $lhs_type {
            type Output = Vector;

            fn sub(self, rhs: $rhs_type) -> Self::Output {
                Vector::from(self) - Vector::from(rhs)
            }
        }

        impl Product<$rhs_type> for $lhs_type {
            type Output = Vector;

            fn dot(self, rhs: $rhs_type) -> Val {
                Vector::from(self).dot(Vector::from(rhs))
            }

            fn cross(self, rhs: $rhs_type) -> Self::Output {
                Vector::from(self).cross(Vector::from(rhs))
            }
        }
    };
}

impl_operations!(UnitVector, UnitVector);
impl_operations!(UnitVector, Vector);
impl_operations!(Vector, UnitVector);

impl Neg for UnitVector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Mul<Val> for UnitVector {
    type Output = Vector;

    fn mul(self, rhs: Val) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<UnitVector> for Val {
    type Output = Vector;

    fn mul(self, rhs: UnitVector) -> Self::Output {
        self * rhs.0
    }
}

impl Div<Val> for UnitVector {
    type Output = Vector;

    fn div(self, rhs: Val) -> Self::Output {
        self.0 / rhs
    }
}

impl Transform<Rotation> for UnitVector {
    fn transform(&self, transformation: &Rotation) -> Self {
        Self(self.0.transform(transformation))
    }
}

impl Transform<Translation> for UnitVector {
    fn transform(&self, transformation: &Translation) -> Self {
        Self(self.0.transform(transformation))
    }
}

#[derive(Debug, Snafu, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryIntoUnitVectorError {
    #[snafu(display("couldn't convert a zero vector to a unit vector"))]
    ZeroVector,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vector3d_linear_operations_succeed() {
        let v1 = Vector::new(Val(1.0), Val(0.0), Val(0.0))
            .normalize()
            .unwrap();
        let v2 = Vector::new(Val(0.0), Val(1.0), Val(0.0))
            .normalize()
            .unwrap();
        assert_eq!(
            v1 + v2.to_vector(),
            Vector::new(Val(1.0), Val(1.0), Val(0.0))
        );
        assert_eq!(
            v1.to_vector() - v2,
            Vector::new(Val(1.0), Val(-1.0), Val(0.0))
        );
        assert_eq!(-v1, UnitVector(Vector::new(Val(-1.0), Val(0.0), Val(0.0))));
        assert_eq!(Val(2.0) * v1, Vector::new(Val(2.0), Val(0.0), Val(0.0)));
        assert_eq!(v2 / Val(2.0), Vector::new(Val(0.0), Val(0.5), Val(0.0)));
    }

    #[test]
    fn unit_vector3d_products_succeed() {
        let v1 = Vector::new(Val(1.0), Val(0.0), Val(0.0))
            .normalize()
            .unwrap();
        let v2 = Vector::new(Val(0.0), Val(1.0), Val(0.0))
            .normalize()
            .unwrap();
        assert_eq!(v1.dot(v2), Val(0.0));
        assert_eq!(v1.cross(v2), Vector::new(Val(0.0), Val(0.0), Val(1.0)));
    }

    #[test]
    fn unit_vector3d_try_from_succeeds() {
        assert_eq!(
            Vector::new(Val(1.0), Val(2.0), Val(2.0)).normalize(),
            Ok(UnitVector(Vector::new(
                Val(1.0) / Val(3.0),
                Val(2.0) / Val(3.0),
                Val(2.0) / Val(3.0)
            ))),
        );
        assert_eq!(
            Vector::new(Val(0.0), Val(0.0), Val(0.0)).normalize(),
            Err(TryIntoUnitVectorError::ZeroVector),
        );
    }
}
