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

    fn encode_gamma(linear: f64) -> f64 {
        linear.sqrt()
    }
}

impl From<Color> for ExternalColor {
    fn from(value: Color) -> Self {
        ExternalColor {
            red: (256.0 * Self::encode_gamma(value.red()).clamp(0.0, 0.999)) as u8,
            green: (256.0 * Self::encode_gamma(value.green()).clamp(0.0, 0.999)) as u8,
            blue: (256.0 * Self::encode_gamma(value.blue()).clamp(0.0, 0.999)) as u8,
        }
    }
}
