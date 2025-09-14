use std::fmt::Debug;

use crate::domain::shape::def::{DynShape, RefDynShape, Shape, ShapeKind};
use crate::domain::shape::primitive::*;
use crate::domain::shape::util::{Instance, ShapeContainer, ShapeId};

#[derive(Debug, Default)]
pub struct ShapePool {
    aabbs: Vec<Aabb>,
    mesh_polygons: Vec<MeshPolygon>,
    mesh_triangles: Vec<MeshTriangle>,
    planes: Vec<Plane>,
    polygons: Vec<Polygon>,
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
    instances: Vec<Instance>,
}

impl ShapePool {
    fn push<S>(shape: S, collection: &mut Vec<S>) -> ShapeId
    where
        S: Shape,
    {
        let kind = shape.kind();
        collection.push(shape);
        ShapeId::new(kind, collection.len() as u32 - 1)
    }
}

impl ShapeContainer for ShapePool {
    fn add_shape(&mut self, shape: DynShape) -> ShapeId {
        match shape {
            DynShape::Aabb(s) => Self::push(s, &mut self.aabbs),
            DynShape::MeshPolygon(s) => Self::push(s, &mut self.mesh_polygons),
            DynShape::MeshTriangle(s) => Self::push(s, &mut self.mesh_triangles),
            DynShape::Plane(s) => Self::push(s, &mut self.planes),
            DynShape::Polygon(s) => Self::push(s, &mut self.polygons),
            DynShape::Sphere(s) => Self::push(s, &mut self.spheres),
            DynShape::Triangle(s) => Self::push(s, &mut self.triangles),
            DynShape::Instance(s) => Self::push(s, &mut self.instances),
        }
    }

    fn get_shape(&self, shape_id: ShapeId) -> Option<RefDynShape> {
        let index = shape_id.index() as usize;
        match shape_id.kind() {
            ShapeKind::Aabb => self.aabbs.get(index).map(Into::into),
            ShapeKind::MeshPolygon => self.mesh_polygons.get(index).map(Into::into),
            ShapeKind::MeshTriangle => self.mesh_triangles.get(index).map(Into::into),
            ShapeKind::Plane => self.planes.get(index).map(Into::into),
            ShapeKind::Polygon => self.polygons.get(index).map(Into::into),
            ShapeKind::Triangle => self.triangles.get(index).map(Into::into),
            ShapeKind::Sphere => self.spheres.get(index).map(Into::into),
            ShapeKind::Instance => self.instances.get(index).map(Into::into),
        }
    }
}
