use getset::{CopyGetters, Getters};

use crate::domain::color::Spectrum;
use crate::domain::math::geometry::{Direction, Point};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct PhotonRay {
    #[getset(get = "pub")]
    ray: Ray,
    #[getset(get_copy = "pub")]
    throughput: Spectrum,
}

impl PhotonRay {
    pub fn new(ray: Ray, throughput: Spectrum) -> Self {
        Self { ray, throughput }
    }

    pub fn scale_throughput(self, multiplier: Val) -> Self {
        Self {
            throughput: self.throughput * multiplier,
            ..self
        }
    }

    pub fn start(&self) -> Point {
        self.ray.start()
    }

    pub fn direction(&self) -> Direction {
        self.ray.direction()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Photon {
    position: Point,
    direction: Direction,
    throughput: Spectrum,
}

impl Photon {
    pub fn new(position: Point, direction: Direction, throughput: Spectrum) -> Self {
        Self {
            position,
            direction,
            throughput,
        }
    }
}
