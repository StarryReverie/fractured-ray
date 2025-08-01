use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::domain::math::geometry::{Rotation, Transform, Translation};
use crate::domain::math::numeric::Val;

use super::{Product, Quaternion, TryIntoUnitVectorError, UnitVector};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector(Val, Val, Val);

impl Vector {
    #[inline]
    pub const fn new(x: Val, y: Val, z: Val) -> Self {
        Self(x, y, z)
    }

    #[inline]
    pub const fn broadcast(x: Val) -> Self {
        Self(x, x, x)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(Val(0.0), Val(0.0), Val(0.0))
    }

    #[inline]
    pub fn x(&self) -> Val {
        self.0
    }

    #[inline]
    pub fn y(&self) -> Val {
        self.1
    }

    #[inline]
    pub fn z(&self) -> Val {
        self.2
    }

    #[inline]
    pub fn norm(&self) -> Val {
        self.norm_squared().sqrt()
    }

    #[inline]
    pub fn norm_squared(&self) -> Val {
        self.dot(*self)
    }

    #[inline]
    pub fn normalize(self) -> Result<UnitVector, TryIntoUnitVectorError> {
        self.try_into()
    }

    #[inline]
    pub fn lerp(a: Self, b: Self, t: Val) -> Self {
        a * (Val(1.0) - t) + b * t
    }

    #[inline]
    pub fn is_perpendicular_to<V>(&self, rhs: V) -> bool
    where
        Self: Product<V, Output = Self>,
    {
        self.dot(rhs) == Val(0.0)
    }

    #[inline]
    pub fn is_parallel_to<V>(&self, rhs: V) -> bool
    where
        Self: Product<V, Output = Self>,
    {
        self.cross(rhs).norm_squared() == Val(0.0)
    }
}

impl Add for Vector {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl AddAssign for Vector {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vector {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Vector {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y(), -self.z())
    }
}

impl Mul<Vector> for Vector {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output {
        Self::new(self.x() * rhs.x(), self.y() * rhs.y(), self.z() * rhs.z())
    }
}

impl MulAssign<Vector> for Vector {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Val> for Vector {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Val) -> Self::Output {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

impl MulAssign<Val> for Vector {
    fn mul_assign(&mut self, rhs: Val) {
        *self = *self * rhs;
    }
}

impl Mul<Vector> for Val {
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output {
        Vector::new(self * rhs.x(), self * rhs.y(), self * rhs.z())
    }
}

impl Div<Val> for Vector {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Val) -> Self::Output {
        Self::new(self.x() / rhs, self.y() / rhs, self.z() / rhs)
    }
}

impl DivAssign<Val> for Vector {
    #[inline]
    fn div_assign(&mut self, rhs: Val) {
        *self = *self / rhs;
    }
}

impl Product for Vector {
    type Output = Self;

    fn dot(self, rhs: Self) -> Val {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }

    fn cross(self, rhs: Self) -> Self::Output {
        Self::new(
            self.y() * rhs.z() - rhs.y() * self.z(),
            self.z() * rhs.x() - rhs.z() * self.x(),
            self.x() * rhs.y() - rhs.x() * self.y(),
        )
    }
}

impl Sum for Vector {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |sum, v| sum + v)
    }
}

impl Transform<Rotation> for Vector {
    fn transform(&self, transformation: &Rotation) -> Self {
        let p = Quaternion::from(*self);
        let q = transformation.quaternion();
        let q_inv = q.conjugate();

        let p = q * p * q_inv;
        Self(p.x(), p.y(), p.z())
    }
}

impl Transform<Translation> for Vector {
    fn transform(&self, _transformation: &Translation) -> Self {
        *self
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn vector_linear_operations_succeed() {
        assert_eq!(
            Vector::new(Val(1.0), Val(-2.0), Val(3.0)) + Vector::new(Val(-4.0), Val(5.0), Val(8.0)),
            Vector::new(Val(-3.0), Val(3.0), Val(11.0)),
        );
        assert_eq!(
            Vector::new(Val(1.0), Val(-2.0), Val(3.0)) - Vector::new(Val(-4.0), Val(5.0), Val(8.0)),
            Vector::new(Val(5.0), Val(-7.0), Val(-5.0)),
        );
        assert_eq!(
            Vector::new(Val(1.0), Val(2.0), Val(3.0)) * Val(2.0),
            Vector::new(Val(2.0), Val(4.0), Val(6.0)),
        );
        assert_eq!(
            Vector::new(Val(1.0), Val(2.0), Val(3.0)) / Val(2.0),
            Vector::new(Val(0.5), Val(1.0), Val(1.5)),
        );
    }

    #[test]
    fn vector_products_succeed() {
        assert_eq!(
            Vector::new(Val(1.0), Val(1.0), Val(-4.0)).dot(Vector::new(
                Val(1.0),
                Val(-2.0),
                Val(2.0)
            )),
            Val(-9.0),
        );
        assert_eq!(
            Vector::new(Val(0.0), Val(-2.0), Val(2.0)).cross(Vector::new(
                Val(1.0),
                Val(2.0),
                Val(1.0)
            )),
            Vector::new(Val(-6.0), Val(2.0), Val(2.0)),
        )
    }

    #[test]
    fn vector_norms_succeed() {
        assert_eq!(
            Vector::new(Val(1.0), Val(-2.0), Val(2.0)).norm_squared(),
            Val(9.0)
        );
        assert_eq!(Vector::new(Val(1.0), Val(-2.0), Val(2.0)).norm(), Val(3.0));
    }

    #[test]
    fn vector_rotation_transform_succeeds() {
        let rotation = Rotation::new(
            -UnitVector::z_direction(),
            Vector::new(Val(-1.0), Val(1.0), Val(0.0))
                .normalize()
                .unwrap(),
            Val::PI / Val(4.0),
        );
        let v = Vector::new(Val(1.0), Val(0.0), Val(0.0)).transform(&rotation);
        assert_eq!(v, Vector::new(Val(0.0), Val(0.0), Val(-1.0)));
    }
}
