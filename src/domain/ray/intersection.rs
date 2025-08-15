use getset::CopyGetters;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::{AllTransformation, Point, Transform};
use crate::domain::math::numeric::Val;

use super::Ray;

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayIntersection {
    distance: Val,
    position: Point,
    normal: UnitVector,
    side: SurfaceSide,
}

impl RayIntersection {
    pub fn new(distance: Val, position: Point, normal: UnitVector, side: SurfaceSide) -> Self {
        Self {
            distance,
            position,
            normal,
            side,
        }
    }

    #[inline]
    pub fn spawn(&self, direction: UnitVector) -> Ray {
        Ray::new(self.position, direction)
    }
}

impl Transform<AllTransformation> for RayIntersection {
    fn transform(&self, transformation: &AllTransformation) -> Self {
        RayIntersection::new(
            self.distance(),
            self.position().transform(transformation),
            self.normal().transform(transformation),
            self.side(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceSide {
    Front,
    Back,
}
