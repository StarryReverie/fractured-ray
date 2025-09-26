use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleUvCoordinate {}

impl VisibleUvCoordinate {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }
}

impl Texture for VisibleUvCoordinate {
    #[inline]
    fn kind(&self) -> TextureKind {
        TextureKind::VIsibleUvCoordinate
    }

    #[inline]
    fn lookup(&self, intersection: &RayIntersection) -> Spectrum {
        let uv = (intersection.uv())
            .expect("`VisibleUvCoordinate` expects a UV coordinate to be provided");
        Spectrum::new(uv.u(), uv.v(), Val(0.0))
    }
}
