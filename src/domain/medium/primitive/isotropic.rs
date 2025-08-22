use std::ops::Bound;

use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::MaterialKind;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::medium::{Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::distance::{
    DistanceSample, DistanceSampling, EquiAngularDistanceSampler, ExponentialDistanceSampler,
};
use crate::domain::sampling::point::PointSample;

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

    fn calc_exp_dis_weight(
        ray: &Ray,
        segment: &RaySegment,
        distance: Val,
        exp_sampler: &ExponentialDistanceSampler,
        ea_sampler: &EquiAngularDistanceSampler,
    ) -> Val {
        let pdf2_exp = exp_sampler.pdf_distance(ray, segment, distance).powi(2);
        let pdf2_ea = ea_sampler.pdf_distance(ray, segment, distance).powi(2);
        pdf2_exp / (pdf2_exp + pdf2_ea)
    }

    fn calc_ea_dis_weight(
        ray: &Ray,
        segment: &RaySegment,
        distance: Val,
        exp_sampler: &ExponentialDistanceSampler,
        ea_sampler: &EquiAngularDistanceSampler,
    ) -> Val {
        let pdf2_exp = exp_sampler.pdf_distance(ray, segment, distance).powi(2);
        let pdf2_ea = ea_sampler.pdf_distance(ray, segment, distance).powi(2);
        pdf2_ea / (pdf2_exp + pdf2_ea)
    }

    fn shade_impl(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        segment: &RaySegment,
        distance_sample: &DistanceSample,
        preselected_light: &PointSample,
    ) -> Contribution {
        let pdf_distance = distance_sample.pdf();
        let scattering = distance_sample.scattering().clone();

        let pdf_point = preselected_light.pdf();

        let tr = self.transmittance(
            &ray,
            &RaySegment::new(segment.start(), scattering.distance() - segment.start()),
        );

        let scene = context.entity_scene();
        let lights = scene.get_lights();
        let Some(light_sample) =
            lights.sample_light_volume(&scattering, Some(preselected_light), *context.rng())
        else {
            return Contribution::new();
        };

        let (ray_next, distance) = (light_sample.ray_next(), light_sample.distance());
        let range = (Bound::Excluded(Val(0.0)), Bound::Included(distance));
        let res = scene.test_intersection(ray_next, range.into(), light_sample.shape_id());

        let (intersection_next, light) = if let Some((intersection_next, id)) = res {
            let id = id.material_id();
            let material = scene.get_entities().get_material(id).unwrap();
            if material.kind() == MaterialKind::Emissive {
                (intersection_next, material)
            } else {
                return Contribution::new();
            }
        } else {
            return Contribution::new();
        };

        let pdf_light = light_sample.pdf();
        let phase = self.phase(
            -ray.direction(),
            &scattering,
            light_sample.ray_next().direction(),
        );
        let ray_next = light_sample.into_ray_next();
        let radiance = light.shade(context, RtState::new(), ray_next, intersection_next);
        let pdf_recip = (pdf_point * pdf_distance * pdf_light).recip();
        let res = self.sigma_s * tr * phase * radiance * pdf_recip;
        res
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

    fn phase(
        &self,
        _dir_out: UnitVector,
        _scattering: &RayScattering,
        _dir_in: UnitVector,
    ) -> Spectrum {
        const PHASE: Spectrum = Spectrum::broadcast(Val(0.25 / Val::FRAC_1_PI.0));
        PHASE
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        _state: RtState,
        ray: Ray,
        segment: RaySegment,
    ) -> Contribution {
        let avg_sigma_t = self.sigma_t.norm() / Val(3.0).sqrt();
        let exp_sampler = ExponentialDistanceSampler::new(avg_sigma_t);

        let light_surfaces = context.entity_scene().get_light_surfaces();
        let Some(preselected_light) = light_surfaces.sample_point(*context.rng()) else {
            return Contribution::new();
        };
        let ea_sampler = EquiAngularDistanceSampler::new(preselected_light.point());

        let exp_dis_sample = exp_sampler.sample_distance(&ray, &segment, *context.rng());
        let ea_dis_sample = ea_sampler.sample_distance(&ray, &segment, *context.rng());

        let exp_radiance =
            self.shade_impl(context, &ray, &segment, &exp_dis_sample, &preselected_light);
        let exp_dis_weight = Self::calc_exp_dis_weight(
            &ray,
            &segment,
            exp_dis_sample.distance(),
            &exp_sampler,
            &ea_sampler,
        );
        let exp_contribution = exp_radiance * exp_dis_weight;

        let ea_radiance =
            self.shade_impl(context, &ray, &segment, &ea_dis_sample, &preselected_light);
        let ea_dis_weight = Self::calc_ea_dis_weight(
            &ray,
            &segment,
            ea_dis_sample.distance(),
            &exp_sampler,
            &ea_sampler,
        );
        let ea_contribution = ea_radiance * ea_dis_weight;

        let mut res = Contribution::new();
        res.add_light(exp_contribution.light());
        res.add_light(ea_contribution.light());
        res
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewIsotropicError {
    #[snafu(display("mean free path's each component should be positive"))]
    InvalidMeanFreePath,
}
