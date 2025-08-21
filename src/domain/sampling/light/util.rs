use rand::prelude::*;

use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
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

    fn sample_light_surface(
        &self,
        _intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        None
    }

    fn pdf_light_surface(&self, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
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

    fn sample_light_surface(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        let sample = self.inner.sample_point(rng)?;
        let ray_spawner = |dir| intersection.spawn(dir);
        LightSample::convert_point_sample(intersection.position(), &sample, ray_spawner)
    }

    fn pdf_light_surface(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        LightSample::convert_point_pdf(intersection.position(), ray_next, &self.inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::UnitVector;
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::event::SurfaceSide;
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
            sampler.pdf_light_surface(&intersection, &ray_next),
            Val(2.0).powi(2) / Val(1.5) / Val(0.6666666667),
        );

        let ray_next = intersection.spawn(UnitVector::y_direction());
        assert_eq!(
            sampler.pdf_light_surface(&intersection, &ray_next),
            Val(0.0)
        );
    }
}
