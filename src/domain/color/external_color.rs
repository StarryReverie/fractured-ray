use getset::CopyGetters;

use crate::domain::math::numeric::Val;

use super::Spectrum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ExternalColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl ExternalColor {
    fn encode_gamma(linear: Val) -> Val {
        if linear <= Val(0.0031308) {
            Val(12.92) * linear
        } else {
            linear.powf(Val(1.0 / 2.4)).mul_add(Val(1.055), Val(-0.055))
        }
    }
}

impl From<Spectrum> for ExternalColor {
    fn from(value: Spectrum) -> Self {
        let red = Val(256.0) * Self::encode_gamma(value.red()).clamp(Val(0.0), Val(0.999));
        let green = Val(256.0) * Self::encode_gamma(value.green()).clamp(Val(0.0), Val(0.999));
        let blue = Val(256.0) * Self::encode_gamma(value.blue()).clamp(Val(0.0), Val(0.999));
        ExternalColor {
            red: red.into(),
            green: green.into(),
            blue: blue.into(),
        }
    }
}
