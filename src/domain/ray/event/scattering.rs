use getset::CopyGetters;

use crate::domain::math::geometry::{Direction, Distance, Point};
use crate::domain::math::transformation::{AtomTransformation, Transform};
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RayScattering {
    distance: Distance,
    position: Point,
}

impl RayScattering {
    pub fn new(distance: Distance, position: Point) -> Self {
        Self { distance, position }
    }

    pub fn spawn(&self, direction: Direction) -> Ray {
        Ray::new(self.position, direction)
    }
}

impl<T> Transform<T> for RayScattering
where
    T: AtomTransformation,
    Distance: Transform<T>,
    Point: Transform<T>,
{
    fn transform_impl(self, transformation: &T) -> Self {
        Self::new(
            self.distance.transform(transformation),
            self.position.transform(transformation),
        )
    }
}
