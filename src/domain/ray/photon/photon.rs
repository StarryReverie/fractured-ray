use getset::{CopyGetters, Getters};

use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct PhotonRay {
    #[getset(get = "pub")]
    ray: Ray,
    #[getset(get_copy = "pub")]
    throughput: Vector,
}

impl PhotonRay {
    pub fn new(ray: Ray, throughput: Vector) -> Self {
        Self { ray, throughput }
    }

    pub fn start(&self) -> Point {
        self.ray.start()
    }

    pub fn direction(&self) -> UnitVector {
        self.ray.direction()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, CopyGetters)]
#[getset(get_copy = "pub")]
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
}
