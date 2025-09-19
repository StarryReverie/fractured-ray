use getset::CopyGetters;

use crate::domain::math::geometry::{Direction, Distance, Normal, Point};
use crate::domain::math::transformation::{AtomTransformation, Transform};
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayIntersectionPart<'a> {
    distance: Distance,
    ray: &'a Ray,
}

impl<'a> RayIntersectionPart<'a> {
    pub fn new(distance: Distance, ray: &'a Ray) -> Self {
        Self { distance, ray }
    }
}

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayIntersection {
    distance: Distance,
    position: Point,
    normal: Normal,
    side: SurfaceSide,
}

impl RayIntersection {
    pub fn new(distance: Distance, position: Point, normal: Normal, side: SurfaceSide) -> Self {
        Self {
            distance,
            position,
            normal,
            side,
        }
    }

    #[inline]
    pub fn spawn(&self, direction: Direction) -> Ray {
        Ray::new(self.position, direction)
    }
}

impl<T> Transform<T> for RayIntersection
where
    T: AtomTransformation,
    Point: Transform<T>,
    Normal: Transform<T>,
{
    fn transform(&self, transformation: &T) -> Self {
        Self::new(
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
