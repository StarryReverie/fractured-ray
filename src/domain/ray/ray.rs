use crate::domain::{
    color::Color,
    geometry::{Point, UnitVector},
};

use super::RayTrace;

#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    trace: RayTrace,
    color: Color,
}

impl Ray {
    pub fn new(trace: RayTrace, color: Color) -> Self {
        Self { trace, color }
    }

    pub fn start(&self) -> Point {
        self.trace.start()
    }

    pub fn direction(&self) -> UnitVector {
        self.trace.direction()
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn at(&self, distance: f32) -> Point {
        self.trace.at(distance)
    }
}
