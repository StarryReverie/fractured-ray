use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::CoefSampling;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Context;

pub trait Material: CoefSampling + Debug + Send + Sync + 'static {
    fn material_kind(&self) -> MaterialKind;

    fn albedo(&self) -> Color;

    fn bsdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val;

    fn shade<'a>(
        &self,
        context: &mut Context<'a>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let sample = self.coef_sample(&ray, &intersection, *context.rng());
        let coefficient = self.albedo() * sample.coefficient();
        let ray_next = sample.into_ray();

        let renderer = context.renderer();
        let radiance = renderer.trace(context, ray_next, DisRange::positive(), depth + 1);
        coefficient * radiance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    Diffuse,
    Emissive,
    Refractive,
    Scattering,
    Specular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId {
    kind: MaterialKind,
    index: u32,
}

impl MaterialId {
    pub fn new(kind: MaterialKind, index: u32) -> Self {
        Self { kind, index }
    }

    pub fn kind(&self) -> MaterialKind {
        self.kind
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub trait MaterialContainer: Debug + Send + Sync + 'static {
    fn add_material<M: Material>(&mut self, material: M) -> MaterialId;

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material>;
}
