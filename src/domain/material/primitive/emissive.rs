use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{CoefficientSample, CoefficientSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Emissive {
    radiance: Color,
}

impl Emissive {
    pub fn new(radiance: Color) -> Self {
        Self { radiance }
    }

    pub fn radiance(&self) -> Color {
        self.radiance
    }
}

impl Material for Emissive {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Emissive
    }

    fn bsdf(
        &self,
        _dir_out: UnitVector,
        _intersection: &RayIntersection,
        _dir_in: UnitVector,
    ) -> Vector {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }

    fn shade(
        &self,
        _context: &mut RtContext<'_>,
        state: RtState,
        _ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        if state.skip_emissive() || intersection.side() == SurfaceSide::Back {
            Contribution::new()
        } else {
            self.radiance.into()
        }
    }

    fn receive(
        &self,
        _context: &mut PmContext<'_>,
        _state: PmState,
        _photon: PhotonRay,
        _intersection: RayIntersection,
    ) {
    }

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefficientSampling for Emissive {
    fn sample_coefficient(
        &self,
        _ray: &Ray,
        _intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> CoefficientSample {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }

    fn pdf_coefficient(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }
}
