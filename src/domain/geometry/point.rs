use std::ops::{Add, Sub};

use super::{UnitVector, Val, Vector};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point(Vector);

impl Point {
    pub fn new(x: Val, y: Val, z: Val) -> Self {
        Vector::new(x, y, z).into()
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

    pub fn to_vector(&self) -> Vector {
        self.0
    }
}

impl From<Vector> for Point {
    fn from(value: Vector) -> Self {
        Self(value)
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        value.0
    }
}

macro_rules! impl_add_sub_vector {
    ($vec_type:ty) => {
        impl Add<$vec_type> for Point {
            type Output = Point;

            fn add(self, rhs: $vec_type) -> Self::Output {
                (self.0 + Vector::from(rhs)).into()
            }
        }

        impl Add<Point> for $vec_type {
            type Output = Point;

            fn add(self, rhs: Point) -> Self::Output {
                (Vector::from(self) + rhs.0).into()
            }
        }

        impl Sub<$vec_type> for Point {
            type Output = Point;

            fn sub(self, rhs: $vec_type) -> Self::Output {
                (self.0 - Vector::from(rhs)).into()
            }
        }
    };
}

impl_add_sub_vector!(Vector);
impl_add_sub_vector!(UnitVector);

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_operation_succeed() {
        assert_eq!(
            Point::new(Val(1.0), Val(0.0), Val(0.0)) + Vector::new(Val(0.0), Val(1.0), Val(0.0)),
            Point::new(Val(1.0), Val(1.0), Val(0.0)),
        );
        assert_eq!(
            Point::new(Val(1.0), Val(0.0), Val(0.0)) - Vector::new(Val(0.0), Val(1.0), Val(0.0)),
            Point::new(Val(1.0), Val(-1.0), Val(0.0)),
        );
        assert_eq!(
            Point::new(Val(1.0), Val(2.0), Val(0.0)) - Point::new(Val(1.0), Val(1.0), Val(0.0)),
            Vector::new(Val(0.0), Val(1.0), Val(0.0)),
        );
    }
}
