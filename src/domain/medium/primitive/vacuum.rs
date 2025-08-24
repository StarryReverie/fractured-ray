use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::{Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};
use crate::domain::renderer::{Contribution, RtContext, RtState};
use crate::domain::sampling::phase::{PhaseSample, PhaseSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Vacuum {}

impl Vacuum {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calc_phase(cos: Val) -> Val {
        if cos == Val(-1.0) { Val(1.0) } else { Val(0.0) }
    }
}

impl Medium for Vacuum {
    fn kind(&self) -> MediumKind {
        MediumKind::Vacuum
    }

    fn transmittance(&self, _ray: &Ray, _segment: &RaySegment) -> Spectrum {
        Spectrum::broadcast(Val(1.0))
    }

    fn phase(
        &self,
        dir_out: UnitVector,
        _scattering: &RayScattering,
        dir_in: UnitVector,
    ) -> Spectrum {
        Spectrum::broadcast(Self::calc_phase(dir_out.dot(dir_in)))
    }

    fn shade(
        &self,
        _context: &mut RtContext<'_>,
        _state: RtState,
        _ray: &Ray,
        _segment: &RaySegment,
    ) -> Contribution {
        Contribution::new()
    }
}

impl PhaseSampling for Vacuum {
    fn sample_phase(
        &self,
        ray: &Ray,
        scattering: &RayScattering,
        _rng: &mut dyn RngCore,
    ) -> PhaseSample {
        let ray_next = scattering.spawn(ray.direction());
        PhaseSample::new(ray_next, Spectrum::broadcast(Val(1.0)), Val(1.0))
    }

    fn pdf_phase(
        &self,
        dir_out: UnitVector,
        _scattering: &RayScattering,
        dir_in: UnitVector,
    ) -> Val {
        Self::calc_phase(dir_out.dot(dir_in))
    }
}
