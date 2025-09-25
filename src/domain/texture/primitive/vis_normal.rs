use crate::domain::color::Spectrum;
use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibieNormal {}

impl VisibieNormal {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }
}

impl Texture for VisibieNormal {
    #[inline]
    fn kind(&self) -> TextureKind {
        TextureKind::VisibleNormal
    }

    #[inline]
    fn lookup(&self, intersection: &RayIntersection) -> Spectrum {
        let val = intersection.normal() * Val(0.5) + Vector::broadcast(Val(0.5));
        Spectrum::new(val.x(), val.y(), val.z())
    }
}
