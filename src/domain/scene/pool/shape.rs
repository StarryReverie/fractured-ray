use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem::ManuallyDrop;

use crate::domain::shape::def::{Shape, ShapeKind};
use crate::domain::shape::primitive::*;
use crate::domain::shape::util::{Instance, ShapeContainer, ShapeId};

#[derive(Debug, Default)]
pub struct ShapePool {
    aabbs: Vec<Aabb>,
    instances: Vec<Instance>,
    mesh_polygons: Vec<MeshPolygon>,
    mesh_triangles: Vec<MeshTriangle>,
    planes: Vec<Plane>,
    polygons: Vec<Polygon>,
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
}

impl ShapePool {
    fn downcast_and_push<S: Shape>(shape: impl Shape + Any, collection: &mut Vec<S>) -> u32 {
        assert_eq!(TypeId::of::<S>(), shape.type_id());
        // SAFETY: Already checked that S == impl Shape + Any.
        let shape = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(shape)) };

        collection.push(shape);
        collection.len() as u32 - 1
    }

    fn upcast<S: Shape>(shape: &S) -> &dyn Shape {
        shape
    }
}

impl ShapeContainer for ShapePool {
    fn add_shape<S: Shape>(&mut self, shape: S) -> ShapeId {
        let kind = shape.kind();
        let type_id = TypeId::of::<S>();

        if type_id == TypeId::of::<Aabb>() {
            let index = Self::downcast_and_push(shape, &mut self.aabbs);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<Instance>() {
            let index = Self::downcast_and_push(shape, &mut self.instances);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<MeshPolygon>() {
            let index = Self::downcast_and_push(shape, &mut self.mesh_polygons);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<MeshTriangle>() {
            let index = Self::downcast_and_push(shape, &mut self.mesh_triangles);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<Plane>() {
            let index = Self::downcast_and_push(shape, &mut self.planes);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<Polygon>() {
            let index = Self::downcast_and_push(shape, &mut self.polygons);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<Sphere>() {
            let index = Self::downcast_and_push(shape, &mut self.spheres);
            ShapeId::new(kind, index)
        } else if type_id == TypeId::of::<Triangle>() {
            let index = Self::downcast_and_push(shape, &mut self.triangles);
            ShapeId::new(kind, index)
        } else {
            unreachable!("all Shape's subtypes should be exhausted")
        }
    }

    fn get_shape(&self, shape_id: ShapeId) -> Option<&dyn Shape> {
        let index = shape_id.index() as usize;
        match shape_id.kind() {
            ShapeKind::Aabb => self.aabbs.get(index).map(Self::upcast),
            ShapeKind::Instance => self.instances.get(index).map(Self::upcast),
            ShapeKind::MeshPolygon => self.mesh_polygons.get(index).map(Self::upcast),
            ShapeKind::MeshTriangle => self.mesh_triangles.get(index).map(Self::upcast),
            ShapeKind::Plane => self.planes.get(index).map(Self::upcast),
            ShapeKind::Polygon => self.polygons.get(index).map(Self::upcast),
            ShapeKind::Triangle => self.triangles.get(index).map(Self::upcast),
            ShapeKind::Sphere => self.spheres.get(index).map(Self::upcast),
        }
    }
}
