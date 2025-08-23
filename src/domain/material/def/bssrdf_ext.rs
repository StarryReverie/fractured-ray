use std::any::Any;

use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling, BssrdfDiffusionSample};

use super::{BsdfMaterial, BsdfMaterialExt, BssrdfMaterial, Material, MaterialKind};

pub trait BssrdfMaterialExt: BssrdfMaterial + Sized {
    fn shade_impl(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let scene = context.entity_scene();
        let sample = self.sample_bssrdf_diffusion(scene, intersection, *context.rng());
        if let Some(diffusion) = sample {
            let adapter = BsdfMaterialAdapter::new(self, &diffusion);
            adapter.shade(context, state, ray, intersection)
        } else {
            Contribution::new()
        }
    }

    fn receive_impl(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let scene = context.scene();
        let sample = self.sample_bssrdf_diffusion(scene, &intersection, *context.rng());
        if let Some(diffusion) = sample {
            let adapter = BsdfMaterialAdapter::new(self, &diffusion);
            adapter.receive(context, state, photon, intersection)
        }
    }
}

impl<M> BssrdfMaterialExt for M where M: BssrdfMaterial {}

#[derive(Debug)]
struct BsdfMaterialAdapter<'a, M>
where
    M: BssrdfMaterial,
{
    inner: &'a M,
    diffusion: &'a BssrdfDiffusionSample,
}

impl<'a, M> BsdfMaterialAdapter<'a, M>
where
    M: BssrdfMaterial,
{
    fn new(inner: &'a M, diffusion: &'a BssrdfDiffusionSample) -> Self {
        Self { inner, diffusion }
    }
}

impl<'a, M> Material for BsdfMaterialAdapter<'a, M>
where
    M: BssrdfMaterial,
{
    fn kind(&self) -> MaterialKind {
        MaterialKind::Refractive
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let light = self.shade_light(context, ray, intersection);
        let state_next = state.with_skip_emissive(true);
        let scattering = self.shade_scattering(context, state_next, ray, intersection);
        (light + scattering) * (self.diffusion.bssrdf_diffusion() / self.diffusion.pdf())
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let photon_scaled = PhotonRay::new(
            Ray::new(photon.start(), photon.direction()),
            photon.throughput() * (self.diffusion.bssrdf_diffusion() / self.diffusion.pdf()),
        );
        self.maybe_bounce_next_photon(context, state, photon_scaled, intersection);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

impl<'a, M> BsdfMaterial for BsdfMaterialAdapter<'a, M>
where
    M: BssrdfMaterial,
{
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Spectrum {
        self.inner.bssrdf_direction(intersection, dir_in)
    }
}

impl<'a, M> BsdfSampling for BsdfMaterialAdapter<'a, M>
where
    M: BssrdfMaterial,
{
    fn sample_bsdf(
        &self,
        _ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let sample = self.inner.sample_bssrdf_direction(intersection, rng);
        let cos = intersection.normal().dot(sample.ray_next().direction());
        let pdf = sample.pdf();
        let coefficient = sample.bssrdf_direction() * cos / pdf;
        BsdfSample::new(sample.into_ray_next(), coefficient, pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.inner.pdf_bssrdf_direction(intersection, ray_next)
    }
}
