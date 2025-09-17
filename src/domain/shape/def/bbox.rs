use std::ops::{Bound, RangeBounds};

use getset::CopyGetters;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::math::transformation::{Sequential, Transform};
use crate::domain::ray::Ray;
use crate::domain::shape::primitive::Aabb;

use super::Shape;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
pub struct BoundingBox(Aabb);

impl BoundingBox {
    #[inline]
    pub fn new(corner1: Point, corner2: Point) -> Self {
        Self(Aabb::new(corner1, corner2))
    }

    #[inline]
    pub fn min(&self) -> Point {
        self.0.min()
    }

    #[inline]
    pub fn max(&self) -> Point {
        self.0.max()
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self::new(
            self.min().component_min(&other.min()),
            self.max().component_max(&other.max()),
        )
    }

    pub fn centroid(&self) -> Point {
        Point::new(
            (self.min().x()).midpoint(self.max().x()),
            (self.min().y()).midpoint(self.max().y()),
            (self.min().z()).midpoint(self.max().z()),
        )
    }

    #[inline]
    pub fn surface_area(&self) -> Val {
        self.0.area()
    }

    pub fn try_hit(&self, ray: &Ray, range: DisRange) -> Option<Val> {
        if let Some((left, right)) = self.0.hit_range(ray) {
            let range = range.intersect(DisRange::inclusive(left, right));
            if range.not_empty() {
                return match range.start_bound() {
                    Bound::Included(distance) => Some(*distance),
                    Bound::Excluded(distance) => Some(*distance),
                    Bound::Unbounded => unreachable!("start_bound should be at least 0.0"),
                };
            }
        }
        None
    }
}

impl Transform<Sequential> for BoundingBox {
    fn transform(&self, transformation: &Sequential) -> Self {
        let (min, max) = (self.min(), self.max());
        let mut c1 = Point::new(Val::INFINITY, Val::INFINITY, Val::INFINITY);
        let mut c2 = Point::new(-Val::INFINITY, -Val::INFINITY, -Val::INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = Val::from(1 - i) * min.axis(0) + Val::from(i) * max.axis(0);
                    let y = Val::from(1 - j) * min.axis(1) + Val::from(j) * max.axis(1);
                    let z = Val::from(1 - k) * min.axis(2) + Val::from(k) * max.axis(2);
                    let point = Point::new(x, y, z).transform(transformation);
                    c1 = c1.component_min(&point);
                    c2 = c2.component_max(&point)
                }
            }
        }

        BoundingBox::new(c1, c2)
    }
}
