use std::ops::RangeBounds;

use getset::CopyGetters;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Area, Distance, Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart, SurfaceSide};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::photon::PhotonSampling;
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::util::ShapeId;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
pub struct Plane {
    #[getset(get_copy = "pub")]
    point: Point,
    normal: Normal,
}

impl Plane {
    pub fn new(point: Point, normal: Normal) -> Self {
        Self { point, normal }
    }

    pub fn calc_ray_intersection_part<'a>(
        ray: &'a Ray,
        range: DisRange,
        point: &Point,
        normal: &Normal,
    ) -> Option<RayIntersectionPart<'a>> {
        let den = ray.direction().dot(*normal);
        let distance = if den != Val(0.0) {
            let num = (*point - ray.start()).dot(*normal);
            Distance::new(num / den)
                .ok()
                .filter(|d| range.contains(d))?
        } else {
            return None;
        };
        Some(RayIntersectionPart::new(distance, ray))
    }

    pub fn complete_ray_intersection_part(
        part: RayIntersectionPart,
        normal: &Normal,
    ) -> RayIntersection {
        let position = part.ray().at(part.distance());
        let (normal, side) = if normal.dot(part.ray().direction()) < Val(0.0) {
            (*normal, SurfaceSide::Front)
        } else {
            (-(*normal), SurfaceSide::Back)
        };
        RayIntersection::new(part.distance(), position, normal, side)
    }
}

impl Shape for Plane {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Plane
    }

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        Self::calc_ray_intersection_part(ray, range, &self.point, &self.normal)
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        Self::complete_ray_intersection_part(part, &self.normal)
    }

    fn area(&self) -> Area {
        Area::infinity()
    }

    fn normal(&self, _position: Point) -> Normal {
        self.normal
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        None
    }
}

impl Sampleable for Plane {
    fn get_point_sampler(&self, _shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        None
    }

    fn get_light_sampler(&self, _shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        None
    }

    fn get_photon_sampler(
        &self,
        _shape_id: ShapeId,
        _emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;
    use crate::domain::math::geometry::Direction;

    use super::*;

    #[test]
    fn plane_hit_succeeds() {
        let plane = Plane::new(
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            Normal::x_direction(),
        );
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Direction::normalize(Vector::new(Val(-1.0), Val(0.0), Val(-1.0))).unwrap(),
        );
        let intersection = plane.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(
            intersection.distance(),
            Distance::new(Val(2.0).sqrt()).unwrap()
        );
        assert_eq!(
            intersection.position(),
            Point::new(Val(-1.0), Val(0.0), Val(-1.0))
        );
        assert_eq!(intersection.normal(), Normal::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }
}
