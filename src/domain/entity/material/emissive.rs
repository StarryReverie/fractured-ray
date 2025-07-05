use crate::domain::color::Color;
use crate::domain::entity::shape::RayIntersection;
use crate::domain::ray::RayTrace;
use crate::domain::renderer::Renderer;

use super::Material;

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
    fn shade(
        &self,
        _renderer: &dyn Renderer,
        _outgoing_ray_trace: RayTrace,
        _intersection: RayIntersection,
        _depth: usize,
    ) -> Color {
        self.radiance
    }
}
