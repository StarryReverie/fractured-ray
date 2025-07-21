use std::any::TypeId;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

pub type WrappedVal = f64;

#[derive(Debug, Default, Clone, Copy)]
pub struct Val(pub WrappedVal);

impl Val {
    pub const PRECISION: WrappedVal = 1e-8;
    pub const PI: Self = Self(std::f64::consts::PI);
    pub const FRAC_1_PI: Self = Self(std::f64::consts::FRAC_1_PI);
    pub const INFINITY: Self = Self(f64::INFINITY);

    #[inline(always)]
    pub fn mul_add(self, a: Self, b: Self) -> Self {
        Val(self.0.mul_add(a.0, b.0))
    }

    #[inline(always)]
    pub const fn abs(self) -> Self {
        Val(self.0.abs())
    }

    #[inline(always)]
    pub fn sqrt(self) -> Self {
        Val(self.0.sqrt())
    }

    #[inline(always)]
    pub fn exp(self) -> Self {
        Val(self.0.exp())
    }

    #[inline(always)]
    pub fn exp2(self) -> Self {
        Val(self.0.exp2())
    }

    #[inline(always)]
    pub fn exp_m1(self) -> Self {
        Val(self.0.exp_m1())
    }

    #[inline(always)]
    pub fn ln(self) -> Self {
        Val(self.0.ln())
    }

    #[inline(always)]
    pub fn log10(self) -> Self {
        Val(self.0.log10())
    }

    #[inline(always)]
    pub fn log2(self) -> Self {
        Val(self.0.log2())
    }

    #[inline(always)]
    pub fn powf(self, n: Self) -> Self {
        Val(self.0.powf(n.0))
    }

    #[inline(always)]
    pub fn powi(self, n: i32) -> Self {
        Val(self.0.powi(n))
    }

    #[inline(always)]
    pub fn div_euclid(self, rhs: Self) -> Self {
        Val(self.0.div_euclid(rhs.0))
    }

    #[inline(always)]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        Val(self.0.rem_euclid(rhs.0))
    }

    #[inline(always)]
    pub fn sin(self) -> Self {
        Val(self.0.sin())
    }

    #[inline(always)]
    pub fn cos(self) -> Self {
        Val(self.0.cos())
    }

    #[inline(always)]
    pub fn sin_cos(self) -> (Self, Self) {
        let res = self.0.sin_cos();
        (Val(res.0), Val(res.1))
    }

    #[inline(always)]
    pub fn tan(self) -> Self {
        Val(self.0.tan())
    }

    #[inline(always)]
    pub fn asin(self) -> Self {
        Val(self.0.asin())
    }

    #[inline(always)]
    pub fn acos(self) -> Self {
        Val(self.0.acos())
    }

    #[inline(always)]
    pub fn atan(self) -> Self {
        Val(self.0.atan())
    }

    #[inline(always)]
    pub fn atan2(self, other: Self) -> Self {
        Val(self.0.atan2(other.0))
    }

    #[inline(always)]
    pub fn sinh(self) -> Self {
        Val(self.0.sinh())
    }

    #[inline(always)]
    pub fn cosh(self) -> Self {
        Val(self.0.cosh())
    }

    #[inline(always)]
    pub fn tanh(self) -> Self {
        Val(self.0.tanh())
    }

    #[inline(always)]
    pub fn asinh(self) -> Self {
        Val(self.0.asinh())
    }

    #[inline(always)]
    pub fn acosh(self) -> Self {
        Val(self.0.acosh())
    }

    #[inline(always)]
    pub fn atanh(self) -> Self {
        Val(self.0.atanh())
    }

    #[inline(always)]
    pub const fn recip(self) -> Self {
        Val(self.0.recip())
    }

    #[inline(always)]
    pub const fn to_degrees(self) -> Self {
        Val(self.0.to_degrees())
    }

    #[inline(always)]
    pub const fn to_radians(self) -> Self {
        Val(self.0.to_radians())
    }

    #[inline(always)]
    pub fn floor(self) -> Self {
        Val(self.0.floor())
    }

    #[inline(always)]
    pub fn ceil(self) -> Self {
        Val(self.0.ceil())
    }

    #[inline(always)]
    pub fn round(self) -> Self {
        Val(self.0.round())
    }

    #[inline(always)]
    pub fn trunc(self) -> Self {
        Val(self.0.trunc())
    }

    #[inline(always)]
    pub fn fract(self) -> Self {
        Val(self.0.fract())
    }

    #[inline(always)]
    pub const fn signum(self) -> Self {
        Val(self.0.signum())
    }

    #[inline(always)]
    pub const fn max(self, other: Self) -> Self {
        Val(self.0.max(other.0))
    }

    #[inline(always)]
    pub const fn min(self, other: Self) -> Self {
        Val(self.0.min(other.0))
    }

    #[inline(always)]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Val(self.0.clamp(min.0, max.0))
    }

    #[inline(always)]
    pub const fn midpoint(self, other: Self) -> Self {
        Val(self.0.midpoint(other.0))
    }

    #[inline(always)]
    pub fn hypot(self, other: Self) -> Self {
        Val(self.0.hypot(other.0))
    }

    #[inline(always)]
    pub const fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    #[inline(always)]
    pub const fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }

    #[inline(always)]
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    #[inline(always)]
    pub const fn is_normal(self) -> bool {
        self.0.is_normal()
    }

    #[inline(always)]
    pub const fn is_sign_positive(self) -> bool {
        self.0.is_sign_positive()
    }

    #[inline(always)]
    pub const fn is_sign_negative(self) -> bool {
        self.0.is_sign_negative()
    }

    #[inline(always)]
    pub fn lerp(a: Self, b: Self, t: Self) -> Self {
        a * (Val(1.0) - t) + b * t
    }
}

impl PartialEq for Val {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Val {}

impl PartialOrd for Val {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Val {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let delta = self.0 - other.0;
        match (delta <= Self::PRECISION, delta >= -Self::PRECISION) {
            (false, false) => self.0.total_cmp(&other.0),
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (true, true) => Ordering::Equal,
        }
    }
}

impl Hash for Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = self.0.to_bits();
        state.write_u64(bytes);
        TypeId::of::<Val>().hash(state);
    }
}

impl Add for Val {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Val(self.0 + rhs.0)
    }
}

impl AddAssign for Val {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Val {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Val(self.0 - rhs.0)
    }
}

impl SubAssign for Val {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Val {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Val(self.0 * rhs.0)
    }
}

impl MulAssign for Val {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div for Val {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Val(self.0 / rhs.0)
    }
}

impl DivAssign for Val {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl Neg for Val {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Val(-self.0)
    }
}

impl Sum for Val {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Val(0.0), |sum, x| sum + x)
    }
}

impl Product for Val {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Val(1.0), |prod, x| prod * x)
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<WrappedVal> for Val {
    fn from(value: WrappedVal) -> Self {
        Self(value)
    }
}

impl From<Val> for WrappedVal {
    fn from(value: Val) -> Self {
        value.0
    }
}

macro_rules! impl_primitive_conversions_for_val {
    ($t:ty) => {
        impl From<$t> for Val {
            fn from(value: $t) -> Self {
                Self(value as WrappedVal)
            }
        }

        impl From<Val> for $t {
            fn from(value: Val) -> Self {
                value.0 as $t
            }
        }
    };
}

impl_primitive_conversions_for_val!(i8);
impl_primitive_conversions_for_val!(i16);
impl_primitive_conversions_for_val!(i32);
impl_primitive_conversions_for_val!(i64);
impl_primitive_conversions_for_val!(i128);
impl_primitive_conversions_for_val!(isize);
impl_primitive_conversions_for_val!(u8);
impl_primitive_conversions_for_val!(u16);
impl_primitive_conversions_for_val!(u32);
impl_primitive_conversions_for_val!(u64);
impl_primitive_conversions_for_val!(u128);
impl_primitive_conversions_for_val!(usize);

impl FromStr for Val {
    type Err = <WrappedVal as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Val(WrappedVal::from_str(s)?))
    }
}
