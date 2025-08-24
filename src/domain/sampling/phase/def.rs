use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayScattering;

pub trait PhaseSampling: Send + Sync {
    fn sample_phase(
        &self,
        ray: &Ray,
        scattering: &RayScattering,
        rng: &mut dyn RngCore,
    ) -> PhaseSample;

    fn pdf_phase(&self, dir_out: UnitVector, dir_in: UnitVector) -> Val;
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct PhaseSample {
    #[getset(get = "pub")]
    ray_next: Ray,
    #[getset(get_copy = "pub")]
    phase: Spectrum,
    #[getset(get_copy = "pub")]
    pdf: Val,
}

impl PhaseSample {
    pub fn new(ray_next: Ray, phase: Spectrum, pdf: Val) -> Self {
        Self {
            ray_next,
            phase,
            pdf,
        }
    }
}
