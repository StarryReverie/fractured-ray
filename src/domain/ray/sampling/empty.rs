use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
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
        _ray: &Ray,
        _intersection: &RayIntersection,
        _material: &dyn Material,
        _rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        None
    }

    fn pdf_light(&self, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(0.0)
    }
}
