use crate::domain::color::Color;
use crate::domain::entity::shape::RayIntersection;
use crate::domain::ray::{Ray, RayTrace};
use crate::domain::renderer::Renderer;

use super::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct Emissive {
    intensity: Color,
}

impl Emissive {
    pub fn new(intensity: Color) -> Self {
        Self { intensity }
    }
}

impl Material for Emissive {
    fn shade(
        &self,
        _renderer: &dyn Renderer,
        outgoing_ray_trace: RayTrace,
        _intersection: RayIntersection,
        _depth: usize,
    ) -> Ray {
        Ray::new(
            RayTrace::new(outgoing_ray_trace.start(), -outgoing_ray_trace.direction()),
            self.intensity,
        )
    }
}
