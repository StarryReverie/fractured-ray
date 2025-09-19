use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

use enum_dispatch::enum_dispatch;

use crate::domain::math::geometry::{Distance, Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart};
use crate::domain::sampling::Sampleable;
use crate::domain::shape::primitive::*;
use crate::domain::shape::util::Instance;

use super::{BoundingBox, DynShape};

#[enum_dispatch]
pub trait Shape: Sampleable + Debug + Send + Sync {
    fn kind(&self) -> ShapeKind;

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>>;

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        self.hit_part(ray, range)
            .map(|part| self.complete_part(part))
    }

    fn hit_all(&self, ray: &Ray, mut range: DisRange) -> Vec<RayIntersection> {
        let mut res = Vec::new();
        let mut last_distance = match range.start_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => unreachable!("range's start bound should not be unbounded"),
        };
        while let Some(intersection) = self.hit(ray, range) {
            let offset = Distance::new(intersection.distance() - last_distance).unwrap();
            last_distance = intersection.distance();
            range = range.advance_start(offset);
            res.push(intersection);
        }
        res
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection;

    fn area(&self) -> Val;

    fn normal(&self, position: Point) -> Normal;

    fn bounding_box(&self) -> Option<BoundingBox>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeKind {
    Aabb,
    Instance,
    MeshPolygon,
    MeshTriangle,
    Plane,
    Polygon,
    Sphere,
    Triangle,
}
