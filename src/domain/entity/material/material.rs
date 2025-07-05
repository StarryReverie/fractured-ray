use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::entity::shape::RayIntersection;
use crate::domain::ray::RayTrace;
use crate::domain::renderer::Renderer;

pub trait Material: Debug + Send + Sync + 'static {
    fn shade(
        &self,
        renderer: &dyn Renderer,
        outgoing_ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color;
}
