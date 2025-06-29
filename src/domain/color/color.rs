use std::ops::{Add, Mul};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
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

    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red: red.max(0.0),
            green: green.max(0.0),
            blue: blue.max(0.0),
        }
    }

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn green(&self) -> f32 {
        self.green
    }

    pub fn blue(&self) -> f32 {
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

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(self * rhs.red, self * rhs.green, self * rhs.blue)
    }
}
