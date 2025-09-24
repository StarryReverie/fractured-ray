use enum_dispatch::enum_dispatch;

use crate::domain::color::Spectrum;
use crate::domain::math::geometry::Point;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::primitive::*;

use super::{DynTexture, UvCoordinate};

#[enum_dispatch]
pub trait Texture: Send + Sync {
    fn kind(&self) -> TextureKind;

    fn lookup(&self, position: Point, uv: Option<UvCoordinate>) -> Spectrum;

    #[inline]
    fn lookup_at(&self, intersection: &RayIntersection) -> Spectrum {
        self.lookup(intersection.position(), intersection.uv())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureKind {
    Checkerboard,
    Constant,
    VisibleNormal,
    VIsibleUvCoordinate,
}
