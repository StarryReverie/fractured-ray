use rand::prelude::*;

use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};

pub trait CoefSampling {
    fn coef_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> CoefSample;

    fn coef_pdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoefSample {
    ray: Ray,
    coefficient: Val,
    pdf: Val,
}

impl CoefSample {
    pub fn new(ray: Ray, coefficient: Val, pdf: Val) -> Self {
        Self {
            ray,
            coefficient,
            pdf,
        }
    }

    pub fn ray(&self) -> &Ray {
        &self.ray
    }

    pub fn into_ray(self) -> Ray {
        self.ray
    }

    pub fn coefficient(&self) -> Val {
        self.coefficient
    }

    pub fn pdf(&self) -> Val {
        self.pdf
    }
}
