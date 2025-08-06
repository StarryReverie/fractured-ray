use std::fmt::Debug;

use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::math::geometry::{AllTransformation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

pub trait LightSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<&dyn Shape>;

    fn sample_light(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample>;

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct LightSample {
    #[getset(get = "pub")]
    ray_next: Ray,
    #[getset(get_copy = "pub")]
    pdf: Val,
    #[getset(get_copy = "pub")]
    distance: Val,
    #[getset(get_copy = "pub")]
    shape_id: ShapeId,
}

impl LightSample {
    pub fn new(ray_next: Ray, pdf: Val, distance: Val, shape_id: ShapeId) -> Self {
        Self {
            ray_next,
            pdf,
            distance,
            shape_id,
        }
    }

    pub fn into_ray_next(self) -> Ray {
        self.ray_next
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            pdf: self.pdf * multiplier,
            ..self
        }
    }
}

impl Transform<AllTransformation> for LightSample {
    fn transform(&self, transformation: &AllTransformation) -> Self {
        LightSample::new(
            self.ray_next.transform(transformation),
            self.pdf,
            self.distance,
            self.shape_id,
        )
    }
}
