use snafu::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::texture::def::{DynTexture, Texture, TextureKind, UvCoordinate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkerboard {
    texture0: Box<DynTexture>,
    texture1: Box<DynTexture>,
    frequency: Val,
}

impl Checkerboard {
    pub fn new<T0, T1>(
        texture0: T0,
        texture1: T1,
        scale: Val,
    ) -> Result<Self, TryNewCheckerboardError>
    where
        T0: Into<DynTexture>,
        T1: Into<DynTexture>,
    {
        ensure!(scale > Val(0.0), NonPositiveScaleSnafu);
        Ok(Self {
            texture0: Box::new(texture0.into()),
            texture1: Box::new(texture1.into()),
            frequency: scale.recip(),
        })
    }
}

impl Texture for Checkerboard {
    fn kind(&self) -> TextureKind {
        TextureKind::Checkerboard
    }

    fn lookup(&self, position: Point, uv: Option<UvCoordinate>) -> Spectrum {
        let x = usize::from((self.frequency * position.x()).floor());
        let y = usize::from((self.frequency * position.y()).floor());
        let z = usize::from((self.frequency * position.z()).floor());
        if (x + y + z) % 2 == 0 {
            self.texture0.lookup(position, uv)
        } else {
            self.texture1.lookup(position, uv)
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewCheckerboardError {
    #[snafu(display("scale of the checkerboard should be positive"))]
    NonPositiveScale,
}
