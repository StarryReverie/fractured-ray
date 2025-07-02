use std::ops::RangeBounds;

use crate::domain::geometry::{Point, Product, UnitVector};
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
        if den.abs() > 1e-8 {
            let num = (self.point - ray.start()).dot(self.normal);
            let distance = num / den;
            if distance > 0.0 && range.contains(&distance) {
                let position = ray.at(distance);
                let (normal, side) = if den < 0.0 {
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
