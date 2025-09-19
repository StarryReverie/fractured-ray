use rand::prelude::*;

use crate::domain::math::geometry::Distance;
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
        let distance = Distance::new(distance).unwrap();
        let position = ray.at(distance);

        let scattering = RayScattering::new(distance, position);
        let pdf = self.pdf_distance(ray, segment, distance);
        DistanceSample::new(scattering, pdf)
    }

    fn pdf_distance(&self, _ray: &Ray, segment: &RaySegment, distance: Distance) -> Val {
        let sigma = self.sigma;
        let (start, length) = (segment.start(), segment.length());

        if segment.contains(distance) {
            if length != Distance::zero() {
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
    use crate::domain::math::geometry::{Direction, Point};

    use super::*;

    #[test]
    fn exponential_distance_sampler_pdf_distance_succeeds() {
        let sampler = ExponentialDistanceSampler::new(Val(0.1));
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Direction::x_direction(),
        );

        let segment = RaySegment::new(
            Distance::new(Val(1.0)).unwrap(),
            Distance::new(Val(4.0)).unwrap(),
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(2.0)).unwrap()),
            Val(0.27445933)
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(6.0)).unwrap()),
            Val(0.0)
        );

        let segment = RaySegment::new(
            Distance::new(Val(1.0)).unwrap(),
            Distance::new(Val(0.0)).unwrap(),
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(1.0)).unwrap()),
            Val(1.0)
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(2.0)).unwrap()),
            Val(0.0)
        );
    }
}
