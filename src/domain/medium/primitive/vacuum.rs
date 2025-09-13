use crate::domain::color::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::medium::def::{Medium, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::renderer::{Contribution, RtContext, RtState};

#[derive(Debug, Clone, PartialEq, Eq)]
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
