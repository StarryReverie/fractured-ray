use getset::CopyGetters;

use crate::domain::math::geometry::{Direction, Point};
use crate::domain::math::numeric::Val;
use crate::domain::math::transformation::{AtomTransformation, Transform};
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

    pub fn spawn(&self, direction: Direction) -> Ray {
        Ray::new(self.position, direction)
    }
}

impl<T> Transform<T> for RayScattering
where
    T: AtomTransformation,
    Point: Transform<T>,
{
    fn transform(&self, transformation: &T) -> Self {
        Self::new(self.distance, self.position.transform(transformation))
    }
}
