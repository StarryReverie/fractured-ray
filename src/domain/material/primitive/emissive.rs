use getset::CopyGetters;

use crate::domain::color::Spectrum;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::SpreadAngle;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, SurfaceSide};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Emissive {
    radiance: Spectrum,
    beam_angle: SpreadAngle,
}

impl Emissive {
    pub fn new(radiance: Spectrum, beam_angle: SpreadAngle) -> Self {
        Self {
            radiance,
            beam_angle,
        }
    }
}

impl Material for Emissive {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Emissive
    }

    fn shade(
        &self,
        _context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        if state.skip_emissive() || intersection.side() == SurfaceSide::Back {
            Contribution::new()
        } else if self.beam_angle.is_hemisphere() {
            Contribution::from_light(self.radiance)
        } else {
            let cos = intersection.normal().dot(-ray.direction());
            if cos >= self.beam_angle.cos_half() {
                Contribution::from_light(self.radiance)
            } else {
                Contribution::new()
            }
        }
    }

    fn receive(
        &self,
        _context: &mut PmContext<'_>,
        _state: PmState,
        _photon: &PhotonRay,
        _intersection: &RayIntersection,
    ) {
    }
}
