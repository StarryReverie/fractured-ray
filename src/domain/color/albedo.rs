use std::ops::Mul;

use snafu::prelude::*;

use crate::domain::math::{algebra::Vector, numeric::Val};

use super::Color;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Albedo(Color);

impl Albedo {
    pub const BLACK: Self = Self(Color::BLACK);
    pub const RED: Self = Self(Color::RED);
    pub const GREEN: Self = Self(Color::GREEN);
    pub const BLUE: Self = Self(Color::BLUE);
    pub const YELLOW: Self = Self(Color::YELLOW);
    pub const MAGENTA: Self = Self(Color::MAGENTA);
    pub const CYAN: Self = Self(Color::CYAN);
    pub const WHITE: Self = Self(Color::WHITE);

    pub fn new(red: Val, green: Val, blue: Val) -> Result<Self, TryNewAlbedoError> {
        let range = Val(0.0)..=Val(1.0);
        ensure!(range.contains(&red), InvalidComponentSnafu);
        ensure!(range.contains(&green), InvalidComponentSnafu);
        ensure!(range.contains(&blue), InvalidComponentSnafu);
        Ok(Self(Color::new(red, green, blue)))
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
    pub fn to_color(&self) -> Color {
        self.0
    }

    #[inline]
    pub fn to_vector(&self) -> Vector {
        self.0.to_vector()
    }
}

impl From<Albedo> for Color {
    #[inline]
    fn from(value: Albedo) -> Self {
        value.to_color()
    }
}

impl From<Color> for Albedo {
    fn from(value: Color) -> Self {
        Self(Color::new(
            value.red().clamp(Val(0.0), Val(1.0)),
            value.green().clamp(Val(0.0), Val(1.0)),
            value.blue().clamp(Val(0.0), Val(1.0)),
        ))
    }
}

impl Mul<Val> for Albedo {
    type Output = Color;

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

impl Mul<Color> for Albedo {
    type Output = Color;

    #[inline]
    fn mul(self, rhs: Color) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<Albedo> for Color {
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
