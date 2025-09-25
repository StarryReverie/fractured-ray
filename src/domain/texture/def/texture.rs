use enum_dispatch::enum_dispatch;

use crate::domain::color::Spectrum;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::primitive::*;

use super::DynTexture;

#[enum_dispatch]
pub trait Texture: Send + Sync {
    fn kind(&self) -> TextureKind;

    fn lookup(&self, intersection: &RayIntersection) -> Spectrum;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureKind {
    Checkerboard,
    Constant,
    Noise,
    VisibleNormal,
    VIsibleUvCoordinate,
}
