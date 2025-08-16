use std::any::Any;

use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::util as ray_util;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Refractive {
    albedo: Albedo,
    refractive_index: Val,
}

impl Refractive {
    pub fn new(albedo: Albedo, refractive_index: Val) -> Result<Self, TryNewRefractiveError> {
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);

        Ok(Self {
            albedo,
            refractive_index,
        })
    }
}

impl Material for Refractive {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Refractive
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        let state_next = state.with_skip_emissive(false);
        self.shade_scattering(context, state_next, &ray, &intersection)
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let state_next = state.with_has_specular(true);
        self.maybe_bounce_next_photon(context, state_next, photon, intersection);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl BsdfMaterial for Refractive {
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        _intersection: &RayIntersection,
        _dir_in: UnitVector,
    ) -> Spectrum {
        Spectrum::zero()
    }
}

impl BsdfSampling for Refractive {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let ri = if intersection.side() == SurfaceSide::Front {
            self.refractive_index
        } else {
            self.refractive_index.recip()
        };
        let (ray_next, _) = ray_util::fresnel_refract(ray, intersection, ri, rng);
        let pdf = self.pdf_bsdf(ray, intersection, &ray_next);
        BsdfSample::new(ray_next, self.albedo.into(), pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(1.0)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewRefractiveError {
    #[snafu(display("refractive index is not positive"))]
    InvalidRefractiveIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refractive_new_fails_when_refractive_index_is_invalid() {
        assert!(matches!(
            Refractive::new(Albedo::WHITE, Val(0.0)),
            Err(TryNewRefractiveError::InvalidRefractiveIndex),
        ));
    }
}
