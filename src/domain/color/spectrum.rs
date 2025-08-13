use std::ops::{Add, Mul};

use getset::CopyGetters;

use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::Val;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Spectrum {
    red: Val,
    green: Val,
    blue: Val,
}

impl Spectrum {
    pub const BLACK: Self = Spectrum::new(Val(0.0), Val(0.0), Val(0.0));
    pub const RED: Self = Spectrum::new(Val(1.0), Val(0.0), Val(0.0));
    pub const GREEN: Self = Spectrum::new(Val(0.0), Val(1.0), Val(0.0));
    pub const BLUE: Self = Spectrum::new(Val(0.0), Val(0.0), Val(1.0));
    pub const YELLOW: Self = Spectrum::new(Val(1.0), Val(1.0), Val(0.0));
    pub const MAGENTA: Self = Spectrum::new(Val(1.0), Val(0.0), Val(1.0));
    pub const CYAN: Self = Spectrum::new(Val(0.0), Val(1.0), Val(1.0));
    pub const WHITE: Self = Spectrum::new(Val(1.0), Val(1.0), Val(1.0));

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

impl Add for Spectrum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl Mul for Spectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        )
    }
}

impl Mul<Val> for Spectrum {
    type Output = Self;

    fn mul(self, rhs: Val) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl Mul<Spectrum> for Val {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        Spectrum::new(self * rhs.red, self * rhs.green, self * rhs.blue)
    }
}

impl Mul<Vector> for Spectrum {
    type Output = Spectrum;

    fn mul(self, rhs: Vector) -> Self::Output {
        Self::new(
            self.red * rhs.x(),
            self.green * rhs.y(),
            self.blue * rhs.z(),
        )
    }
}

impl Mul<Spectrum> for Vector {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        Spectrum::new(
            self.x() * rhs.red,
            self.y() * rhs.green,
            self.z() * rhs.blue,
        )
    }
}

impl From<Vector> for Spectrum {
    fn from(value: Vector) -> Self {
        Self::new(value.x(), value.y(), value.z())
    }
}

impl From<Spectrum> for Vector {
    fn from(value: Spectrum) -> Self {
        Self::new(value.red, value.green, value.blue)
    }
}
