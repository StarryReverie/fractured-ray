use crate::domain::color::Color;
use crate::domain::ray::Ray;
use crate::domain::renderer::CoreRenderer;

use super::material::Material;
use super::shape::{DisRange, MeshPolygon, MeshTriangle, RayIntersection, Shape};

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
    inner: EntityInner,
}

impl Entity {
    pub fn new<S: Shape, M: Material>(id: Id, shape: S, material: M) -> Self {
        Entity {
            id,
            inner: EntityInner::Composition {
                shape: Box::new(shape),
                material: Box::new(material),
            },
        }
    }

    pub fn new_mesh_triangle(id: Id, triangle: MeshTriangle) -> Self {
        Entity {
            id,
            inner: EntityInner::MeshTriangle(triangle),
        }
    }

    pub fn new_mesh_polygon(id: Id, polygon: MeshPolygon) -> Self {
        Entity {
            id,
            inner: EntityInner::MeshPolygon(polygon),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        match &self.inner {
            EntityInner::Composition { shape, .. } => shape.hit(ray, range),
            EntityInner::MeshTriangle(s) => s.hit(ray, range),
            EntityInner::MeshPolygon(s) => s.hit(ray, range),
        }
    }

    pub fn shade(
        &self,
        renderer: &CoreRenderer,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        match &self.inner {
            EntityInner::Composition { material, .. } => {
                material.shade(renderer, ray, intersection, depth)
            }
            EntityInner::MeshTriangle(s) => s.shade(renderer, ray, intersection, depth),
            EntityInner::MeshPolygon(s) => s.shade(renderer, ray, intersection, depth),
        }
    }
}

#[derive(Debug)]
enum EntityInner {
    Composition {
        shape: Box<dyn Shape>,
        material: Box<dyn Material>,
    },
    MeshTriangle(MeshTriangle),
    MeshPolygon(MeshPolygon),
}
