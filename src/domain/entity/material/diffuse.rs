use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::geometry::{Product, Vector};
use crate::domain::ray::{Ray, RayTrace};
use crate::domain::renderer::Renderer;

use super::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct Diffuse {
    albedo: Color,
}

impl Diffuse {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn generate_ray_trace<R: Rng>(&self, intersection: &RayIntersection, mut rng: R) -> RayTrace {
        let normal = intersection.normal();
        loop {
            let (x, y, z) = rng.random::<(f64, f64, f64)>();
            let direction = Vector::new(x * 2.0 - 1.0, y * 2.0 - 1.0, z * 2.0 - 1.0);
            if direction.norm_squared() > 1e-8 && direction.dot(normal) > 0.0 {
                let direction = direction
                    .normalize()
                    .expect("direction should not be zero vector");
                return RayTrace::new(intersection.position(), direction);
            }
        }
    }

    fn shade_impl(
        &self,
        renderer: &Renderer,
        ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
        rng: ThreadRng,
    ) -> Ray {
        let incident_ray_trace = self.generate_ray_trace(&intersection, rng);
        let incident_ray = renderer.trace(incident_ray_trace, DisRange::positive(), depth + 1);
        let cos_angle = intersection.normal().dot(incident_ray.direction()).abs();
        let color = cos_angle * incident_ray.color() * self.albedo;
        Ray::new(
            RayTrace::new(ray_trace.start(), -ray_trace.direction()),
            color,
        )
    }
}

impl Material for Diffuse {
    fn shade(
        &self,
        renderer: &Renderer,
        ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
    ) -> Ray {
        let rng = rand::rng();
        self.shade_impl(renderer, ray_trace, intersection, depth, rng)
    }
}
