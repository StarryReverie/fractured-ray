use crate::domain::color::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::ray::util::VisibilityTester;
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::distance::{
    DistanceSample, DistanceSampling, EquiAngularDistanceSampler, ExponentialDistanceSampler,
};
use crate::domain::sampling::point::PointSample;

use super::Medium;

pub trait MediumExt: Medium {
    fn shade_source_using_light_sampling(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        segment: &RaySegment,
        sigma_s: Spectrum,
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

        let vtester = VisibilityTester::new(scene, light_sample.ray_next());
        let Some(target) = vtester.test(light_sample.distance(), light_sample.shape_id()) else {
            return Contribution::new();
        };
        let (intersection_next, light) = target.into();

        let pdf_light = light_sample.pdf();
        let phase = self.phase(
            -ray.direction(),
            &scattering,
            light_sample.ray_next().direction(),
        );
        let ray_next = light_sample.into_ray_next();
        let radiance = light.shade(context, RtState::new(), ray_next, intersection_next);
        let pdf_recip = (pdf_point * pdf_distance * pdf_light).recip();
        sigma_s * tr * phase * radiance * pdf_recip
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
}

impl<M> MediumExt for M where M: Medium {}
