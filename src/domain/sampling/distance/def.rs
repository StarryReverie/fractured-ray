use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};

pub trait DistanceSampling: Send + Sync {
    fn sample_distance(
        &self,
        ray: &Ray,
        segment: &RaySegment,
        rng: &mut dyn RngCore,
    ) -> DistanceSample;

    fn pdf_distance(&self, ray: &Ray, segment: &RaySegment, distance: Val) -> Val;
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct DistanceSample {
    #[getset(get = "pub")]
    scattering: RayScattering,
    #[getset(get_copy = "pub")]
    pdf: Val,
}

impl DistanceSample {
    pub fn new(scattering: RayScattering, pdf: Val) -> Self {
        Self { scattering, pdf }
    }

    #[inline]
    pub fn into_scattering(self) -> RayScattering {
        self.scattering
    }

    #[inline]
    pub fn distance(&self) -> Val {
        self.scattering.distance()
    }

    #[inline]
    pub fn position(&self) -> Point {
        self.scattering.position()
    }
}
