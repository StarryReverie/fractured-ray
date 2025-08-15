use rand::prelude::*;

use crate::domain::math::algebra::Product;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::def::{Shape, ShapeId};

use super::{LightSample, LightSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyLightSampler {}

impl EmptyLightSampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl LightSampling for EmptyLightSampler {
    fn id(&self) -> Option<ShapeId> {
        None
    }

    fn shape(&self) -> Option<&dyn Shape> {
        None
    }

    fn sample_light(
        &self,
        _intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        None
    }

    fn pdf_light(&self, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(0.0)
    }
}

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
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let sample = self.inner.sample_point(rng)?;

        let Ok(direction) = (sample.point() - intersection.position()).normalize() else {
            return None;
        };
        let ray_next = intersection.spawn(direction);

        let cos = sample.normal().dot(direction).abs();
        let dis_squared = (sample.point() - intersection.position()).norm_squared();
        let pdf = sample.pdf() * dis_squared / cos;
        let distance = (sample.point() - intersection.position()).norm();
        Some(LightSample::new(ray_next, pdf, distance, sample.shape_id()))
    }

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let Some(shape) = &self.inner.shape() else {
            return Val(0.0);
        };
        if let Some(intersection_next) = shape.hit(ray_next, DisRange::positive()) {
            let point = intersection_next.position();
            let cos = shape.normal(point).dot(ray_next.direction()).abs();
            let dis_squared = (point - intersection.position()).norm_squared();
            self.inner.pdf_point(point, true) * dis_squared / cos
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::UnitVector;
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::SurfaceSide;
    use crate::domain::sampling::point::TrianglePointSampler;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::primitive::Triangle;

    use super::*;

    #[test]
    fn light_sampler_adapter_pdf_light_succeeds() {
        let sampler = TrianglePointSampler::new(
            ShapeId::new(ShapeKind::Triangle, 0),
            Triangle::new(
                Point::new(Val(-2.0), Val(0.0), Val(0.0)),
                Point::new(Val(0.0), Val(0.0), Val(-1.0)),
                Point::new(Val(0.0), Val(1.0), Val(0.0)),
            )
            .unwrap(),
        );
        let sampler = LightSamplerAdapter::new(sampler);

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(0.0), Val(0.0), Val(1.0)),
            UnitVector::y_direction(),
            SurfaceSide::Front,
        );

        let ray_next = intersection.spawn(-UnitVector::z_direction());
        assert_eq!(
            sampler.pdf_light(&intersection, &ray_next),
            Val(2.0).powi(2) / Val(1.5) / Val(0.6666666667),
        );

        let ray_next = intersection.spawn(UnitVector::y_direction());
        assert_eq!(sampler.pdf_light(&intersection, &ray_next), Val(0.0));
    }
}
