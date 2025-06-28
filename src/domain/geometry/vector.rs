use std::ops::{Add, Div, Mul, Neg, Sub};

use super::{Product, TryIntoUnitVectorError, UnitVector};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector(f32, f32, f32);

impl Vector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z)
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }

    pub fn z(&self) -> f32 {
        self.2
    }

    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }

    pub fn norm_squared(&self) -> f32 {
        self.dot(*self)
    }

    pub fn normalize(self) -> Result<UnitVector, TryIntoUnitVectorError> {
        self.try_into()
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

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

impl Mul<Vector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector::new(self * rhs.x(), self * rhs.y(), self * rhs.z())
    }
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x() / rhs, self.y() / rhs, self.z() / rhs)
    }
}

impl Product for Vector {
    type Output = Self;

    fn dot(self, rhs: Self) -> f32 {
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
            Vector::new(1.0, -2.0, 3.0) + Vector::new(-4.0, 5.0, 8.0),
            Vector::new(-3.0, 3.0, 11.0),
        );
        assert_eq!(
            Vector::new(1.0, -2.0, 3.0) - Vector::new(-4.0, 5.0, 8.0),
            Vector::new(5.0, -7.0, -5.0),
        );
        assert_eq!(Vector::new(1.0, 2.0, 3.0) * 2.0, Vector::new(2.0, 4.0, 6.0),);
        assert_eq!(Vector::new(1.0, 2.0, 3.0) / 2.0, Vector::new(0.5, 1.0, 1.5),);
    }

    #[test]
    fn vector3d_products_succeed() {
        assert_eq!(
            Vector::new(1.0, 1.0, -4.0).dot(Vector::new(1.0, -2.0, 2.0)),
            -9.0,
        );
        assert_eq!(
            Vector::new(0.0, -2.0, 2.0).cross(Vector::new(1.0, 2.0, 1.0)),
            Vector::new(-6.0, 2.0, 2.0),
        )
    }

    #[test]
    fn vector3d_norms_succeed() {
        assert_eq!(Vector::new(1.0, -2.0, 2.0).norm_squared(), 9.0);
        assert_eq!(Vector::new(1.0, -2.0, 2.0).norm(), 3.0);
    }
}
