use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use getset::CopyGetters;

use crate::domain::math::numeric::Val;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Spectrum {
    red: Val,
    green: Val,
    blue: Val,
}

impl Spectrum {
    #[inline]
    pub const fn new(red: Val, green: Val, blue: Val) -> Self {
        Self {
            red: red.max(Val(0.0)),
            green: green.max(Val(0.0)),
            blue: blue.max(Val(0.0)),
        }
    }

    #[inline]
    pub const fn broadcast(val: Val) -> Self {
        Self::new(val, val, val)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::broadcast(Val(0.0))
    }

    #[inline]
    pub fn lerp(a: Self, b: Self, t: Val) -> Self {
        a * (Val(1.0) - t) + b * t
    }

    #[inline]
    pub fn norm(&self) -> Val {
        (self.red.powi(2) + self.green.powi(2) + self.blue.powi(2)).sqrt()
    }

    pub fn channel(&self, index: usize) -> Val {
        match index {
            0 => self.red,
            1 => self.green,
            2 => self.blue,
            _ => panic!("channel index out of range"),
        }
    }
}

impl Add for Spectrum {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl AddAssign for Spectrum {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Spectrum {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red - rhs.red,
            self.green - rhs.green,
            self.blue - rhs.blue,
        )
    }
}

impl SubAssign for Spectrum {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Spectrum {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        )
    }
}

impl MulAssign for Spectrum {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Val> for Spectrum {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Val) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl Mul<Spectrum> for Val {
    type Output = <Spectrum as Mul<Val>>::Output;

    #[inline]
    fn mul(self, rhs: Spectrum) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<Val> for Spectrum {
    #[inline]
    fn mul_assign(&mut self, rhs: Val) {
        *self = *self * rhs;
    }
}

impl Div<Val> for Spectrum {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Val) -> Self::Output {
        Self::new(self.red / rhs, self.green / rhs, self.blue / rhs)
    }
}

impl DivAssign<Val> for Spectrum {
    #[inline]
    fn div_assign(&mut self, rhs: Val) {
        *self = *self / rhs;
    }
}

impl Sum for Spectrum {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Spectrum::zero(), |sum, x| sum + x)
    }
}
