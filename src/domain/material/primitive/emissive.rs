use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Renderer;

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
    fn shade(
        &self,
        _renderer: &dyn Renderer,
        _ray: Ray,
        _intersection: RayIntersection,
        _depth: usize,
    ) -> Color {
        self.radiance
    }
}
