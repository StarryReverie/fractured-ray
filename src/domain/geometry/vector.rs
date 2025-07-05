use std::ops::{Add, Div, Mul, Neg, Sub};

use super::{Product, TryIntoUnitVectorError, UnitVector, Val};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector(Val, Val, Val);

impl Vector {
    pub fn new(x: Val, y: Val, z: Val) -> Self {
        Self(x, y, z)
    }

    pub fn x(&self) -> Val {
        self.0
    }

    pub fn y(&self) -> Val {
        self.1
    }

    pub fn z(&self) -> Val {
        self.2
    }

    pub fn norm(&self) -> Val {
        self.norm_squared().sqrt()
    }

    pub fn norm_squared(&self) -> Val {
        self.dot(*self)
    }

    pub fn normalize(self) -> Result<UnitVector, TryIntoUnitVectorError> {
        self.try_into()
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

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y(), -self.z())
    }
}

impl Mul<Val> for Vector {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

impl Mul<Vector> for Val {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector::new(self * rhs.x(), self * rhs.y(), self * rhs.z())
    }
}

impl Div<Val> for Vector {
    type Output = Self;

    fn div(self, rhs: Val) -> Self::Output {
        Self::new(self.x() / rhs, self.y() / rhs, self.z() / rhs)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3d_linear_operations_succeed() {
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
    fn vector3d_products_succeed() {
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
    fn vector3d_norms_succeed() {
        assert_eq!(
            Vector::new(Val(1.0), Val(-2.0), Val(2.0)).norm_squared(),
            Val(9.0)
        );
        assert_eq!(Vector::new(Val(1.0), Val(-2.0), Val(2.0)).norm(), Val(3.0));
    }
}
