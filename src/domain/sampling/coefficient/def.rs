use std::fmt::Debug;

use rand::prelude::*;

use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};

pub trait CoefficientSampling: Debug + Send + Sync {
    fn sample_coefficient(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> CoefficientSample;

    fn pdf_coefficient(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoefficientSample {
    ray_next: Ray,
    coefficient: Vector,
    pdf: Val,
}

impl CoefficientSample {
    pub fn new(ray_next: Ray, coefficient: Vector, pdf: Val) -> Self {
        Self {
            ray_next,
            coefficient,
            pdf,
        }
    }

    pub fn ray_next(&self) -> &Ray {
        &self.ray_next
    }

    pub fn into_ray_next(self) -> Ray {
        self.ray_next
    }

    pub fn coefficient(&self) -> Vector {
        self.coefficient
    }

    pub fn pdf(&self) -> Val {
        self.pdf
    }
}
