use rand::prelude::*;

use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};

use super::{DistanceSample, DistanceSampling};

#[derive(Debug, Clone)]
pub struct ExponentialDistanceSampler {
    sigma: Val,
}

impl ExponentialDistanceSampler {
    pub fn new(sigma: Val) -> Self {
        Self { sigma }
    }
}

impl DistanceSampling for ExponentialDistanceSampler {
    fn sample_distance(
        &self,
        ray: &Ray,
        segment: &RaySegment,
        rng: &mut dyn RngCore,
    ) -> DistanceSample {
        let sigma = self.sigma;
        let (start, length) = (segment.start(), segment.length());

        let u = Val(rng.random());
        let distance = start - (-sigma * length).exp_m1().mul_add(u, Val(1.0)).ln() / sigma;
        let position = ray.at(distance);

        let scattering = RayScattering::new(distance, position);
        let pdf = self.pdf_distance(segment, distance);
        DistanceSample::new(scattering, pdf)
    }

    fn pdf_distance(&self, segment: &RaySegment, distance: Val) -> Val {
        let sigma = self.sigma;
        let (start, length) = (segment.start(), segment.length());

        if (start..=(start + length)).contains(&distance) {
            if length != Val(0.0) {
                let num = -sigma * (-sigma * (distance - start)).exp();
                let den = (-sigma * length).exp_m1();
                num / den
            } else {
                Val(1.0)
            }
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_distance_sampler_pdf_distance_succeeds() {
        let sampler = ExponentialDistanceSampler::new(Val(0.1));

        let segment = RaySegment::new(Val(1.0), Val(4.0));
        assert_eq!(sampler.pdf_distance(&segment, Val(2.0)), Val(0.27445933));
        assert_eq!(sampler.pdf_distance(&segment, Val(6.0)), Val(0.0));

        let segment = RaySegment::new(Val(1.0), Val(0.0));
        assert_eq!(sampler.pdf_distance(&segment, Val(1.0)), Val(1.0));
        assert_eq!(sampler.pdf_distance(&segment, Val(2.0)), Val(0.0));
    }
}
