use std::fmt::Debug;

use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;

pub trait BsdfSampling: Debug + Send + Sync {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample;

    fn pdf_bsdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct BsdfSample {
    #[getset(get = "pub")]
    ray_next: Ray,
    #[getset(get_copy = "pub")]
    coefficient: Spectrum,
    #[getset(get_copy = "pub")]
    pdf: Val,
}

impl BsdfSample {
    pub fn new(ray_next: Ray, coefficient: Spectrum, pdf: Val) -> Self {
        Self {
            ray_next,
            coefficient,
            pdf,
        }
    }

    pub fn into_ray_next(self) -> Ray {
        self.ray_next
    }
}
