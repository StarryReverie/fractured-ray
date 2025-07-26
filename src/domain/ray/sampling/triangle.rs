use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::Triangle;

use super::{LightSample, LightSamplerAdapter, LightSampling, PointSample, PointSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct TrianglePointSampler {
    id: ShapeId,
    shape: Triangle,
    normal: UnitVector,
    area_inv: Val,
}

impl TrianglePointSampler {
    pub fn new(id: ShapeId, shape: Triangle) -> Self {
        let normal = shape.normal();
        let area_inv = shape.area().recip();
        Self {
            id,
            shape,
            normal,
            area_inv,
        }
    }
}

impl PointSampling for TrianglePointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.shape)
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let (mut r1, mut r2) = (Val(rng.random()), Val(rng.random()));
        if r1 + r2 > Val(1.0) {
            r1 = Val(1.0) - r1;
            r2 = Val(1.0) - r2;
        }
        let point = (Val(1.0) - r1 - r2) * self.shape.vertex0().into_vector()
            + r1 * self.shape.vertex1().into_vector()
            + r2 * self.shape.vertex2().into_vector();
        let point = Point::from(point);
        Some(PointSample::new(
            point,
            self.normal(point),
            self.pdf_point_checked_inside(point),
            self.id,
        ))
    }

    fn pdf_point(&self, point: Point) -> Val {
        let p0 = point - self.shape.vertex0();
        let p1 = point - self.shape.vertex1();
        let p2 = point - self.shape.vertex2();
        let a = p1.cross(p2).norm_squared() * self.area_inv;
        let b = p2.cross(p0).norm_squared() * self.area_inv;
        let c = p0.cross(p1).norm_squared() * self.area_inv;
        let sum = a + b + c;
        if a >= Val(0.0) && b >= Val(0.0) && c >= Val(0.0) && sum == Val(1.0) {
            self.area_inv
        } else {
            Val(0.0)
        }
    }

    fn pdf_point_checked_inside(&self, _point: Point) -> Val {
        self.area_inv
    }

    fn normal(&self, _point: Point) -> UnitVector {
        self.normal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriangleLightSampler {
    inner: LightSamplerAdapter<TrianglePointSampler>,
}

impl TriangleLightSampler {
    pub fn new(id: ShapeId, shape: Triangle) -> Self {
        let inner = TrianglePointSampler::new(id, shape);
        let inner = LightSamplerAdapter::new(inner);
        Self { inner }
    }
}

impl LightSampling for TriangleLightSampler {
    fn id(&self) -> Option<ShapeId> {
        self.inner.id()
    }

    fn shape(&self) -> Option<&dyn Shape> {
        self.inner.shape()
    }

    fn sample_light(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        self.inner.sample_light(ray, intersection, material, rng)
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.inner.pdf_light(intersection, ray_next)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ray::SurfaceSide;
    use crate::domain::shape::def::ShapeKind;

    use super::*;

    #[test]
    fn triangle_sampler_pdf_light_succeeds() {
        let sampler = TriangleLightSampler::new(
            ShapeId::new(ShapeKind::Triangle, 0),
            Triangle::new(
                Point::new(Val(-2.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(-1.0)),
                Point::new(Val(0.0), Val(1.0), Val(0.0)),
            )
            .unwrap(),
        );

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(0.0), Val(0.0), Val(1.0)),
            UnitVector::y_direction(),
            SurfaceSide::Front,
        );

        let ray_next = Ray::new(intersection.position(), -UnitVector::z_direction());
        assert_eq!(
            sampler.pdf_light(&intersection, &ray_next),
            Val(2.0).powi(2) / Val(1.5) / Val(0.6666666667),
        );

        let ray_next = Ray::new(intersection.position(), UnitVector::y_direction());
        assert_eq!(sampler.pdf_light(&intersection, &ray_next), Val(0.0));
    }
}
