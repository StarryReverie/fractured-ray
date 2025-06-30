use std::ops::Bound;

use crate::domain::geometry::{Point, UnitVector};
use crate::domain::ray::RayTrace;

pub type DisRange = (Bound<f32>, Bound<f32>);

pub trait Shape {
    fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RayIntersection {
    distance: f32,
    position: Point,
    normal: UnitVector,
    side: SurfaceSide,
}

impl RayIntersection {
    pub fn new(distance: f32, position: Point, normal: UnitVector, side: SurfaceSide) -> Self {
        Self {
            distance,
            position,
            normal,
            side,
        }
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn side(&self) -> SurfaceSide {
        self.side
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceSide {
    Front,
    Back,
}
