use getset::CopyGetters;

use crate::domain::color::Spectrum;
use crate::domain::math::geometry::Point;
use crate::domain::texture::def::{Texture, TextureKind, UvCoordinate};

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
    fn lookup(&self, _position: Point, _uv: Option<UvCoordinate>) -> Spectrum {
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
