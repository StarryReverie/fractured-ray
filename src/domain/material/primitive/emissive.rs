use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::numeric::Val;
use crate::domain::ray::sampling::{CoefSample, CoefSampling};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct Emissive {
    radiance: Color,
}

impl Emissive {
    pub fn new(radiance: Color) -> Self {
        Self { radiance }
    }
}

impl Material for Emissive {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Emissive
    }

    fn albedo(&self) -> Color {
        Color::WHITE
    }

    fn bsdf(&self, _ray: &Ray, _intersectionn: &RayIntersection, _ray_next: &Ray) -> Val {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }

    fn shade(
        &self,
        _context: &mut Context<'_>,
        _ray: Ray,
        _intersection: RayIntersection,
        _depth: usize,
    ) -> Color {
        self.radiance
    }

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefSampling for Emissive {
    fn coef_sample(
        &self,
        _ray: &Ray,
        _intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> CoefSample {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }

    fn coef_pdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        unimplemented!("rays should not bounce again if hitting an emissive material")
    }
}
