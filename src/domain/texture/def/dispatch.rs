use enum_dispatch::enum_dispatch;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::math::geometry::Point;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::primitive::*;

use super::{Texture, TextureKind, UvCoordinate};

#[enum_dispatch(Texture)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynTexture {
    Checkerboard(Checkerboard),
    Constant(Constant),
    VisibleNormal(VisibieNormal),
    VisibleUvCoordinate(VisibleUvCoordinate),
}

impl<S> From<S> for DynTexture
where
    S: Into<Spectrum>,
{
    #[inline]
    fn from(value: S) -> Self {
        Self::Constant(Constant::new(value.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynAlbedoTexture {
    Constant(Albedo),
    Dyn(Box<DynTexture>),
}

impl DynAlbedoTexture {
    pub fn kind(&self) -> TextureKind {
        match self {
            Self::Constant(_) => TextureKind::Constant,
            Self::Dyn(s) => s.kind(),
        }
    }

    pub fn lookup(&self, position: Point, uv: Option<UvCoordinate>) -> Albedo {
        match self {
            Self::Constant(albedo) => *albedo,
            Self::Dyn(s) => Albedo::clamp(s.lookup(position, uv)),
        }
    }

    #[inline]
    pub fn lookup_at(&self, intersection: &RayIntersection) -> Albedo {
        match self {
            Self::Constant(albedo) => *albedo,
            Self::Dyn(s) => Albedo::clamp(s.lookup_at(intersection)),
        }
    }
}

impl<T> From<T> for DynAlbedoTexture
where
    T: Into<DynTexture>,
{
    fn from(value: T) -> Self {
        match value.into() {
            DynTexture::Constant(albedo) => Self::Constant(Albedo::clamp(albedo.value())),
            texture => Self::Dyn(Box::new(texture)),
        }
    }
}
