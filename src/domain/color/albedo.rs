use std::ops::Mul;

use snafu::prelude::*;

use crate::domain::math::{algebra::Vector, numeric::Val};

use super::Spectrum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Albedo(Spectrum);

impl Albedo {
    pub const BLACK: Self = Self(Spectrum::BLACK);
    pub const RED: Self = Self(Spectrum::RED);
    pub const GREEN: Self = Self(Spectrum::GREEN);
    pub const BLUE: Self = Self(Spectrum::BLUE);
    pub const YELLOW: Self = Self(Spectrum::YELLOW);
    pub const MAGENTA: Self = Self(Spectrum::MAGENTA);
    pub const CYAN: Self = Self(Spectrum::CYAN);
    pub const WHITE: Self = Self(Spectrum::WHITE);

    pub fn new(red: Val, green: Val, blue: Val) -> Result<Self, TryNewAlbedoError> {
        let range = Val(0.0)..=Val(1.0);
        ensure!(range.contains(&red), InvalidComponentSnafu);
        ensure!(range.contains(&green), InvalidComponentSnafu);
        ensure!(range.contains(&blue), InvalidComponentSnafu);
        Ok(Self(Spectrum::new(red, green, blue)))
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

    #[inline]
    pub fn to_vector(&self) -> Vector {
        self.0.to_vector()
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
        Self(Spectrum::new(
            value.red().clamp(Val(0.0), Val(1.0)),
            value.green().clamp(Val(0.0), Val(1.0)),
            value.blue().clamp(Val(0.0), Val(1.0)),
        ))
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
