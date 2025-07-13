use std::ops::Mul;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::{UnitVector, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quaternion(Val, Val, Val, Val);

impl Quaternion {
    pub fn new(w: Val, x: Val, y: Val, z: Val) -> Self {
        Self(w, x, y, z)
    }

    pub fn w(&self) -> Val {
        self.0
    }

    pub fn x(&self) -> Val {
        self.1
    }

    pub fn y(&self) -> Val {
        self.2
    }

    pub fn z(&self) -> Val {
        self.3
    }

    pub fn conjugate(self) -> Self {
        Self::new(self.0, -self.1, -self.2, -self.3)
    }
}

impl From<Vector> for Quaternion {
    fn from(value: Vector) -> Self {
        Self::new(Val(0.0), value.x(), value.y(), value.z())
    }
}

impl From<UnitVector> for Quaternion {
    fn from(value: UnitVector) -> Self {
        Self::new(Val(0.0), value.x(), value.y(), value.z())
    }
}

impl From<Point> for Quaternion {
    fn from(value: Point) -> Self {
        Self::new(Val(0.0), value.x(), value.y(), value.z())
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Self(w1, x1, y1, z1) = self;
        let Self(w2, x2, y2, z2) = rhs;
        let w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2;
        let x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2;
        let y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2;
        let z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2;
        Self::new(w, x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quaternion_mul_succeeds() {
        let q1 = Quaternion::new(Val(1.0), Val(2.0), Val(3.0), Val(4.0));
        let q2 = Quaternion::new(Val(-1.0), Val(1.0), Val(-0.5), Val(0.5));
        assert_eq!(
            q1 * q2,
            Quaternion::new(Val(-3.5), Val(2.5), Val(-0.5), Val(-7.5)),
        );
    }
}
