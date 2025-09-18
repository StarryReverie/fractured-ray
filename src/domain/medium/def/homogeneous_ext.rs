use crate::domain::math::geometry::Direction;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};
use crate::domain::ray::util::VisibilityTester;
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::distance::{
    DistanceSample, DistanceSampling, EquiAngularDistanceSampler, ExponentialDistanceSampler,
};
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::phase::{PhaseSample, PhaseSampling};
use crate::domain::sampling::point::PointSample;

use super::HomogeneousMedium;

pub trait HomogeneousMediumExt: HomogeneousMedium {
    fn shade_light_using_light_sampling(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        segment: &RaySegment,
        distance_sample: &DistanceSample,
        preselected_light: &PointSample,
    ) -> Contribution {
        let pdf_distance = distance_sample.pdf();
        let pdf_point = preselected_light.pdf();
        if pdf_distance == Val(0.0) || pdf_point == Val(0.0) {
            return Contribution::new();
        }
        let scattering = distance_sample.scattering();

        let scene = context.entity_scene();
        let lights = scene.get_lights();
        let Some(light_sample) =
            lights.sample_light_volume(scattering, Some(preselected_light), *context.rng())
        else {
            return Contribution::new();
        };
        let pdf_light = light_sample.pdf();

        let ray_next = light_sample.ray_next();
        let vtester = VisibilityTester::new(scene, ray_next);
        let Some(target) = vtester.test(light_sample.distance(), light_sample.shape_id()) else {
            return Contribution::new();
        };

        let length = scattering.distance() - segment.start();
        let tr = self.transmittance(ray, &RaySegment::new(segment.start(), length));

        let phase = self.phase(-ray.direction(), ray_next.direction());

        let renderer = context.renderer();
        let state = RtState::new().with_skip_medium_inscattering(true);
        let radiance = renderer.trace_to(context, state, ray_next, target.as_some());

        let pdf_recip = (pdf_point * pdf_distance * pdf_light).recip();
        self.sigma_s() * tr * phase * radiance * pdf_recip
    }

    fn shade_light_using_phase_sampling(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        segment: &RaySegment,
        distance_sample: &DistanceSample,
        phase_sample: &PhaseSample,
    ) -> Contribution {
        let pdf_distance = distance_sample.pdf();
        let pdf_phase = phase_sample.pdf();
        if pdf_distance == Val(0.0) || pdf_phase == Val(0.0) {
            return Contribution::new();
        }

        let scene = context.entity_scene();
        let ray_next = phase_sample.ray_next();
        let vtester = VisibilityTester::new(scene, ray_next);
        let Some(target) = vtester.cast() else {
            return Contribution::new();
        };

        let scattering = distance_sample.scattering();
        let length = scattering.distance() - segment.start();
        let tr = self.transmittance(ray, &RaySegment::new(segment.start(), length));

        let ray_next = phase_sample.ray_next();
        let phase = self.phase(-ray.direction(), ray_next.direction());

        let renderer = context.renderer();
        let state = RtState::new().with_skip_medium_inscattering(true);
        let radiance = renderer.trace_to(context, state, ray_next, target.as_some());

        let pdf_recip = (pdf_distance * pdf_phase).recip();
        self.sigma_s() * tr * phase * radiance * pdf_recip
    }

    fn calc_exp_weight(
        ray: &Ray,
        segment: &RaySegment,
        exp_dis_sample: &DistanceSample,
        ea_sampler: &EquiAngularDistanceSampler,
    ) -> Val {
        let distance = exp_dis_sample.distance();
        let pdf2_exp = exp_dis_sample.pdf().powi(2);
        let pdf2_ea = ea_sampler.pdf_distance(ray, segment, distance).powi(2);
        pdf2_exp / (pdf2_exp + pdf2_ea)
    }

    fn calc_ea_weight(
        ray: &Ray,
        segment: &RaySegment,
        ea_dis_sampler: &DistanceSample,
        exp_sampler: &ExponentialDistanceSampler,
    ) -> Val {
        let distance = ea_dis_sampler.distance();
        let pdf2_ea = ea_dis_sampler.pdf().powi(2);
        let pdf2_exp = exp_sampler.pdf_distance(ray, segment, distance).powi(2);
        pdf2_ea / (pdf2_exp + pdf2_ea)
    }

    fn calc_light_weight(
        ray: &Ray,
        scattering: &RayScattering,
        preselected_light: &PointSample,
        light_sampler: &dyn LightSampling,
        phase_sampler: &dyn PhaseSampling,
    ) -> Val {
        let light_point = preselected_light.point();
        let Ok(direction) = Direction::normalize(light_point - scattering.position()) else {
            return Val(0.0);
        };
        let ray_next = scattering.spawn(direction);
        let pdf2_light = preselected_light.pdf()
            * light_sampler
                .pdf_light_volume(&ray_next, Some(preselected_light))
                .powi(2);
        let pdf2_phase = phase_sampler
            .pdf_phase(-ray.direction(), ray_next.direction())
            .powi(2);
        pdf2_light / (pdf2_light + pdf2_phase)
    }

    fn calc_phase_weight(phase_sample: &PhaseSample, light_sampler: &dyn LightSampling) -> Val {
        let ray_next = phase_sample.ray_next();
        let pdf2_phase = phase_sample.pdf().powi(2);
        let pdf2_light = light_sampler.pdf_light_volume(ray_next, None).powi(2);
        pdf2_phase / (pdf2_light + pdf2_phase)
    }
}

impl<M> HomogeneousMediumExt for M where M: HomogeneousMedium {}
