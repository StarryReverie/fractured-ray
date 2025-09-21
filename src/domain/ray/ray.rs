use getset::CopyGetters;

use crate::domain::math::geometry::{Direction, Distance, Point};
use crate::domain::math::transformation::{AtomTransformation, Transform};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Ray {
    start: Point,
    direction: Direction,
}

impl Ray {
    pub fn new(start: Point, direction: Direction) -> Self {
        Self { start, direction }
    }

    pub fn at(&self, distance: Distance) -> Point {
        self.start + distance.value() * self.direction
    }
}

impl<T> Transform<T> for Ray
where
    T: AtomTransformation,
    Point: Transform<T>,
    Direction: Transform<T>,
{
    fn transform_impl(self, transformation: &T) -> Self {
        Ray::new(
            self.start.transform(transformation),
            self.direction.transform(transformation),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::numeric::Val;
    use crate::domain::math::transformation::{Scaling, Sequential};

    use super::*;

    #[test]
    fn ray_at_succeeds() {
        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            Direction::x_direction(),
        );
        assert_eq!(
            ray.at(Distance::new(Val(1.0)).unwrap()),
            Point::new(Val(1.0), Val(1.0), Val(0.0))
        );
    }

    #[test]
    fn ray_transform_succeeds_given_scaling_transformation() {
        let ray = Ray::new(
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
            Direction::x_direction(),
        );
        let ray_tr =
            ray.transform(&Sequential::default().with_scaling(Scaling::uniform(Val(0.5)).unwrap()));
        assert_eq!(ray_tr.start(), Point::new(Val(0.0), Val(0.5), Val(0.0)));
    }
}
