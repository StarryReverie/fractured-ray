use std::ops::{Add, Mul};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

impl Color {
    pub const BLACK: Self = Color::new(0.0, 0.0, 0.0);
    pub const RED: Self = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Self = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Self = Color::new(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Color::new(1.0, 1.0, 0.0);
    pub const MAGENTA: Self = Color::new(1.0, 0.0, 1.0);
    pub const CYAN: Self = Color::new(0.0, 1.0, 1.0);
    pub const WHITE: Self = Color::new(1.0, 1.0, 1.0);

    pub const fn new(red: f64, green: f64, blue: f64) -> Self {
        Self {
            red: red.max(0.0),
            green: green.max(0.0),
            blue: blue.max(0.0),
        }
    }

    pub fn red(&self) -> f64 {
        self.red
    }

    pub fn green(&self) -> f64 {
        self.green
    }

    pub fn blue(&self) -> f64 {
        self.blue
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

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(self * rhs.red, self * rhs.green, self * rhs.blue)
    }
}
