use crate::domain::color::Color;
use crate::domain::ray::Ray;
use crate::domain::renderer::CoreRenderer;

use super::material::Material;
use super::shape::{DisRange, RayIntersection, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(usize);

impl Id {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Entity {
    id: Id,
    shape: Box<dyn Shape>,
    material: Box<dyn Material>,
}

impl Entity {
    pub fn new<S: Shape, M: Material>(id: Id, shape: S, material: M) -> Self {
        Entity {
            id,
            shape: Box::new(shape),
            material: Box::new(material),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        self.shape.hit(ray, range)
    }

    pub fn shade(
        &self,
        renderer: &CoreRenderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        self.material.shade(renderer, ray, intersection, depth)
    }
}
