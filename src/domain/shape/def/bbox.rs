use std::ops::{Bound, RangeBounds};

use crate::domain::math::geometry::{AllTransformation, Point, Transform};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub fn new(corner1: Point, corner2: Point) -> Self {
        Self {
            min: corner1.component_min(&corner2),
            max: corner1.component_max(&corner2),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.component_min(&other.min),
            max: self.max.component_max(&other.max),
        }
    }

    pub fn min(&self) -> Point {
        self.min
    }

    pub fn max(&self) -> Point {
        self.max
    }

    pub fn centroid(&self) -> Point {
        Point::new(
            (self.min.x()).midpoint(self.max.x()),
            (self.min.y()).midpoint(self.max.y()),
            (self.min.z()).midpoint(self.max.z()),
        )
    }

    pub fn surface_area(&self) -> Val {
        let a = self.max.x() - self.min.x();
        let b = self.max.y() - self.min.y();
        let c = self.max.z() - self.min.z();
        Val(2.0) * (a * b + a * c + b * c)
    }

    pub fn hit(&self, ray: &Ray, range: DisRange) -> Option<Val> {
        let (s, d) = (ray.start(), ray.direction());
        let xr = Self::calc_axis_range(s.x(), d.x(), self.min.x(), self.max.x());
        let yr = Self::calc_axis_range(s.y(), d.y(), self.min.y(), self.max.y());
        let zr = Self::calc_axis_range(s.z(), d.z(), self.min.z(), self.max.z());
        let range = range.intersect(xr).intersect(yr).intersect(zr);

        if range.not_empty() {
            match range.start_bound() {
                Bound::Included(distance) => Some(*distance),
                Bound::Excluded(distance) => Some(*distance),
                Bound::Unbounded => unreachable!("start_bound should be at least 0.0"),
            }
        } else {
            None
        }
    }

    fn calc_axis_range(start: Val, direction: Val, min: Val, max: Val) -> DisRange {
        if direction != Val(0.0) {
            let mut dis1 = (min - start) / direction;
            let mut dis2 = (max - start) / direction;
            if dis1 > dis2 {
                std::mem::swap(&mut dis1, &mut dis2);
            }
            DisRange::inclusive(dis1, dis2)
        } else if (min..=max).contains(&start) {
            DisRange::unbounded()
        } else {
            DisRange::empty()
        }
    }
}

impl Transform<AllTransformation> for BoundingBox {
    fn transform(&self, transformation: &AllTransformation) -> Self {
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
