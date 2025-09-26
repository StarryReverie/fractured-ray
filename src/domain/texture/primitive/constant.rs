use getset::CopyGetters;

use crate::domain::color::core::Spectrum;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind};

#[derive(Debug, Clone, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Constant {
    value: Spectrum,
}

impl Constant {
    #[inline]
    pub fn new<S>(value: S) -> Self
    where
        S: Into<Spectrum>,
    {
        Self {
            value: value.into(),
        }
    }
}

impl Texture for Constant {
    #[inline]
    fn kind(&self) -> TextureKind {
        TextureKind::Constant
    }

    #[inline]
    fn lookup(&self, _intersection: &RayIntersection) -> Spectrum {
        self.value
    }
}

impl<S> From<S> for Constant
where
    S: Into<Spectrum>,
{
    #[inline]
    fn from(value: S) -> Self {
        Self::new(value)
    }
}
