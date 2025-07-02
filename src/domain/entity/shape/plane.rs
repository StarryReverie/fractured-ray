use std::ops::RangeBounds;

use crate::domain::geometry::{Point, Product, UnitVector, Val};
use crate::domain::ray::RayTrace;

use super::{DisRange, RayIntersection, Shape, SurfaceSide};

#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    point: Point,
    normal: UnitVector,
}

impl Plane {
    pub fn new(point: Point, normal: UnitVector) -> Self {
        Self { point, normal }
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }
}

impl Shape for Plane {
    fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection> {
        let den = ray.direction().dot(self.normal);
        if den != Val(0.0) {
            let num = (self.point - ray.start()).dot(self.normal);
            let distance = num / den;
            if distance > Val(0.0) && range.contains(&distance) {
                let position = ray.at(distance);
                let (normal, side) = if den < Val(0.0) {
                    (self.normal, SurfaceSide::Front)
                } else {
                    (-self.normal, SurfaceSide::Back)
                };
                Some(RayIntersection::new(distance, position, normal, side))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::geometry::Vector;

    use super::*;

    #[test]
    fn plane_hit_succeeds() {
        let plane = Plane::new(
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            UnitVector::x_direction(),
        );
        let ray_trace = RayTrace::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Vector::new(Val(-1.0), Val(0.0), Val(-1.0))
                .normalize()
                .unwrap(),
        );
        let intersection = plane.hit(&ray_trace, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(2.0).sqrt());
        assert_eq!(
            intersection.position(),
            Point::new(Val(-1.0), Val(0.0), Val(-1.0))
        );
        assert_eq!(intersection.normal(), UnitVector::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }
}
