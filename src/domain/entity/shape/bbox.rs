use std::ops::{Bound, RangeBounds};

use crate::domain::geometry::{Point, Val};
use crate::domain::ray::Ray;

use super::DisRange;

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
