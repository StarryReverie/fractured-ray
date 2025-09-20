use std::ops::RangeBounds;

use getset::CopyGetters;
use snafu::prelude::*;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::geometry::{Area, Distance, Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart, SurfaceSide};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{LightSampling, SphereLightSampler};
use crate::domain::sampling::photon::{PhotonSamplerAdapter, PhotonSampling};
use crate::domain::sampling::point::{PointSampling, SpherePointSampler};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeKind};
use crate::domain::shape::util::ShapeId;

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Sphere {
    center: Point,
    radius: Val,
}

impl Sphere {
    pub fn new(center: Point, radius: Val) -> Result<Self, TryNewSphereError> {
        ensure!(radius > Val(0.0), InvalidRadiusSnafu);
        Ok(Self { center, radius })
    }

    pub fn unit(center: Point) -> Self {
        Self {
            center,
            radius: Val(1.0),
        }
    }
}

impl Shape for Sphere {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Sphere
    }

    fn hit_part<'a>(&self, ray: &'a Ray, range: DisRange) -> Option<RayIntersectionPart<'a>> {
        let a = ray.direction().norm_squared();
        let b = Val(2.0) * (ray.start() - self.center).dot(ray.direction());
        let c = (ray.start() - self.center).norm_squared() - self.radius * self.radius;
        let discriminant = b * b - Val(4.0) * a * c;

        let distance = if discriminant > Val(0.0) {
            let x1 = (-b - discriminant.sqrt()) / (Val(2.0) * a);
            let x2 = (-b + discriminant.sqrt()) / (Val(2.0) * a);
            if let Some(x1) = Distance::new(x1).ok().filter(|x| range.contains(x)) {
                x1
            } else if let Some(x2) = Distance::new(x2).ok().filter(|x| range.contains(x)) {
                x2
            } else {
                return None;
            }
        } else if discriminant == Val(0.0) {
            let x = -b / (Val(2.0) * a);
            Distance::new(x).ok().filter(|x| range.contains(x))?
        } else {
            return None;
        };

        Some(RayIntersectionPart::new(distance, ray))
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        let position = part.ray().at(part.distance());
        let normal =
            Normal::normalize(position - self.center).expect("normal should not be zero vector");
        let (normal, side) = if part.ray().direction().dot(normal) < Val(0.0) {
            (normal, SurfaceSide::Front)
        } else {
            (-normal, SurfaceSide::Back)
        };
        RayIntersection::new(part.distance(), position, normal, side)
    }

    fn area(&self) -> Area {
        Area::new(Val(4.0) * Val::PI * self.radius.powi(2)).unwrap()
    }

    fn normal(&self, position: Point) -> Normal {
        Normal::normalize(position - self.center).unwrap_or(Normal::x_direction())
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let d = Vector::new(self.radius, self.radius, self.radius);
        Some(BoundingBox::new(self.center - d, self.center + d))
    }
}

impl Sampleable for Sphere {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        Some(Box::new(SpherePointSampler::new(shape_id, self.clone())))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        Some(Box::new(SphereLightSampler::new(shape_id, self.clone())))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        let inner = SpherePointSampler::new(shape_id, self.clone());
        let sampler = PhotonSamplerAdapter::new(inner, emissive);
        Some(Box::new(sampler))
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewSphereError {
    #[snafu(display("radius is not positive"))]
    InvalidRadius,
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;
    use crate::domain::math::geometry::{Direction, Distance};

    use super::*;

    #[test]
    fn sphere_new_fails_when_radius_is_invalid() {
        assert!(matches!(
            Sphere::new(Point::default(), Val(0.0)),
            Err(TryNewSphereError::InvalidRadius),
        ));
    }

    #[test]
    fn sphere_hit_succeeds_returning_intersection_outside() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        let ray = Ray::new(
            Point::new(Val(2.0), Val(0.0), Val(0.0)),
            Direction::normalize(Vector::new(Val(-1.0), Val(1.0), Val(0.0))).unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(
            intersection.distance(),
            Distance::new(Val(2.0).sqrt()).unwrap()
        );
        assert_eq!(
            intersection.position(),
            Point::new(Val(1.0), Val(1.0), Val(0.0)),
        );
        assert_eq!(intersection.normal(), Normal::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }

    #[test]
    fn sphere_hit_succeeds_returning_tangent_intersection() {
        let sphere = Sphere::new(Point::new(Val(1.0), Val(0.5), Val(-1.0)), Val(0.5)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.5), Val(0.5), Val(1.0)),
            -Direction::z_direction(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Distance::new(Val(2.0)).unwrap());
    }

    #[test]
    fn sphere_hit_succeeds_returning_intersection_inside() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Direction::normalize(Vector::new(Val(1.0), Val(1.0), Val(0.0))).unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(
            intersection.distance(),
            Distance::new(Val(2.0).sqrt()).unwrap()
        );
        assert_eq!(
            intersection.position(),
            Point::new(Val(1.0), Val(1.0), Val(0.0)),
        );
        assert_eq!(intersection.normal(), -Normal::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn shpere_hit_succeeds_returning_none() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(1.000001)),
            Direction::y_direction(),
        );
        assert!(sphere.hit(&ray, DisRange::positive()).is_none());
    }

    #[test]
    fn sphere_bounding_box_succeeds() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        assert_eq!(
            sphere.bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(-1.0), Val(0.0), Val(-1.0)),
                Point::new(Val(1.0), Val(2.0), Val(1.0)),
            )),
        );
    }
}
