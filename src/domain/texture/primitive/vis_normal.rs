use crate::domain::color::Spectrum;
use crate::domain::math::algebra::Vector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind, UvCoordinate};

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
    fn lookup_at(&self, intersection: &RayIntersection) -> Spectrum {
        let val = intersection.normal() * Val(0.5) + Vector::broadcast(Val(0.5));
        Spectrum::new(val.x(), val.y(), val.z())
    }

    #[inline]
    fn lookup(&self, _position: Point, _uv: Option<UvCoordinate>) -> Spectrum {
        Spectrum::broadcast(Val(1.0))
    }
}
