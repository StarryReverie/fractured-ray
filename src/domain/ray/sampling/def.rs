use std::fmt::Debug;

use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::{AllTransformation, Point, Transform};
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

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

pub trait PointSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<&dyn Shape>;

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample>;

    fn pdf_point(&self, point: Point) -> Val;

    fn pdf_point_checked_inside(&self, point: Point) -> Val;

    fn normal(&self, point: Point) -> UnitVector;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointSample {
    point: Point,
    normal: UnitVector,
    pdf: Val,
    shape_id: ShapeId,
}

impl PointSample {
    pub fn new(point: Point, normal: UnitVector, pdf: Val, shape_id: ShapeId) -> Self {
        Self {
            point,
            normal,
            pdf,
            shape_id,
        }
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn pdf(&self) -> Val {
        self.pdf
    }

    pub fn shape_id(&self) -> ShapeId {
        self.shape_id
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            pdf: self.pdf * multiplier,
            ..self
        }
    }
}

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

pub trait Sampleable {
    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>>;
}
