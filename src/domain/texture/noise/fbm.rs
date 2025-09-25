use snafu::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;

use super::NoiseGenerator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FbmNoiseGenerator<NG>
where
    NG: NoiseGenerator,
{
    generator: NG,
    octaves: usize,
    lacunarity: Val,
    gain: Val,
}

impl<NG> FbmNoiseGenerator<NG>
where
    NG: NoiseGenerator,
{
    pub fn new(
        generator: NG,
        octaves: usize,
        lacunarity: Val,
        gain: Val,
    ) -> Result<Self, TryNewFbmNoiseGeneratorError> {
        ensure!(octaves > 0, InvalidOctavesSnafu);
        ensure!(lacunarity > Val(1.0), InvalidLacunaritySnafu);
        ensure!(Val(0.0) < gain && gain < Val(1.0), InvalidGainSnafu);

        Ok(Self {
            generator,
            octaves,
            lacunarity,
            gain,
        })
    }
}

impl<NG> NoiseGenerator for FbmNoiseGenerator<NG>
where
    NG: NoiseGenerator,
{
    fn evaluate(&self, mut position: Point) -> Val {
        let mut res = Val(0.0);
        let mut amplitude = self.gain;
        for _ in 0..self.octaves {
            res += amplitude * self.generator.evaluate(position);
            position = (position.into_vector() * self.lacunarity).into();
            amplitude *= self.gain;
        }
        res.clamp(Val(-1.0), Val(1.0))
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewFbmNoiseGeneratorError {
    #[snafu(display("octaves should be positive"))]
    InvalidOctaves,
    #[snafu(display("lacunarity should be greater than 1"))]
    InvalidLacunarity,
    #[snafu(display("gain should be in (0, 1)"))]
    InvalidGain,
}
