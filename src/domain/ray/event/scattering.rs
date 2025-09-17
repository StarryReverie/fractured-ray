use getset::CopyGetters;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{Sequential, Transform};
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayScattering {
    distance: Val,
    position: Point,
}

impl RayScattering {
    pub fn new(distance: Val, position: Point) -> Self {
        Self { distance, position }
    }

    pub fn spawn(&self, direction: UnitVector) -> Ray {
        Ray::new(self.position, direction)
    }
}

impl Transform<Sequential> for RayScattering {
    fn transform(&self, transformation: &Sequential) -> Self {
        Self::new(self.distance, self.position.transform(transformation))
    }
}
