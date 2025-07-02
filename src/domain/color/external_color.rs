use crate::domain::geometry::Val;

use super::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl ExternalColor {
    pub fn red(&self) -> u8 {
        self.red
    }

    pub fn green(&self) -> u8 {
        self.green
    }

    pub fn blue(&self) -> u8 {
        self.blue
    }

    fn encode_gamma(linear: Val) -> Val {
        if linear <= Val(0.0031308) {
            Val(12.92) * linear
        } else {
            linear.powf(Val(1.0 / 2.4)).mul_add(Val(1.055), Val(-0.055))
        }
    }
}

impl From<Color> for ExternalColor {
    fn from(value: Color) -> Self {
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
