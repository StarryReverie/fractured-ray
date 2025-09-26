use std::fmt::Debug;

use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::scene::entity::EntityScene;

pub trait BssrdfSampling: Debug + Send + Sync {
    fn sample_bssrdf_diffusion(
        &self,
        scene: &dyn EntityScene,
        intersection_out: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<BssrdfDiffusionSample>;

    fn pdf_bssrdf_diffusion(
        &self,
        intersection_out: &RayIntersection,
        intersection_in: &RayIntersection,
    ) -> Val;

    fn sample_bssrdf_direction(
        &self,
        intersection_in: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BssrdfDirectionSample;

    fn pdf_bssrdf_direction(&self, intersection_in: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct BssrdfDiffusionSample {
    #[getset(get_copy = "pub")]
    distance: Val,
    #[getset(get = "pub")]
    intersection_in: RayIntersection,
    #[getset(get_copy = "pub")]
    bssrdf_diffusion: Spectrum,
    #[getset(get_copy = "pub")]
    pdf: Val,
}

impl BssrdfDiffusionSample {
    pub fn new(
        distance: Val,
        intersection_in: RayIntersection,
        bssrdf_diffusion: Spectrum,
        pdf: Val,
    ) -> Self {
        Self {
            distance,
            intersection_in,
            bssrdf_diffusion,
            pdf,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct BssrdfDirectionSample {
    #[getset(get = "pub")]
    ray_next: Ray,
    #[getset(get_copy = "pub")]
    bssrdf_direction: Spectrum,
    #[getset(get_copy = "pub")]
    pdf: Val,
}

impl BssrdfDirectionSample {
    pub fn new(ray_next: Ray, bssrdf_direction: Spectrum, pdf: Val) -> Self {
        Self {
            ray_next,
            bssrdf_direction,
            pdf,
        }
    }

    pub fn into_ray_next(self) -> Ray {
        self.ray_next
    }
}
