use std::any::Any;

use rand::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::util as ray_util;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specular {
    albedo: Albedo,
}

impl Specular {
    pub fn new(albedo: Albedo) -> Self {
        Self { albedo }
    }
}

impl Material for Specular {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let state_next = state.with_skip_emissive(false);
        self.shade_scattering(context, state_next, ray, intersection)
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        let state_next = state.with_has_specular(true);
        self.maybe_bounce_next_photon(context, state_next, photon, intersection);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl BsdfMaterial for Specular {
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        _intersection: &RayIntersection,
        _dir_in: UnitVector,
    ) -> Spectrum {
        Spectrum::zero()
    }
}

impl BsdfSampling for Specular {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let direction = ray_util::reflect(ray, intersection);
        let pdf = self.pdf_bsdf(ray, intersection, &direction);
        BsdfSample::new(direction, self.albedo.into(), pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(1.0)
    }
}
