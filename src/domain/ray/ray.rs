use getset::CopyGetters;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Sequential, Transform};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Ray {
    start: Point,
    direction: UnitVector,
}

impl Ray {
    pub fn new(start: Point, direction: UnitVector) -> Self {
        Self { start, direction }
    }

    pub fn at(&self, distance: Val) -> Point {
        self.start + distance * self.direction
    }
}

impl Transform<Sequential> for Ray {
    fn transform(&self, transformation: &Sequential) -> Self {
        Ray::new(
            self.start().transform(transformation),
            self.direction().transform(transformation),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_at_succeeds() {
        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            UnitVector::x_direction(),
        );
        assert_eq!(ray.at(Val(1.0)), Point::new(Val(1.0), Val(1.0), Val(0.0)));
    }
}
