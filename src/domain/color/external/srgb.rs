use getset::CopyGetters;

use crate::domain::color::core::Spectrum;
use crate::domain::math::numeric::Val;

#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SRgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl SRgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn encode_gamma(linear: Val) -> Val {
        if linear <= Val(0.0031308) {
            Val(12.92) * linear
        } else {
            linear.powf(Val(1.0 / 2.4)).mul_add(Val(1.055), Val(-0.055))
        }
    }

    fn decode_gamma(srgb: Val) -> Val {
        if srgb <= Val(0.04045) {
            srgb / Val(12.92)
        } else {
            ((srgb + Val(0.055)) / Val(1.055)).powf(Val(2.4))
        }
    }
}

impl From<Spectrum> for SRgbColor {
    fn from(value: Spectrum) -> Self {
        let red = Val(256.0) * Self::encode_gamma(value.red()).clamp(Val(0.0), Val(0.999));
        let green = Val(256.0) * Self::encode_gamma(value.green()).clamp(Val(0.0), Val(0.999));
        let blue = Val(256.0) * Self::encode_gamma(value.blue()).clamp(Val(0.0), Val(0.999));
        SRgbColor {
            red: red.into(),
            green: green.into(),
            blue: blue.into(),
        }
    }
}

impl From<SRgbColor> for Spectrum {
    fn from(value: SRgbColor) -> Self {
        let red = SRgbColor::decode_gamma(Val::from(value.red) / Val(255.0));
        let green = SRgbColor::decode_gamma(Val::from(value.green) / Val(255.0));
        let blue = SRgbColor::decode_gamma(Val::from(value.blue) / Val(255.0));
        Spectrum::new(red, green, blue)
    }
}
