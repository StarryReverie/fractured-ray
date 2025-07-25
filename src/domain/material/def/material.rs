use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::CoefSampling;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Context;

pub trait Material: CoefSampling + Debug + Send + Sync + 'static {
    fn material_kind(&self) -> MaterialKind;

    fn bsdf(
        &self,
        dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Vector;

    fn shade(
        &self,
        context: &mut Context<'_>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let this = self.as_dyn();
        let radiance_light = shade_light(this, context, &ray, &intersection);
        let radiance_scattering = shade_scattering(this, context, &ray, &intersection, depth);
        radiance_light + radiance_scattering
    }

    fn as_dyn(&self) -> &dyn Material;
}

fn shade_light(
    material: &dyn Material,
    context: &mut Context<'_>,
    ray: &Ray,
    intersection: &RayIntersection,
) -> Color {
    let scene = context.scene();
    let lights = scene.get_lights();
    let res = lights.light_sample(ray, intersection, material, *context.rng());
    let Some(sample) = res else {
        return Color::BLACK;
    };

    let ray_next = sample.ray();
    let res = scene.test_intersection(ray_next, DisRange::positive(), sample.shape_id());
    let (intersection_next, light_material) = if let Some(res) = res {
        let intersection_next = res.0;
        let material_id = res.1.material_id();
        let light_material = scene.get_entities().get_material(material_id).unwrap();
        (intersection_next, light_material)
    } else {
        return Color::BLACK;
    };

    let pdf_light = sample.pdf();
    if pdf_light == Val(0.0) {
        return Color::BLACK;
    }
    let pdf_scattering = material.coef_pdf(ray, intersection, ray_next);
    let weight = pdf_light / (pdf_light + pdf_scattering);

    let coefficient = sample.coefficient();
    let ray_next = sample.into_ray();
    let radiance = light_material.shade(context, ray_next, intersection_next, 0);
    coefficient * radiance * weight
}

fn shade_scattering(
    material: &dyn Material,
    context: &mut Context<'_>,
    ray: &Ray,
    intersection: &RayIntersection,
    depth: usize,
) -> Color {
    let renderer = context.renderer();
    let lights = context.scene().get_lights();

    let sample = material.coef_sample(ray, intersection, *context.rng());
    let ray_next = sample.ray();

    let pdf_scattering = sample.pdf();
    if pdf_scattering == Val(0.0) {
        return Color::BLACK;
    }
    let pdf_light = lights.light_pdf(intersection, ray_next);
    let weight = pdf_scattering / (pdf_light + pdf_scattering);

    let coefficient = sample.coefficient();
    let ray_next = sample.into_ray();
    let radiance = renderer.trace(context, ray_next, DisRange::positive(), depth + 1);
    coefficient * radiance * weight
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    Diffuse,
    Emissive,
    Glossy,
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
    fn add_material<M: Material>(&mut self, material: M) -> MaterialId
    where
        Self: Sized;

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material>;
}
