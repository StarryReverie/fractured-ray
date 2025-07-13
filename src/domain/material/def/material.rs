use std::fmt::Debug;

use crate::domain::color::Color;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Context;

pub trait Material: Debug + Send + Sync + 'static {
    fn material_kind(&self) -> MaterialKind;

    fn shade(
        &self,
        context: &Context<'_>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    Diffuse,
    Emissive,
    Refractive,
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
    fn add_material<M: Material>(&mut self, material: M) -> MaterialId;

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material>;
}
