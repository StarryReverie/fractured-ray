use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::sampling::{CoefSample, CoefSampling};
use crate::domain::ray::{Ray, RayIntersection};

#[derive(Debug, Clone, PartialEq)]
pub struct Diffuse {
    albedo: Color,
}

impl Diffuse {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Diffuse {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Diffuse
    }

    fn albedo(&self) -> Color {
        self.albedo
    }

    fn bsdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        if intersection.normal().dot(ray_next.direction()) > Val(0.0) {
            Val::FRAC_1_PI
        } else {
            Val(0.0)
        }
    }
}

impl CoefSampling for Diffuse {
    fn coef_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> CoefSample {
        let normal = intersection.normal();
        let direction = loop {
            let unit = UnitVector::random(rng);
            if let Ok(direction) = (normal + unit).normalize() {
                break direction;
            }
        };

        let ray_next = Ray::new(intersection.position(), direction);
        let pdf = self.coef_pdf(ray, intersection, &ray_next);
        CoefSample::new(ray_next, Val(1.0), pdf)
    }

    fn coef_pdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let cos = ray_next.direction().dot(intersection.normal());
        cos.max(Val(0.0)) * Val::FRAC_1_PI
    }
}
