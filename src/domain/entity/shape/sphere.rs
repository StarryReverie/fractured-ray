use std::ops::RangeBounds;

use snafu::prelude::*;

use crate::domain::geometry::{Point, Product, Val, Vector};
use crate::domain::ray::Ray;

use super::{BoundingBox, DisRange, RayIntersection, Shape, SurfaceSide};

#[derive(Debug, Clone, PartialEq)]
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

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn radius(&self) -> Val {
        self.radius
    }
}

impl Shape for Sphere {
    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        let a = ray.direction().norm_squared();
        let b = Val(2.0) * (ray.start() - self.center).dot(ray.direction());
        let c = (ray.start() - self.center).norm_squared() - self.radius * self.radius;
        let discriminant = b * b - Val(4.0) * a * c;

        let distance = if discriminant > Val(0.0) {
            let x1 = (-b - discriminant.sqrt()) / (Val(2.0) * a);
            let x2 = (-b + discriminant.sqrt()) / (Val(2.0) * a);
            if x1 > Val(0.0) && range.contains(&x1) {
                x1
            } else if x2 > Val(0.0) && range.contains(&x2) {
                x2
            } else {
                return None;
            }
        } else if discriminant == Val(0.0) {
            let x = -b / (Val(2.0) * a);
            if x > Val(0.0) && range.contains(&x) {
                x
            } else {
                return None;
            }
        } else {
            return None;
        };

        let position = ray.at(distance);
        let normal = (position - self.center)
            .normalize()
            .expect("normal should not be zero vector");
        let (normal, side) = if ray.direction().dot(normal) < Val(0.0) {
            (normal, SurfaceSide::Front)
        } else {
            (-normal, SurfaceSide::Back)
        };

        Some(RayIntersection::new(distance, position, normal, side))
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let d = Vector::new(self.radius, self.radius, self.radius);
        Some(BoundingBox::new(self.center - d, self.center + d))
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
    use crate::domain::geometry::{UnitVector, Vector};

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
            Vector::new(Val(-1.0), Val(1.0), Val(0.0))
                .normalize()
                .unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(2.0).sqrt());
        assert_eq!(
            intersection.position(),
            Point::new(Val(1.0), Val(1.0), Val(0.0)),
        );
        assert_eq!(intersection.normal(), UnitVector::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }

    #[test]
    fn sphere_hit_succeeds_returning_tangent_intersection() {
        let sphere = Sphere::new(Point::new(Val(1.0), Val(0.5), Val(-1.0)), Val(0.5)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.5), Val(0.5), Val(1.0)),
            -UnitVector::z_direction(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        println!("{intersection:#?}");
    }

    #[test]
    fn sphere_hit_succeeds_returning_intersection_inside() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Vector::new(Val(1.0), Val(1.0), Val(0.0))
                .normalize()
                .unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(2.0).sqrt());
        assert_eq!(
            intersection.position(),
            Point::new(Val(1.0), Val(1.0), Val(0.0)),
        );
        assert_eq!(intersection.normal(), -UnitVector::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn shpere_hit_succeeds_returning_none() {
        let sphere = Sphere::new(Point::new(Val(0.0), Val(1.0), Val(0.0)), Val(1.0)).unwrap();
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(1.000001)),
            UnitVector::y_direction(),
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
