use rand::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{
    Contribution, PmContext, PmState, RtContext, RtState, StoragePolicy,
};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diffuse {
    albedo: Albedo,
}

impl Diffuse {
    pub fn new(albedo: Albedo) -> Self {
        Self { albedo }
    }
}

impl Material for Diffuse {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Diffuse
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        if state.visible() {
            let light = self.shade_light(context, ray, intersection);
            let caustic = self.estimate_flux(ray, intersection, context.photon_casutic());
            let scattering = self.shade_scattering(
                context,
                state.with_visible(false).with_skip_emissive(true),
                ray,
                intersection,
            );
            light + scattering + Contribution::from_caustic(caustic)
        } else {
            let global = self.estimate_flux(ray, intersection, context.photon_global());
            Contribution::from_global(global)
        }
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        match state.policy() {
            StoragePolicy::Global => {
                self.store_photon(context, photon, intersection);
                self.maybe_bounce_next_photon(context, state, photon, intersection);
            }
            StoragePolicy::Caustic => {
                if state.has_specular() {
                    self.store_photon(context, photon, intersection);
                }
            }
        }
    }
}

impl BsdfMaterial for Diffuse {
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Spectrum {
        if intersection.normal().dot(dir_in) > Val(0.0) {
            Val::FRAC_1_PI * self.albedo
        } else {
            Spectrum::zero()
        }
    }
}

impl BsdfSampling for Diffuse {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let normal = intersection.normal();
        let direction = UnitVector::random_cosine_hemisphere(normal, rng);

        let ray_next = intersection.spawn(direction);
        let pdf = self.pdf_bsdf(ray, intersection, &ray_next);
        BsdfSample::new(ray_next, self.albedo.into(), pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let cos = ray_next.direction().dot(intersection.normal());
        cos.max(Val(0.0)) * Val::FRAC_1_PI
    }
}
