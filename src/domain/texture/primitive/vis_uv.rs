use crate::domain::color::Spectrum;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::texture::def::{Texture, TextureKind, UvCoordinate};

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
    fn lookup(&self, _position: Point, uv: Option<UvCoordinate>) -> Spectrum {
        let uv = uv.expect("`VisibleUvCoordinate` expects a UV coordinate to be provided");
        Spectrum::new(uv.u(), uv.v(), Val(0.0))
    }
}
