use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialExt, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::{PmContext, PmState, RtContext, RtState, StoragePolicy};
use crate::domain::sampling::coefficient::{CoefficientSample, CoefficientSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Diffuse {
    color: Color,
}

impl Diffuse {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Material for Diffuse {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Diffuse
    }

    fn bsdf(
        &self,
        _dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Vector {
        if intersection.normal().dot(dir_in) > Val(0.0) {
            Val::FRAC_1_PI * self.color.to_vector()
        } else {
            Vector::zero()
        }
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Color {
        let radiance_light = self.shade_light(context, &ray, &intersection, true);
        let radiance_scattering = self.shade_scattering(context, state, &ray, &intersection, true);
        radiance_light + radiance_scattering
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        match state.policy() {
            StoragePolicy::Global => {
                self.store_photon(context, &photon, &intersection);
                self.maybe_bounce_next_photon(context, state, photon, intersection);
            }
            StoragePolicy::Caustic => {
                if state.has_specular() {
                    self.store_photon(context, &photon, &intersection);
                }
            }
        }
    }

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefficientSampling for Diffuse {
    fn sample_coefficient(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> CoefficientSample {
        let normal = intersection.normal();
        let direction = UnitVector::random_cosine_hemisphere(normal, rng);

        let ray_next = Ray::new(intersection.position(), direction);
        let pdf = self.pdf_coefficient(ray, intersection, &ray_next);
        CoefficientSample::new(ray_next, self.color.to_vector(), pdf)
    }

    fn pdf_coefficient(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let cos = ray_next.direction().dot(intersection.normal());
        cos.max(Val(0.0)) * Val::FRAC_1_PI
    }
}
