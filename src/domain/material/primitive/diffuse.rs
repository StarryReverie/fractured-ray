use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::numeric::{Val, WrappedVal};
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
            let (x, y, z) = rng.random::<(WrappedVal, WrappedVal, WrappedVal)>();
            let (x, y, z) = (Val(x * 2.0 - 1.0), Val(y * 2.0 - 1.0), Val(z * 2.0 - 1.0));
            if let Ok(unit) = Vector::new(x, y, z).normalize() {
                if let Ok(direction) = (normal + unit).normalize() {
                    break direction;
                }
            }
        };

        let ray_next = Ray::new(intersection.position(), direction);
        let pdf = self.coef_pdf(&ray, &intersection, &ray_next);
        CoefSample::new(ray_next, Val(1.0), pdf)
    }

    fn coef_pdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        ray_next.direction().dot(intersection.normal())
    }
}
