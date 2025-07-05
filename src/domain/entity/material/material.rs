use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::entity::shape::RayIntersection;
use crate::domain::ray::Ray;
use crate::domain::renderer::Renderer;

pub trait Material: Debug + Send + Sync + 'static {
    fn shade(
        &self,
        renderer: &dyn Renderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color;
}
