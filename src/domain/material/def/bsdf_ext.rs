use std::ops::{Add, AddAssign, Mul, MulAssign};

use getset::CopyGetters;
use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::Product;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::{Photon, PhotonRay, SearchPolicy};
use crate::domain::ray::util::VisibilityTester;
use crate::domain::renderer::{Contribution, PhotonInfo, PmContext, PmState, RtContext, RtState};

use super::BsdfMaterial;

pub trait BsdfMaterialExt: BsdfMaterial {
    fn shade_light(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        const SAMPLE_LIGHT_PROB: Val = Val(0.5);
        if Val(context.rng().random()) <= SAMPLE_LIGHT_PROB {
            let radiance = self.shade_light_using_light_sampling(context, ray, intersection);
            radiance * SAMPLE_LIGHT_PROB.recip()
        } else {
            let radiance = self.shade_light_using_bsdf_sampling(context, ray, intersection);
            radiance * (Val(1.0) - SAMPLE_LIGHT_PROB).recip()
        }
    }

    fn shade_light_using_light_sampling(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let scene = context.entity_scene();
        let lights = scene.get_lights();

        let res = lights.sample_light_surface(intersection, *context.rng());
        let Some(sample) = res else {
            return Contribution::new();
        };
        if sample.pdf() == Val(0.0) {
            return Contribution::new();
        }

        let ray_next = sample.ray_next();
        let vtester = VisibilityTester::new(scene, ray_next);
        let Some(target) = vtester.test(sample.distance(), sample.shape_id()) else {
            return Contribution::new();
        };
        let (intersection_next, light) = target.into();

        let pdf_light = sample.pdf();
        let pdf_bsdf = self.pdf_bsdf(ray, intersection, ray_next);
        let weight = pdf_light / (pdf_light + pdf_bsdf);

        let bsdf = self.bsdf(-ray.direction(), intersection, ray_next.direction());
        let cos = intersection.normal().dot(ray_next.direction());
        let coefficient = bsdf * cos / pdf_light;

        let ray_next = sample.into_ray_next();
        let radiance = light.shade(context, RtState::new(), &ray_next, &intersection_next);
        weight * coefficient * radiance
    }

    fn shade_light_using_bsdf_sampling(
        &self,
        context: &mut RtContext<'_>,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let scene = context.entity_scene();
        let lights = scene.get_lights();

        let sample = self.sample_bsdf(ray, intersection, *context.rng());
        if sample.pdf() == Val(0.0) {
            return Contribution::new();
        }

        let ray_next = sample.ray_next();
        let vtester = VisibilityTester::new(scene, ray_next);
        let Some(target) = vtester.cast() else {
            return Contribution::new();
        };
        let (intersection_next, light) = target.into();

        let pdf_bsdf = sample.pdf();
        let pdf_light = lights.pdf_light_surface(intersection, ray_next);
        let weight = pdf_bsdf / (pdf_light + pdf_bsdf);

        let coefficient = sample.coefficient();
        let ray_next = sample.into_ray_next();
        let radiance = light.shade(context, RtState::new(), &ray_next, &intersection_next);
        weight * coefficient * radiance
    }

    fn shade_scattering(
        &self,
        context: &mut RtContext<'_>,
        state_next: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let renderer = context.renderer();

        let sample = self.sample_bsdf(ray, intersection, *context.rng());
        if sample.pdf() == Val(0.0) {
            return Contribution::new();
        }

        let coefficient = sample.coefficient();
        let ray_next = sample.into_ray_next();
        let radiance = renderer.trace(context, state_next, &ray_next, DisRange::positive());
        coefficient * radiance
    }

    fn store_photon(
        &self,
        context: &mut PmContext<'_>,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        context.photons().push(Photon::new(
            intersection.position(),
            -photon.direction(),
            photon.throughput(),
        ));
    }

    fn maybe_bounce_next_photon(
        &self,
        context: &mut PmContext<'_>,
        state_next: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let renderer = context.renderer();
        let mut throughput = photon.throughput();

        let continue_prob = (throughput.red())
            .max(throughput.green())
            .max(throughput.blue())
            .clamp(Val(0.0), Val(1.0));
        if Val(context.rng().random()) < continue_prob {
            throughput /= continue_prob;
        } else {
            return;
        }

        let sample = self.sample_bsdf(photon.ray(), &intersection, *context.rng());
        let throughput_next = sample.coefficient() * throughput;
        let photon_next = PhotonRay::new(sample.into_ray_next(), throughput_next);
        renderer.emit(context, state_next, photon_next, DisRange::positive());
    }

    fn estimate_flux(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        photon_info: &PhotonInfo,
    ) -> FluxEstimation {
        let (pm, policy) = (photon_info.photons(), photon_info.policy());
        let center = intersection.position();
        let photons = pm.search(center, policy);

        let mut flux = Spectrum::zero();
        for photon in &photons {
            let bsdf = self.bsdf(-ray.direction(), intersection, photon.direction());
            flux += bsdf * photon.throughput();
        }

        let radius = if let SearchPolicy::Radius(radius) = policy {
            radius
        } else {
            (photons.iter())
                .map(|photon| (center - photon.position()).norm_squared())
                .max()
                .map_or(Val::INFINITY, |r2| r2.sqrt())
        };

        FluxEstimation::new(flux, photons.len().into(), radius)
    }
}

impl<M> BsdfMaterialExt for M where M: BsdfMaterial {}

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
pub struct FluxEstimation {
    #[getset(get_copy = "pub")]
    flux: Spectrum,
    #[getset(get_copy = "pub")]
    num: Val,
    #[getset(get_copy = "pub")]
    radius: Val,
}

impl FluxEstimation {
    pub fn new(flux: Spectrum, num: Val, radius: Val) -> Self {
        Self { flux, num, radius }
    }

    pub fn empty() -> Self {
        Self::new(Spectrum::zero(), Val(0.0), Val::INFINITY)
    }

    pub fn average<'a, I>(estimations: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        let estimations = estimations.filter(|e| !e.is_empty()).collect::<Vec<_>>();
        if estimations.is_empty() {
            return Self::empty();
        }

        let radius2_sum = estimations.iter().map(|e| e.radius.powi(2)).sum::<Val>();
        let radius2_avg = radius2_sum / Val::from(estimations.len());

        let (mut flux_sum, mut num_sum) = (Spectrum::zero(), Val(0.0));
        for estimation in &estimations {
            let proportion = estimation.radius.powi(2) / radius2_avg;
            flux_sum += estimation.flux / proportion;
            num_sum += estimation.num / proportion;
        }
        let len = Val::from(estimations.len());
        Self::new(flux_sum / len, num_sum / len, radius2_avg.sqrt())
    }

    pub fn is_empty(&self) -> bool {
        self.num == Val(0.0)
    }
}

impl Add for FluxEstimation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_empty() {
            rhs
        } else if rhs.is_empty() {
            self
        } else {
            Self::average([self, rhs].iter()) * Val(2.0)
        }
    }
}

impl AddAssign for FluxEstimation {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = std::mem::replace(self, FluxEstimation::empty()) + rhs;
    }
}

impl Mul<Val> for FluxEstimation {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        Self {
            flux: self.flux * rhs,
            ..self
        }
    }
}

impl Mul<FluxEstimation> for Val {
    type Output = FluxEstimation;

    #[inline]
    fn mul(self, rhs: FluxEstimation) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<Val> for FluxEstimation {
    #[inline]
    fn mul_assign(&mut self, rhs: Val) {
        self.flux *= rhs;
    }
}

impl Mul<Spectrum> for FluxEstimation {
    type Output = Self;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        Self {
            flux: self.flux * rhs,
            ..self
        }
    }
}

impl Mul<FluxEstimation> for Spectrum {
    type Output = <FluxEstimation as Mul<Spectrum>>::Output;

    #[inline]
    fn mul(self, rhs: FluxEstimation) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<Spectrum> for FluxEstimation {
    #[inline]
    fn mul_assign(&mut self, rhs: Spectrum) {
        self.flux *= rhs;
    }
}

impl Default for FluxEstimation {
    fn default() -> Self {
        Self::empty()
    }
}
