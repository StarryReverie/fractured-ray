use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq)]
pub struct PhotonRay {
    ray: Ray,
    throughput: Vector,
}

impl PhotonRay {
    pub fn new(ray: Ray, throughput: Vector) -> Self {
        Self { ray, throughput }
    }

    pub fn as_ray(&self) -> &Ray {
        &self.ray
    }

    pub fn start(&self) -> Point {
        self.ray.start()
    }

    pub fn direction(&self) -> UnitVector {
        self.ray.direction()
    }

    pub fn throughput(&self) -> Vector {
        self.throughput
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Photon {
    position: Point,
    direction: UnitVector,
    throughput: Vector,
}

impl Photon {
    pub fn new(position: Point, direction: UnitVector, throughput: Vector) -> Self {
        Self {
            position,
            direction,
            throughput,
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn direction(&self) -> UnitVector {
        self.direction
    }

    pub fn throughput(&self) -> Vector {
        self.throughput
    }
}
