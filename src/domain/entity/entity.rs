use std::any::{Any, TypeId};
use std::mem::ManuallyDrop;

use super::material::{Diffuse, Emissive, Material, MaterialKind, Specular};
use super::shape::{MeshPolygon, MeshTriangle, Plane, Polygon, Shape, ShapeKind, Sphere, Triangle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityId {
    Composition {
        shape_kind: ShapeKind,
        shape_id: u32,
        material_kind: MaterialKind,
        material_id: u32,
    },
    MeshTriangle {
        id: u32,
    },
    MeshPolygon {
        id: u32,
    },
}

impl EntityId {
    pub fn composition(
        shape_kind: ShapeKind,
        shape_id: u32,
        material_kind: MaterialKind,
        material_id: u32,
    ) -> Self {
        Self::Composition {
            shape_kind,
            shape_id,
            material_kind,
            material_id,
        }
    }

    pub fn mesh_triangle(id: u32) -> Self {
        Self::MeshTriangle { id }
    }

    pub fn mesh_polygon(id: u32) -> Self {
        Self::MeshPolygon { id }
    }
}

#[derive(Debug, Default)]
pub struct EntityPool {
    shapes: ShapePool,
    materials: MaterialPool,
    mesh_triangles: Vec<MeshTriangle>,
    mesh_polygons: Vec<MeshPolygon>,
}

impl EntityPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_composition<S, M>(&mut self, shape: S, material: M) -> EntityId
    where
        S: Shape,
        M: Material,
    {
        let (shape_kind, shape_id) = self.shapes.add(shape);
        let (material_kind, material_id) = self.materials.add(material);
        EntityId::composition(shape_kind, shape_id, material_kind, material_id)
    }

    pub fn add_mesh_triangle(&mut self, mesh_triangle: MeshTriangle) -> EntityId {
        self.mesh_triangles.push(mesh_triangle);
        EntityId::mesh_triangle(self.mesh_triangles.len() as u32 - 1)
    }

    pub fn add_mesh_polygon(&mut self, mesh_polygon: MeshPolygon) -> EntityId {
        self.mesh_polygons.push(mesh_polygon);
        EntityId::mesh_polygon(self.mesh_polygons.len() as u32 - 1)
    }

    pub fn get_shape(&self, id: EntityId) -> Option<&dyn Shape> {
        match id {
            EntityId::Composition {
                shape_kind,
                shape_id,
                ..
            } => self.shapes.get(shape_kind, shape_id),
            EntityId::MeshTriangle { id } => {
                self.mesh_triangles.get(id as usize).map(ShapePool::upcast)
            }
            EntityId::MeshPolygon { id } => {
                self.mesh_polygons.get(id as usize).map(ShapePool::upcast)
            }
        }
    }

    pub fn get_material(&self, id: EntityId) -> Option<&dyn Material> {
        match id {
            EntityId::Composition {
                material_kind,
                material_id,
                ..
            } => self.materials.get(material_kind, material_id),
            EntityId::MeshTriangle { id } => self
                .mesh_triangles
                .get(id as usize)
                .map(MaterialPool::upcast),
            EntityId::MeshPolygon { id } => self
                .mesh_polygons
                .get(id as usize)
                .map(MaterialPool::upcast),
        }
    }
}

#[derive(Debug, Default)]
struct ShapePool {
    planes: Vec<Plane>,
    polygons: Vec<Polygon>,
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
}

impl ShapePool {
    pub fn add<S: Shape>(&mut self, shape: S) -> (ShapeKind, u32) {
        let shape_kind = shape.shape_kind();
        let type_id = TypeId::of::<S>();
        if type_id == TypeId::of::<Plane>() {
            let id = Self::downcast_and_push(shape, &mut self.planes);
            (shape_kind, id)
        } else if type_id == TypeId::of::<Polygon>() {
            let id = Self::downcast_and_push(shape, &mut self.polygons);
            (shape_kind, id)
        } else if type_id == TypeId::of::<Sphere>() {
            let id = Self::downcast_and_push(shape, &mut self.spheres);
            (shape_kind, id)
        } else if type_id == TypeId::of::<Triangle>() {
            let id = Self::downcast_and_push(shape, &mut self.triangles);
            (shape_kind, id)
        } else {
            unreachable!("all Shape's subtypes should be exhausted")
        }
    }

    fn downcast_and_push<S: Shape>(shape: impl Shape + Any, collection: &mut Vec<S>) -> u32 {
        assert_eq!(TypeId::of::<S>(), shape.type_id());
        // SAFETY: Already checked that S == impl Shape + Any.
        let shape = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(shape)) };

        collection.push(shape);
        collection.len() as u32 - 1
    }

    pub fn get(&self, shape_kind: ShapeKind, shape_id: u32) -> Option<&dyn Shape> {
        let shape_id = shape_id as usize;
        match shape_kind {
            ShapeKind::Plane => self.planes.get(shape_id).map(Self::upcast),
            ShapeKind::Polygon => self.polygons.get(shape_id).map(Self::upcast),
            ShapeKind::Triangle => self.triangles.get(shape_id).map(Self::upcast),
            ShapeKind::Sphere => self.spheres.get(shape_id).map(Self::upcast),
        }
    }

    fn upcast<S: Shape>(shape: &S) -> &dyn Shape {
        shape
    }
}

#[derive(Debug, Default)]
struct MaterialPool {
    diffuse: Vec<Diffuse>,
    emissive: Vec<Emissive>,
    specular: Vec<Specular>,
}

impl MaterialPool {
    pub fn add<M: Material>(&mut self, material: M) -> (MaterialKind, u32) {
        let material_kind = material.material_kind();
        let type_id = TypeId::of::<M>();
        if type_id == TypeId::of::<Diffuse>() {
            let id = Self::downcast_and_push(material, &mut self.diffuse);
            (material_kind, id)
        } else if type_id == TypeId::of::<Emissive>() {
            let id = Self::downcast_and_push(material, &mut self.emissive);
            (material_kind, id)
        } else if type_id == TypeId::of::<Specular>() {
            let id = Self::downcast_and_push(material, &mut self.specular);
            (material_kind, id)
        } else {
            unreachable!("all Material's subtypes should be exhausted")
        }
    }

    fn downcast_and_push<M: Material>(
        material: impl Material + Any,
        collection: &mut Vec<M>,
    ) -> u32 {
        assert_eq!(TypeId::of::<M>(), material.type_id());
        // SAFETY: Already checked that M == impl Material + Any.
        let material = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(material)) };

        collection.push(material);
        collection.len() as u32 - 1
    }

    pub fn get(&self, material_kind: MaterialKind, material_id: u32) -> Option<&dyn Material> {
        let material_id = material_id as usize;
        match material_kind {
            MaterialKind::Diffuse => self.diffuse.get(material_id).map(Self::upcast),
            MaterialKind::Emissive => self.emissive.get(material_id).map(Self::upcast),
            MaterialKind::Specular => self.specular.get(material_id).map(Self::upcast),
        }
    }

    fn upcast<S: Material>(material: &S) -> &dyn Material {
        material
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Color;
    use crate::domain::geometry::{Point, Val};

    use super::*;

    #[test]
    fn entity_pool_operation_succeeds() {
        let mut pool = EntityPool::new();
        let id = pool.add_composition(
            Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(1.0)).unwrap(),
            Diffuse::new(Color::WHITE),
        );
        assert_eq!(pool.get_shape(id).unwrap().shape_kind(), ShapeKind::Sphere);
        assert_eq!(
            pool.get_material(id).unwrap().material_kind(),
            MaterialKind::Diffuse,
        );
    }
}
