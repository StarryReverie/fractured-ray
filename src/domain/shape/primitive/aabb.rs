use std::ops::{Bound, RangeBounds};

use getset::CopyGetters;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart, SurfaceSide};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{LightSamplerAdapter, LightSampling};
use crate::domain::sampling::photon::{PhotonSamplerAdapter, PhotonSampling};
use crate::domain::sampling::point::{AabbPointSampler, PointSampling};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::util::ShapeId;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Aabb {
    min: Point,
    max: Point,
}

impl Aabb {
    pub fn new(corner1: Point, corner2: Point) -> Self {
        Self {
            min: corner1.component_min(&corner2),
            max: corner1.component_max(&corner2),
        }
    }

    pub fn hit_range(&self, ray: &Ray) -> Option<(Val, Val)> {
        let (s, d) = (ray.start(), ray.direction());
        let xr = Self::calc_axis_range(s.x(), d.x(), self.min.x(), self.max.x());
        let yr = Self::calc_axis_range(s.y(), d.y(), self.min.y(), self.max.y());
        let zr = Self::calc_axis_range(s.z(), d.z(), self.min.z(), self.max.z());
        let range = xr.intersect(yr).intersect(zr);
        if range.not_empty() {
            match (range.start_bound(), range.end_bound()) {
                (Bound::Included(left), Bound::Included(right)) => Some((*left, *right)),
                _ => unreachable!("range should be a closed range"),
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

impl Shape for Aabb {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Aabb
    }

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        let (left, right) = self.hit_range(ray)?;
        let distance = if range.contains(&left) {
            left
        } else if range.contains(&right) {
            right
        } else {
            return None;
        };
        Some(RayIntersectionPart::new(distance, ray))
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        let position = part.ray().at(part.distance());
        let normal = self.normal(position);
        let (normal, side) = if part.ray().direction().dot(normal) < Val(0.0) {
            (normal, SurfaceSide::Front)
        } else {
            (-normal, SurfaceSide::Back)
        };
        RayIntersection::new(part.distance(), position, normal, side)
    }

    fn area(&self) -> Val {
        let a = self.max.x() - self.min.x();
        let b = self.max.y() - self.min.y();
        let c = self.max.z() - self.min.z();
        Val(2.0) * (a * b + a * c + b * c)
    }

    fn normal(&self, position: Point) -> Normal {
        if position.x() == self.min.x() {
            -Normal::x_direction()
        } else if position.x() == self.max.x() {
            Normal::x_direction()
        } else if position.y() == self.min.y() {
            -Normal::y_direction()
        } else if position.y() == self.max.y() {
            Normal::y_direction()
        } else if position.z() == self.min.z() {
            -Normal::z_direction()
        } else if position.z() == self.max.z() {
            Normal::z_direction()
        } else {
            Normal::x_direction()
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        Some(BoundingBox::new(self.min, self.max))
    }
}

impl Sampleable for Aabb {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        Some(Box::new(AabbPointSampler::new(shape_id, self.clone())))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        let inner = AabbPointSampler::new(shape_id, self.clone());
        let sampler = LightSamplerAdapter::new(inner);
        Some(Box::new(sampler))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        let inner = AabbPointSampler::new(shape_id, self.clone());
        let sampler = PhotonSamplerAdapter::new(inner, emissive);
        Some(Box::new(sampler))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;

    use super::*;

    #[test]
    fn aabb_hit_succeeds() {
        let aabb = Aabb::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(2.0), Val(3.0), Val(2.0)),
        );

        let ray = Ray::new(
            Point::new(Val(-2.0), Val(0.0), Val(0.0)),
            Vector::new(Val(2.0), Val(2.0), Val(1.0))
                .normalize()
                .unwrap(),
        );

        let intersection = aabb.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(3.0));
        assert_eq!(intersection.side(), SurfaceSide::Front);

        let intersection = aabb
            .hit(&ray, DisRange::positive().advance_start(Val(3.0)))
            .unwrap();
        assert_eq!(intersection.distance(), Val(4.5));
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn aabb_normal_succeeds() {
        let aabb = Aabb::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(2.0), Val(3.0), Val(2.0)),
        );

        assert_eq!(
            aabb.normal(Point::new(Val(0.0), Val(1.0), Val(1.0))),
            -Normal::x_direction(),
        );
        assert_eq!(
            aabb.normal(Point::new(Val(2.0), Val(1.0), Val(1.0))),
            Normal::x_direction(),
        );
        assert_eq!(
            aabb.normal(Point::new(Val(1.0), Val(0.0), Val(1.0))),
            -Normal::y_direction(),
        );
        assert_eq!(
            aabb.normal(Point::new(Val(1.0), Val(3.0), Val(1.0))),
            Normal::y_direction(),
        );
        assert_eq!(
            aabb.normal(Point::new(Val(1.0), Val(1.0), Val(0.0))),
            -Normal::z_direction(),
        );
        assert_eq!(
            aabb.normal(Point::new(Val(1.0), Val(1.0), Val(2.0))),
            Normal::z_direction(),
        );
    }
}
