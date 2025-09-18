use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::math::geometry::Direction;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::{HomogeneousMedium, HomogeneousMediumExt, Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::distance::{
    DistanceSampling, EquiAngularDistanceSampler, ExponentialDistanceSampler,
};
use crate::domain::sampling::phase::{PhaseSample, PhaseSampling};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isotropic {
    sigma_s: Spectrum,
    sigma_t: Spectrum,
}

impl Isotropic {
    pub fn new(albedo: Albedo, mean_free_path: Spectrum) -> Result<Self, TryNewIsotropicError> {
        ensure!(mean_free_path.red() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.green() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.blue() > Val(0.0), InvalidMeanFreePathSnafu);

        let sigma_t = Spectrum::new(
            mean_free_path.red().recip(),
            mean_free_path.green().recip(),
            mean_free_path.blue().recip(),
        );
        let sigma_s = albedo * sigma_t;
        Ok(Self { sigma_s, sigma_t })
    }
}

impl Medium for Isotropic {
    fn kind(&self) -> MediumKind {
        MediumKind::Isotropic
    }

    fn transmittance(&self, _ray: &Ray, segment: &RaySegment) -> Spectrum {
        Spectrum::new(
            (-self.sigma_t.red() * segment.length()).exp(),
            (-self.sigma_t.green() * segment.length()).exp(),
            (-self.sigma_t.blue() * segment.length()).exp(),
        )
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        _state: RtState,
        ray: &Ray,
        segment: &RaySegment,
    ) -> Contribution {
        let avg_sigma_t = self.sigma_t.norm() / Val(3.0).sqrt();
        let exp_sampler = ExponentialDistanceSampler::new(avg_sigma_t);

        let light_surfaces = context.entity_scene().get_light_surfaces();
        let Some(preselected_light) = light_surfaces.sample_point(*context.rng()) else {
            return Contribution::new();
        };
        let ea_sampler = EquiAngularDistanceSampler::new(preselected_light.point());

        let exp_sample = exp_sampler.sample_distance(ray, segment, *context.rng());
        let ea_sample = ea_sampler.sample_distance(ray, segment, *context.rng());

        let exp_radiance = self.shade_light_using_light_sampling(
            context,
            ray,
            segment,
            &exp_sample,
            &preselected_light,
        );
        let exp_weight = Self::calc_exp_weight(ray, segment, &exp_sample, &ea_sampler);
        let exp_contribution = exp_radiance * exp_weight;

        let ea_radiance = self.shade_light_using_light_sampling(
            context,
            ray,
            segment,
            &ea_sample,
            &preselected_light,
        );
        let ea_weight = Self::calc_ea_weight(ray, segment, &ea_sample, &exp_sampler);
        let ea_contribution = ea_radiance * ea_weight;

        exp_contribution + ea_contribution
    }
}

impl HomogeneousMedium for Isotropic {
    fn sigma_s(&self) -> Spectrum {
        self.sigma_s
    }

    fn phase(&self, _dir_out: Direction, _dir_in: Direction) -> Spectrum {
        const PHASE: Spectrum = Spectrum::broadcast(Val(0.25 * Val::FRAC_1_PI.0));
        PHASE
    }
}

impl PhaseSampling for Isotropic {
    fn sample_phase(
        &self,
        ray: &Ray,
        scattering: &RayScattering,
        rng: &mut dyn RngCore,
    ) -> PhaseSample {
        let ray_next = scattering.spawn(Direction::random(rng));
        let phase = self.phase(-ray.direction(), ray_next.direction());
        let pdf = self.pdf_phase(-ray.direction(), ray_next.direction());
        PhaseSample::new(ray_next, phase, pdf)
    }

    fn pdf_phase(&self, _dir_out: Direction, _dir_in: Direction) -> Val {
        Val(0.25) * Val::FRAC_1_PI
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewIsotropicError {
    #[snafu(display("mean free path's each component should be positive"))]
    InvalidMeanFreePath,
}
