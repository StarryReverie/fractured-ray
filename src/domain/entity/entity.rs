use crate::domain::ray::{Ray, RayTrace};
use crate::domain::renderer::Renderer;

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

    pub fn hit(&self, ray: &RayTrace, range: DisRange) -> Option<RayIntersection> {
        self.shape.hit(ray, range)
    }

    pub fn shade(
        &self,
        renderer: &Renderer,
        ray_trace: RayTrace,
        intersection: RayIntersection,
        depth: usize,
    ) -> Ray {
        self.material
            .shade(renderer, ray_trace, intersection, depth)
    }
}
