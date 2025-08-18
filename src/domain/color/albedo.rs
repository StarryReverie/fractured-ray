use std::ops::Mul;

use snafu::prelude::*;

use crate::domain::math::numeric::Val;

use super::Spectrum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Albedo(Spectrum);

impl Albedo {
    pub const BLACK: Self = Self(Spectrum::new(Val(0.0), Val(0.0), Val(0.0)));
    pub const RED: Self = Self(Spectrum::new(Val(1.0), Val(0.0), Val(0.0)));
    pub const GREEN: Self = Self(Spectrum::new(Val(0.0), Val(1.0), Val(0.0)));
    pub const BLUE: Self = Self(Spectrum::new(Val(0.0), Val(0.0), Val(1.0)));
    pub const YELLOW: Self = Self(Spectrum::new(Val(1.0), Val(1.0), Val(0.0)));
    pub const MAGENTA: Self = Self(Spectrum::new(Val(1.0), Val(0.0), Val(1.0)));
    pub const CYAN: Self = Self(Spectrum::new(Val(0.0), Val(1.0), Val(1.0)));
    pub const WHITE: Self = Self(Spectrum::new(Val(1.0), Val(1.0), Val(1.0)));

    pub fn new(red: Val, green: Val, blue: Val) -> Result<Self, TryNewAlbedoError> {
        let range = Val(0.0)..=Val(1.0);
        ensure!(range.contains(&red), InvalidComponentSnafu);
        ensure!(range.contains(&green), InvalidComponentSnafu);
        ensure!(range.contains(&blue), InvalidComponentSnafu);
        Ok(Self(Spectrum::new(red, green, blue)))
    }

    pub fn clamp(spectrum: Spectrum) -> Self {
        Self(Spectrum::new(
            spectrum.red().clamp(Val(0.0), Val(1.0)),
            spectrum.green().clamp(Val(0.0), Val(1.0)),
            spectrum.blue().clamp(Val(0.0), Val(1.0)),
        ))
    }

    #[inline]
    pub fn broadcast(val: Val) -> Result<Self, TryNewAlbedoError> {
        Self::new(val, val, val)
    }

    #[inline]
    pub fn red(&self) -> Val {
        self.0.red()
    }

    #[inline]
    pub fn green(&self) -> Val {
        self.0.green()
    }

    #[inline]
    pub fn blue(&self) -> Val {
        self.0.blue()
    }

    #[inline]
    pub fn to_spectrum(&self) -> Spectrum {
        self.0
    }
}

impl From<Albedo> for Spectrum {
    #[inline]
    fn from(value: Albedo) -> Self {
        value.to_spectrum()
    }
}

impl From<Spectrum> for Albedo {
    fn from(value: Spectrum) -> Self {
        Self::clamp(value)
    }
}

impl Mul<Val> for Albedo {
    type Output = Spectrum;

    #[inline]
    fn mul(self, rhs: Val) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<Albedo> for Val {
    type Output = <Albedo as Mul<Self>>::Output;

    #[inline]
    fn mul(self, rhs: Albedo) -> Self::Output {
        rhs * self
    }
}

impl Mul<Spectrum> for Albedo {
    type Output = Spectrum;

    #[inline]
    fn mul(self, rhs: Spectrum) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<Albedo> for Spectrum {
    type Output = <Albedo as Mul<Self>>::Output;

    #[inline]
    fn mul(self, rhs: Albedo) -> Self::Output {
        rhs * self
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewAlbedoError {
    #[snafu(display("albedo's components should be in [0, 1]"))]
    InvalidComponent,
}
