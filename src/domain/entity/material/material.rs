use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::entity::shape::RayIntersection;
use crate::domain::ray::Ray;
use crate::domain::renderer::Renderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    Diffuse,
    Emissive,
    Specular,
}

pub trait Material: Debug + Send + Sync + 'static {
    fn material_kind(&self) -> MaterialKind;

    fn shade(
        &self,
        renderer: &dyn Renderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color;
}
