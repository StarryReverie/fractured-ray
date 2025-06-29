use std::ops::{Add, Div, Mul, Neg, Sub};

use snafu::prelude::*;

use super::{Product, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitVector(Vector);

impl UnitVector {
    pub fn x_direction() -> Self {
        Self(Vector::new(1.0, 0.0, 0.0))
    }

    pub fn y_direction() -> Self {
        Self(Vector::new(0.0, 1.0, 0.0))
    }

    pub fn z_direction() -> Self {
        Self(Vector::new(0.0, 0.0, 1.0))
    }

    pub fn x(&self) -> f32 {
        self.0.x()
    }

    pub fn y(&self) -> f32 {
        self.0.y()
    }

    pub fn z(&self) -> f32 {
        self.0.z()
    }

    pub fn to_vector(&self) -> Vector {
        self.0
    }
}

impl TryFrom<Vector> for UnitVector {
    type Error = TryIntoUnitVectorError;

    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        let norm = value.norm();
        ensure!(norm > 0.0, ZeroVectorSnafu);
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

            fn dot(self, rhs: $rhs_type) -> f32 {
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

impl Mul<f32> for UnitVector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<UnitVector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: UnitVector) -> Self::Output {
        self * rhs.0
    }
}

impl Div<f32> for UnitVector {
    type Output = Vector;

    fn div(self, rhs: f32) -> Self::Output {
        self.0 / rhs
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
        let v1 = Vector::new(1.0, 0.0, 0.0).normalize().unwrap();
        let v2 = Vector::new(0.0, 1.0, 0.0).normalize().unwrap();
        assert_eq!(v1 + v2.to_vector(), Vector::new(1.0, 1.0, 0.0));
        assert_eq!(v1.to_vector() - v2, Vector::new(1.0, -1.0, 0.0));
        assert_eq!(-v1, UnitVector(Vector::new(-1.0, 0.0, 0.0)));
        assert_eq!(2.0 * v1, Vector::new(2.0, 0.0, 0.0));
        assert_eq!(v2 / 2.0, Vector::new(0.0, 0.5, 0.0));
    }

    #[test]
    fn unit_vector3d_products_succeed() {
        let v1 = Vector::new(1.0, 0.0, 0.0).normalize().unwrap();
        let v2 = Vector::new(0.0, 1.0, 0.0).normalize().unwrap();
        assert_eq!(v1.dot(v2), 0.0);
        assert_eq!(v1.cross(v2), Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn unit_vector3d_try_from_succeeds() {
        assert_eq!(
            Vector::new(1.0, 2.0, 2.0).normalize(),
            Ok(UnitVector(Vector::new(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0))),
        );
        assert_eq!(
            Vector::new(0.0, 0.0, 0.0).normalize(),
            Err(TryIntoUnitVectorError::ZeroVector),
        );
    }
}
