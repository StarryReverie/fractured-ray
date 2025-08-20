use std::ops::RangeBounds;

use getset::CopyGetters;
use snafu::prelude::*;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, SurfaceSide};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::{LightSamplerAdapter, LightSampling};
use crate::domain::sampling::photon::{PhotonSamplerAdapter, PhotonSampling};
use crate::domain::sampling::point::{PointSampling, TrianglePointSampler};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeId, ShapeKind};

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Triangle {
    vertex0: Point,
    vertex1: Point,
    vertex2: Point,
}

impl Triangle {
    pub fn new(
        vertex0: Point,
        vertex1: Point,
        vertex2: Point,
    ) -> Result<Self, TryNewTriangleError> {
        Self::validate_vertices(&vertex0, &vertex1, &vertex2)?;
        Ok(Self {
            vertex0,
            vertex1,
            vertex2,
        })
    }

    pub fn validate_vertices(
        vertex0: &Point,
        vertex1: &Point,
        vertex2: &Point,
    ) -> Result<(), TryNewTriangleError> {
        ensure!(
            vertex0 != vertex1 && vertex1 != vertex2 && vertex0 != vertex2,
            DuplicatedVerticesSnafu
        );
        ensure!(
            !(*vertex1 - *vertex0).is_parallel_to(*vertex2 - *vertex0),
            ParallelSidesSnafu
        );
        Ok(())
    }

    pub fn calc_ray_intersection(
        ray: &Ray,
        range: DisRange,
        vertex0: &Point,
        vertex1: &Point,
        vertex2: &Point,
    ) -> Option<RayIntersection> {
        let side1 = *vertex1 - *vertex0;
        let side2 = *vertex2 - *vertex0;
        let vec0 = ray.direction().cross(side2);
        let det = side1.dot(vec0);
        if det == Val(0.0) {
            return None;
        }

        let inv_det = det.recip();
        let vec1 = ray.start() - *vertex0;
        let u = vec0.dot(vec1) * inv_det;
        if !(Val(0.0)..=Val(1.0)).contains(&u) {
            return None;
        }

        let vec2 = vec1.cross(side1);
        let v = ray.direction().dot(vec2) * inv_det;
        if !(Val(0.0)..=(Val(1.0) - u)).contains(&v) {
            return None;
        }

        let distance = side2.dot(vec2) * inv_det;
        if !range.contains(&distance) {
            return None;
        }

        let position = ray.at(distance);
        let normal = side1
            .cross(side2)
            .normalize()
            .expect("side1 and side2 should not be zero vectors and should not be parallel");
        let (normal, side) = if ray.direction().dot(normal) < Val(0.0) {
            (normal, SurfaceSide::Front)
        } else {
            (-normal, SurfaceSide::Back)
        };
        Some(RayIntersection::new(distance, position, normal, side))
    }
}

impl Shape for Triangle {
    fn kind(&self) -> ShapeKind {
        ShapeKind::Triangle
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        Self::calc_ray_intersection(ray, range, &self.vertex0, &self.vertex1, &self.vertex2)
    }

    fn area(&self) -> Val {
        let side1 = self.vertex1 - self.vertex0;
        let side2 = self.vertex2 - self.vertex0;
        Val(0.5) * side1.cross(side2).norm()
    }

    fn normal(&self, _position: Point) -> UnitVector {
        (self.vertex1 - self.vertex0)
            .cross(self.vertex2 - self.vertex0)
            .normalize()
            .expect("triangle's two sides should not parallel")
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let (v0, v1, v2) = (&self.vertex0, &self.vertex1, &self.vertex2);
        let min = v0.component_min(v1).component_min(v2);
        let max = v0.component_max(v1).component_max(v2);
        Some(BoundingBox::new(min, max))
    }
}

impl Sampleable for Triangle {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        Some(Box::new(TrianglePointSampler::new(shape_id, self.clone())))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        let inner = TrianglePointSampler::new(shape_id, self.clone());
        let sampler = LightSamplerAdapter::new(inner);
        Some(Box::new(sampler))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        let inner = TrianglePointSampler::new(shape_id, self.clone());
        let sampler = PhotonSamplerAdapter::new(inner, emissive);
        Some(Box::new(sampler))
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewTriangleError {
    #[snafu(display("triangle has duplicated vertices"))]
    DuplicatedVertices,
    #[snafu(display("triangle has parallel sides"))]
    ParallelSides,
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn triangle_new_succeeds() {
        assert!(
            Triangle::new(
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(2.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(3.0)),
            )
            .is_ok()
        );
    }

    #[test]
    fn triangle_new_fails_when_points_are_duplicated() {
        assert!(matches!(
            Triangle::new(
                Point::new(Val(0.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
            ),
            Err(TryNewTriangleError::DuplicatedVertices)
        ));
    }

    #[test]
    fn triangle_new_fails_when_sides_are_parallel() {
        assert!(matches!(
            Triangle::new(
                Point::new(Val(0.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(0.0), Val(0.0)),
                Point::new(Val(2.0), Val(0.0), Val(0.0)),
            ),
            Err(TryNewTriangleError::ParallelSides)
        ));
    }

    #[test]
    fn triangle_hit_succeeds() {
        let triangle = Triangle::new(
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(3.0)),
        )
        .unwrap();

        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.5), Val(1.0)),
            UnitVector::x_direction(),
        );

        let intersection = triangle.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(0.41666666666666663));
        assert_eq!(
            intersection.position(),
            Point::new(Val(0.41666666666666663), Val(0.5), Val(1.0))
        );
        assert_eq!(intersection.side(), SurfaceSide::Back);
    }

    #[test]
    fn triangle_hit_succeeds_returning_none() {
        let triangle = Triangle::new(
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(3.0)),
        )
        .unwrap();

        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.5)),
            Vector::new(Val(0.0), Val(1.0), Val(-0.5))
                .normalize()
                .unwrap(),
        );

        let intersection = triangle.hit(&ray, DisRange::positive());
        assert!(intersection.is_none());
    }

    #[test]
    fn triangle_area_succeeds() {
        let triangle = Triangle::new(
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(3.0)),
        )
        .unwrap();
        assert_eq!(triangle.area(), Val(3.5));
    }

    #[test]
    fn triangle_bounding_box_succeeds() {
        let triangle = Triangle::new(
            Point::new(Val(1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(3.0)),
        )
        .unwrap();

        assert_eq!(
            triangle.bounding_box(),
            Some(BoundingBox::new(
                Point::new(Val(0.0), Val(0.0), Val(0.0)),
                Point::new(Val(1.0), Val(2.0), Val(3.0)),
            )),
        );
    }
}
