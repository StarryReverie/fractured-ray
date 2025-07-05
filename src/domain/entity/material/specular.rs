use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::geometry::{Product, Val};
use crate::domain::ray::RayTrace;
use crate::domain::renderer::Renderer;

use super::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    albedo: Color,
}

impl Specular {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn calc_incident_ray_trace(
        &self,
        outgoing_ray_trace: &RayTrace,
        intersection: RayIntersection,
    ) -> RayTrace {
        let normal = intersection.normal();
        let dir = outgoing_ray_trace.direction();
        RayTrace::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("incident ray trace's direction should not be zero vector"),
        )
    }
}

impl Material for Specular {
    fn shade(
        &self,
        renderer: &dyn Renderer,
        outgoing_ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let incident_ray_trace = self.calc_incident_ray_trace(&outgoing_ray_trace, intersection);
        let color = renderer.trace(incident_ray_trace, DisRange::positive(), depth + 1);
        color * self.albedo
    }
}
