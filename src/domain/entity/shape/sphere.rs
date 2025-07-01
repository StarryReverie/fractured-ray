use std::ops::RangeBounds;

use snafu::prelude::*;

use crate::domain::geometry::{Point, Product};
use crate::domain::ray::RayTrace;

use super::{DisRange, RayIntersection, Shape, SurfaceSide};

#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    center: Point,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Point, radius: f64) -> Result<Self, TryNewSphereError> {
        ensure!(radius > 0.0, InvalidRadiusSnafu);
        Ok(Self { center, radius })
    }

    pub fn unit(center: Point) -> Self {
        Self {
            center,
            radius: 1.0,
        }
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }
}

impl Shape for Sphere {
    fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection> {
        let a = ray.direction().norm_squared();
        let b = 2.0 * (ray.start() - self.center).dot(ray.direction());
        let c = (ray.start() - self.center).norm_squared() - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        let distance = if discriminant > 1e-8 {
            let x1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let x2 = (-b + discriminant.sqrt()) / (2.0 * a);
            if x1 > 0.0 && range.contains(&x1) {
                x1
            } else if x2 > 0.0 && range.contains(&x2) {
                x2
            } else {
                return None;
            }
        } else if discriminant >= 0.0 {
            let x = -b / (2.0 * a);
            if x > 0.0 && range.contains(&x) {
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
        let (normal, side) = if ray.direction().dot(normal) < 0.0 {
            (normal, SurfaceSide::Front)
        } else {
            (-normal, SurfaceSide::Back)
        };

        Some(RayIntersection::new(distance, position, normal, side))
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
            Sphere::new(Point::default(), 0.0),
            Err(TryNewSphereError::InvalidRadius),
        ));
    }

    #[test]
    fn sphere_hit_succeeds_returning_intersection_outside() {
        let sphere = Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0).unwrap();
        let ray = RayTrace::new(
            Point::new(2.0, 0.0, 0.0),
            Vector::new(-1.0, 1.0, 0.0).normalize().unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert!((intersection.distance() - 2f64.sqrt()).abs() < 1e-6);
        assert!((intersection.position() - Point::new(1.0, 1.0, 0.0)).norm() < 1e-6);
        assert!((intersection.normal() - UnitVector::x_direction()).norm() < 1e-6);
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }

    #[test]
    fn sphere_hit_succeeds_returning_tangent_intersection() {
        let sphere = Sphere::new(Point::new(1.0, 0.5, -1.0), 0.5).unwrap();
        let ray = RayTrace::new(Point::new(0.5, 0.5, 1.0), -UnitVector::z_direction());
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        println!("{intersection:#?}");
    }

    #[test]
    fn sphere_hit_succeeds_returning_intersection_inside() {
        let sphere = Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0).unwrap();
        let ray = RayTrace::new(
            Point::new(0.0, 0.0, 0.0),
            Vector::new(1.0, 1.0, 0.0).normalize().unwrap(),
        );
        let intersection = sphere.hit(&ray, DisRange::positive()).unwrap();
        assert!((intersection.distance() - 2f64.sqrt()).abs() < 1e-6);
        assert!((intersection.position() - Point::new(1.0, 1.0, 0.0)).norm() < 1e-6);
        assert!((intersection.normal() - -UnitVector::x_direction()).norm() < 1e-6);
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn shpere_hit_succeeds_returning_none() {
        let sphere = Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0).unwrap();
        let ray = RayTrace::new(Point::new(0.0, 0.0, 1.000001), UnitVector::y_direction());
        assert!(sphere.hit(&ray, DisRange::positive()).is_none());
    }
}
