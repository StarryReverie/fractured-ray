use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::Product;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

use super::{LightSample, LightSampling, PointSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct LightSamplerAdapter<PS>
where
    PS: PointSampling,
{
    inner: PS,
}

impl<PS> LightSamplerAdapter<PS>
where
    PS: PointSampling,
{
    pub fn new(inner: PS) -> Self {
        Self { inner }
    }
}

impl<PS> LightSampling for LightSamplerAdapter<PS>
where
    PS: PointSampling,
{
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
        let sample = self.inner.sample_point(rng)?;

        let Ok(direction) = (sample.point() - intersection.position()).normalize() else {
            return None;
        };
        let ray_next = Ray::new(intersection.position(), direction);

        let bsdf = material.bsdf(-ray.direction(), intersection, ray_next.direction());
        if bsdf.norm_squared() != Val(0.0) {
            let cos1 = direction.dot(intersection.normal());
            let cos2 = sample.normal().dot(-direction);
            let dis_squared = (sample.point() - intersection.position()).norm_squared();
            let pdf = sample.pdf() * dis_squared / cos2;
            let coefficient = bsdf * cos1 / pdf;
            Some(LightSample::new(
                ray_next,
                coefficient,
                pdf,
                sample.shape_id(),
            ))
        } else {
            None
        }
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let Some(shape) = &self.inner.shape() else {
            return Val(0.0);
        };
        if let Some(intersection_next) = shape.hit(ray_next, DisRange::positive()) {
            let point = intersection_next.position();
            let cos = self.inner.normal(point).dot(ray_next.direction()).abs();
            let dis_squared = (point - intersection.position()).norm_squared();
            self.inner.pdf_point_checked_inside(point) * dis_squared / cos
        } else {
            Val(0.0)
        }
    }
}
