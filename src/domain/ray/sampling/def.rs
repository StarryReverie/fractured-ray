use std::fmt::Debug;

use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::geometry::{AllTransformation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

pub trait CoefSampling: Debug + Send + Sync {
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

pub trait LightSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<&dyn Shape>;

    fn light_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample>;

    fn light_pdf(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightSample {
    ray: Ray,
    coefficient: Val,
    pdf: Val,
    shape_id: ShapeId,
}

impl LightSample {
    pub fn new(ray: Ray, coefficient: Val, pdf: Val, shape_id: ShapeId) -> Self {
        Self {
            ray,
            coefficient,
            pdf,
            shape_id,
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

    pub fn shape_id(&self) -> ShapeId {
        self.shape_id
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            coefficient: self.coefficient / multiplier,
            pdf: self.pdf * multiplier,
            ..self
        }
    }
}

impl Transform<AllTransformation> for LightSample {
    fn transform(&self, transformation: &AllTransformation) -> Self {
        LightSample::new(
            self.ray.transform(transformation),
            self.coefficient,
            self.pdf,
            self.shape_id,
        )
    }
}
