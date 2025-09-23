use getset::CopyGetters;

use crate::domain::color::Spectrum;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Point, SpreadAngle};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, SurfaceSide};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::texture::def::{DynTexture, Texture, UvCoordinate};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
pub struct Emissive {
    radiance: DynTexture,
    #[getset(get_copy = "pub")]
    beam_angle: SpreadAngle,
}

impl Emissive {
    pub fn new<T>(radiance: T, beam_angle: SpreadAngle) -> Self
    where
        T: Into<DynTexture>,
    {
        Self {
            radiance: radiance.into(),
            beam_angle,
        }
    }

    #[inline]
    pub fn radiance(&self) -> Spectrum {
        self.radiance.lookup(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Some(UvCoordinate::new(Val(0.0), Val(0.0)).unwrap()),
        )
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
            Contribution::from_light(self.radiance.lookup_at(intersection))
        } else {
            let cos = intersection.normal().dot(-ray.direction());
            if cos >= self.beam_angle.cos_half() {
                Contribution::from_light(self.radiance.lookup_at(intersection))
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
