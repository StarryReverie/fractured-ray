use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::{AllTransformation, Point, Transform};
use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn distance(&self) -> Val {
        self.distance
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn side(&self) -> SurfaceSide {
        self.side
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
