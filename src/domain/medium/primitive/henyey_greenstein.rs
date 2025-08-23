use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::Frame;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::{Medium, MediumExt, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::distance::{
    DistanceSampling, EquiAngularDistanceSampler, ExponentialDistanceSampler,
};
use crate::domain::sampling::phase::{PhaseSample, PhaseSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct HenyeyGreenstein {
    sigma_s: Spectrum,
    sigma_t: Spectrum,
    asymmetric: Val,
}

impl HenyeyGreenstein {
    pub fn new(
        albedo: Albedo,
        mean_free_path: Spectrum,
        asymmetric: Val,
    ) -> Result<Self, TryNewHenyeyGreensteinError> {
        ensure!(mean_free_path.red() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.green() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(mean_free_path.blue() > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(
            (Val(-1.0)..=Val(1.0)).contains(&asymmetric),
            InvalidAsymmetricParameterSnafu
        );

        let sigma_t = Spectrum::new(
            mean_free_path.red().recip(),
            mean_free_path.green().recip(),
            mean_free_path.blue().recip(),
        );
        let sigma_s = albedo * sigma_t;
        Ok(Self {
            sigma_s,
            sigma_t,
            asymmetric,
        })
    }

    fn calc_hg(&self, cos: Val) -> Val {
        let g = self.asymmetric;
        let num = Val(1.0) - g * g;
        let den = Val(4.0) * Val::PI * (Val(1.0) + g * g - Val(2.0) * g * cos).powf(Val(1.5));
        num / den
    }
}

impl Medium for HenyeyGreenstein {
    fn kind(&self) -> MediumKind {
        MediumKind::HenyeyGreenstein
    }

    fn transmittance(&self, _ray: &Ray, segment: &RaySegment) -> Spectrum {
        Spectrum::new(
            (-self.sigma_t.red() * segment.length()).exp(),
            (-self.sigma_t.green() * segment.length()).exp(),
            (-self.sigma_t.blue() * segment.length()).exp(),
        )
    }

    fn phase(
        &self,
        dir_out: UnitVector,
        _scattering: &RayScattering,
        dir_in: UnitVector,
    ) -> Spectrum {
        Spectrum::broadcast(self.calc_hg(-dir_out.dot(dir_in)))
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        _state: RtState,
        ray: Ray,
        segment: RaySegment,
    ) -> Contribution {
        let light = context.entity_scene().get_lights();
        let avg_sigma_t = self.sigma_t.norm() / Val(3.0).sqrt();
        let exp_sampler = ExponentialDistanceSampler::new(avg_sigma_t);

        let light_surfaces = context.entity_scene().get_light_surfaces();
        let Some(preselected) = light_surfaces.sample_point(*context.rng()) else {
            return Contribution::new();
        };
        let ea_sampler = EquiAngularDistanceSampler::new(preselected.point());

        let exp_sample = exp_sampler.sample_distance(&ray, &segment, *context.rng());
        let ea_sample = ea_sampler.sample_distance(&ray, &segment, *context.rng());
        let exp_scattering = exp_sample.scattering();
        let ea_scattering = ea_sample.scattering();

        let phase_exp_sample = self.sample_phase(&ray, exp_scattering, *context.rng());
        let phase_ea_sample = self.sample_phase(&ray, ea_scattering, *context.rng());

        let exp_light_contribution = {
            let radiance = self.shade_source_using_light_sampling(
                context,
                &ray,
                &segment,
                self.sigma_s,
                &exp_sample,
                &preselected,
            );
            radiance
                * Self::calc_exp_weight(&ray, &segment, &exp_sample, &ea_sampler)
                * Self::calc_light_weight(&ray, exp_scattering, &preselected, light, self)
        };

        let ea_light_contribution = {
            let radiance = self.shade_source_using_light_sampling(
                context,
                &ray,
                &segment,
                self.sigma_s,
                &ea_sample,
                &preselected,
            );
            radiance
                * Self::calc_ea_weight(&ray, &segment, &ea_sample, &exp_sampler)
                * Self::calc_light_weight(&ray, exp_scattering, &preselected, light, self)
        };

        let exp_phase_contribution = {
            let radiance = self.shade_source_using_phase_sampling(
                context,
                &ray,
                &segment,
                self.sigma_s,
                &exp_sample,
                &phase_exp_sample,
            );
            radiance
                * Self::calc_exp_weight(&ray, &segment, &exp_sample, &ea_sampler)
                * Self::calc_phase_weight(&phase_exp_sample, light)
        };

        let ea_phase_contribution = {
            let radiance = self.shade_source_using_phase_sampling(
                context,
                &ray,
                &segment,
                self.sigma_s,
                &ea_sample,
                &phase_ea_sample,
            );
            radiance
                * Self::calc_ea_weight(&ray, &segment, &ea_sample, &exp_sampler)
                * Self::calc_phase_weight(&phase_ea_sample, light)
        };

        let light_contribution = exp_light_contribution + ea_light_contribution;
        let phase_contribution = exp_phase_contribution + ea_phase_contribution;
        light_contribution + phase_contribution
    }
}

impl PhaseSampling for HenyeyGreenstein {
    fn sample_phase(
        &self,
        ray: &Ray,
        scattering: &RayScattering,
        rng: &mut dyn RngCore,
    ) -> PhaseSample {
        let phi = Val(2.0) * Val::PI * Val(rng.random());
        let (cos_phi, sin_phi) = (phi.cos(), phi.sin());

        let cos_theta = if self.asymmetric != Val(0.0) {
            let g = self.asymmetric;
            let s = Val(2.0) * Val(rng.random()) - Val(1.0);
            (Val(0.5) / g) * (Val(1.0) + g * g - ((Val(1.0) - g * g) / (Val(1.0) + g * s)).powi(2))
        } else {
            Val(2.0) * Val(rng.random()) - Val(1.0)
        };
        let sin_theta = (Val(1.0) - cos_theta * cos_theta).sqrt();

        let frame = Frame::new(ray.direction());
        let dir_next_local = Vector::new(cos_phi * sin_theta, sin_phi * sin_theta, cos_theta);
        let dir_next = frame.to_canonical(dir_next_local).normalize().unwrap();

        let ray_next = scattering.spawn(dir_next);
        let hg = self.calc_hg(cos_theta);
        PhaseSample::new(ray_next, Spectrum::broadcast(hg), hg)
    }

    fn pdf_phase(
        &self,
        direction_out: UnitVector,
        _scattering: &RayScattering,
        direction_in: UnitVector,
    ) -> Val {
        self.calc_hg(-direction_out.dot(direction_in))
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewHenyeyGreensteinError {
    #[snafu(display("mean free path's each component should be positive"))]
    InvalidMeanFreePath,
    #[snafu(display("asymmetric parameter (g) should be in [-1, 1]"))]
    InvalidAsymmetricParameter,
}
