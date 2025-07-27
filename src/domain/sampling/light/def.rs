use std::fmt::Debug;

use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::Vector;
use crate::domain::math::geometry::{AllTransformation, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

pub trait LightSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<&dyn Shape>;

    fn sample_light(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        material: &dyn Material,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample>;

    fn pdf_light(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightSample {
    ray_next: Ray,
    coefficient: Vector,
    pdf: Val,
    shape_id: ShapeId,
}

impl LightSample {
    pub fn new(ray_next: Ray, coefficient: Vector, pdf: Val, shape_id: ShapeId) -> Self {
        Self {
            ray_next,
            coefficient,
            pdf,
            shape_id,
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
            self.ray_next.transform(transformation),
            self.coefficient,
            self.pdf,
            self.shape_id,
        )
    }
}
