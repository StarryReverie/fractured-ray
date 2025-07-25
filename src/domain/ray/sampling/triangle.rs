use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};
use crate::domain::shape::primitive::Triangle;

use super::{LightSample, LightSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct TriangleSampler {
    id: ShapeId,
    shape: Triangle,
    normal: UnitVector,
    area_inv: Val,
}

impl TriangleSampler {
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

impl LightSampling for TriangleSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.shape)
    }

    fn light_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let (mut r1, mut r2) = (Val(rng.random()), Val(rng.random()));
        if r1 + r2 > Val(1.0) {
            r1 = Val(1.0) - r1;
            r2 = Val(1.0) - r2;
        }
        let point = (Val(1.0) - r1 - r2) * self.shape.vertex0().into_vector()
            + r1 * self.shape.vertex1().into_vector()
            + r2 * self.shape.vertex2().into_vector();
        let point = Point::from(point);
        let Ok(direction) = (point - intersection.position()).normalize() else {
            return None;
        };
        let ray_next = Ray::new(intersection.position(), direction);

        let bsdf = material.bsdf(-ray.direction(), intersection, ray_next.direction());
        if bsdf.norm_squared() != Val(0.0) {
            let cos1 = direction.dot(intersection.normal());
            let cos2 = self.normal.dot(ray_next.direction()).abs();
            let dis_squared = (point - intersection.position()).norm_squared();
            let pdf = self.area_inv * dis_squared / cos2;
            let coefficient = bsdf * cos1 / pdf;
            Some(LightSample::new(ray_next, coefficient, pdf, self.id))
        } else {
            None
        }
    }

    fn light_pdf(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        if let Some(intersection_next) = self.shape.hit(ray_next, DisRange::positive()) {
            let cos = self.normal.dot(ray_next.direction()).abs();
            let point = intersection_next.position();
            let dis_squared = (point - intersection.position()).norm_squared();
            self.area_inv * dis_squared / cos
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ray::SurfaceSide;
    use crate::domain::shape::def::ShapeKind;

    use super::*;

    #[test]
    fn triangle_sampler_light_pdf_succeeds() {
        let sampler = TriangleSampler::new(
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
            sampler.light_pdf(&intersection, &ray_next),
            Val(2.0).powi(2) / Val(1.5) / Val(0.6666666667),
        );

        let ray_next = Ray::new(intersection.position(), UnitVector::y_direction());
        assert_eq!(sampler.light_pdf(&intersection, &ray_next), Val(0.0),);
    }
}
