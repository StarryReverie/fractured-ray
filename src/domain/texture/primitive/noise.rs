use std::sync::Arc;

use snafu::prelude::*;

use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;
use crate::domain::ray::event::RayIntersection;
use crate::domain::texture::def::{Texture, TextureKind};
use crate::domain::texture::noise::NoiseGenerator;

#[derive(Debug, Clone)]
pub struct Noise {
    generator: Arc<dyn NoiseGenerator>,
    frequency: Val,
}

impl Noise {
    pub fn new<NG>(generator: NG, scale: Val) -> Result<Self, TryNewNoiseError>
    where
        NG: NoiseGenerator + 'static,
    {
        ensure!(scale > Val(0.0), NonPositiveScaleSnafu);
        Ok(Self {
            generator: Arc::new(generator),
            frequency: scale.recip(),
        })
    }
}

impl Texture for Noise {
    fn kind(&self) -> TextureKind {
        TextureKind::Noise
    }

    fn lookup(&self, intersection: &RayIntersection) -> Spectrum {
        let position = (intersection.position().into_vector() * self.frequency).into();
        let value = self.generator.evaluate(position);
        Spectrum::broadcast(value * Val(0.5) + Val(0.5))
    }
}

impl PartialEq for Noise {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.generator, &other.generator)
    }
}

impl Eq for Noise {}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewNoiseError {
    #[snafu(display("scale of the noise texture should be positive"))]
    NonPositiveScale,
}
