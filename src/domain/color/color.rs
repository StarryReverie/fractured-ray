use std::ops::{Add, Mul};

use getset::CopyGetters;

use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::Val;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Color {
    red: Val,
    green: Val,
    blue: Val,
}

impl Color {
    pub const BLACK: Self = Color::new(Val(0.0), Val(0.0), Val(0.0));
    pub const RED: Self = Color::new(Val(1.0), Val(0.0), Val(0.0));
    pub const GREEN: Self = Color::new(Val(0.0), Val(1.0), Val(0.0));
    pub const BLUE: Self = Color::new(Val(0.0), Val(0.0), Val(1.0));
    pub const YELLOW: Self = Color::new(Val(1.0), Val(1.0), Val(0.0));
    pub const MAGENTA: Self = Color::new(Val(1.0), Val(0.0), Val(1.0));
    pub const CYAN: Self = Color::new(Val(0.0), Val(1.0), Val(1.0));
    pub const WHITE: Self = Color::new(Val(1.0), Val(1.0), Val(1.0));

    pub const fn new(red: Val, green: Val, blue: Val) -> Self {
        Self {
            red: red.max(Val(0.0)),
            green: green.max(Val(0.0)),
            blue: blue.max(Val(0.0)),
        }
    }

    pub fn to_vector(&self) -> Vector {
        (*self).into()
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        )
    }
}

impl Mul<Val> for Color {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl Mul<Color> for Val {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(self * rhs.red, self * rhs.green, self * rhs.blue)
    }
}

impl Mul<Vector> for Color {
    type Output = Color;

    fn mul(self, rhs: Vector) -> Self::Output {
        Self::new(
            self.red * rhs.x(),
            self.green * rhs.y(),
            self.blue * rhs.z(),
        )
    }
}

impl Mul<Color> for Vector {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(
            self.x() * rhs.red,
            self.y() * rhs.green,
            self.z() * rhs.blue,
        )
    }
}

impl From<Vector> for Color {
    fn from(value: Vector) -> Self {
        Self::new(value.x(), value.y(), value.z())
    }
}

impl From<Color> for Vector {
    fn from(value: Color) -> Self {
        Self::new(value.red, value.green, value.blue)
    }
}
