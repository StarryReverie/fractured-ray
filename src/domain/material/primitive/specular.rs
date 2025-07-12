use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::Product;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Renderer;

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    albedo: Color,
}

impl Specular {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn calc_reflective_ray(&self, ray: &Ray, intersection: RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        Ray::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("reflective ray's direction should not be zero vector"),
        )
    }
}

impl Material for Specular {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn shade(
        &self,
        renderer: &dyn Renderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let reflective_ray = self.calc_reflective_ray(&ray, intersection);
        let color = renderer.trace(reflective_ray, DisRange::positive(), depth + 1);
        color * self.albedo
    }
}
