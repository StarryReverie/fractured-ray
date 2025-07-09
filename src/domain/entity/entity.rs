use std::any::{Any, TypeId};
use std::mem::ManuallyDrop;

use super::material::{Diffuse, Emissive, Material, MaterialKind, Refractive, Specular};
use super::shape::{MeshPolygon, MeshTriangle, Plane, Polygon, Shape, ShapeKind, Sphere, Triangle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeId {
    kind: ShapeKind,
    index: u32,
}

impl ShapeId {
    pub fn new(kind: ShapeKind, index: u32) -> Self {
        Self { kind, index }
    }
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    shape_kind: ShapeKind,
    shape_index: u32,
    material_kind: MaterialKind,
    material_index: u32,
}

impl EntityId {
    pub fn new(shape_id: ShapeId, material_id: MaterialId) -> Self {
        Self {
            shape_kind: shape_id.kind,
            shape_index: shape_id.index,
            material_kind: material_id.kind,
            material_index: material_id.index,
        }
    }

    pub fn shape_kind(&self) -> ShapeKind {
        self.shape_kind
    }

    pub fn shape_index(&self) -> u32 {
        self.shape_index
    }

    pub fn material_kind(&self) -> MaterialKind {
        self.material_kind
    }

    pub fn material_index(&self) -> u32 {
        self.material_index
    }
}

#[derive(Debug, Default)]
pub struct EntityPool {
    shapes: ShapePool,
    materials: MaterialPool,
}

impl EntityPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_shape<S: Shape>(&mut self, shape: S) -> ShapeId {
        self.shapes.add(shape)
    }

    pub fn add_material<M: Material>(&mut self, material: M) -> MaterialId {
        self.materials.add(material)
    }

    pub fn get_shape(&self, id: EntityId) -> Option<&dyn Shape> {
        self.shapes
            .get(ShapeId::new(id.shape_kind(), id.shape_index()))
    }

    pub fn get_material(&self, id: EntityId) -> Option<&dyn Material> {
        self.materials
            .get(MaterialId::new(id.material_kind(), id.material_index()))
    }
}

#[derive(Debug, Default)]
struct ShapePool {
    mesh_polygons: Vec<MeshPolygon>,
    mesh_triangles: Vec<MeshTriangle>,
    planes: Vec<Plane>,
    polygons: Vec<Polygon>,
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
}

impl ShapePool {
    pub fn add<S: Shape>(&mut self, shape: S) -> ShapeId {
        let kind = shape.shape_kind();
        let type_id = TypeId::of::<S>();

        if type_id == TypeId::of::<MeshPolygon>() {
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

    fn downcast_and_push<S: Shape>(shape: impl Shape + Any, collection: &mut Vec<S>) -> u32 {
        assert_eq!(TypeId::of::<S>(), shape.type_id());
        // SAFETY: Already checked that S == impl Shape + Any.
        let shape = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(shape)) };

        collection.push(shape);
        collection.len() as u32 - 1
    }

    pub fn get(&self, shape_id: ShapeId) -> Option<&dyn Shape> {
        let index = shape_id.index as usize;
        match shape_id.kind {
            ShapeKind::MeshPolygon => self.mesh_polygons.get(index).map(Self::upcast),
            ShapeKind::MeshTriangle => self.mesh_triangles.get(index).map(Self::upcast),
            ShapeKind::Plane => self.planes.get(index).map(Self::upcast),
            ShapeKind::Polygon => self.polygons.get(index).map(Self::upcast),
            ShapeKind::Triangle => self.triangles.get(index).map(Self::upcast),
            ShapeKind::Sphere => self.spheres.get(index).map(Self::upcast),
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
    refractive: Vec<Refractive>,
    specular: Vec<Specular>,
}

impl MaterialPool {
    pub fn add<M: Material>(&mut self, material: M) -> MaterialId {
        let kind = material.material_kind();
        let type_id = TypeId::of::<M>();

        if type_id == TypeId::of::<Diffuse>() {
            let index = Self::downcast_and_push(material, &mut self.diffuse);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Emissive>() {
            let index = Self::downcast_and_push(material, &mut self.emissive);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Specular>() {
            let index = Self::downcast_and_push(material, &mut self.specular);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Refractive>() {
            let index = Self::downcast_and_push(material, &mut self.refractive);
            MaterialId::new(kind, index)
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

    pub fn get(&self, material_id: MaterialId) -> Option<&dyn Material> {
        let index = material_id.index as usize;
        match material_id.kind {
            MaterialKind::Diffuse => self.diffuse.get(index).map(Self::upcast),
            MaterialKind::Emissive => self.emissive.get(index).map(Self::upcast),
            MaterialKind::Refractive => self.refractive.get(index).map(Self::upcast),
            MaterialKind::Specular => self.specular.get(index).map(Self::upcast),
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
        let shape_id = pool
            .add_shape(Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(1.0)).unwrap());
        let material_id = pool.add_material(Diffuse::new(Color::WHITE));
        let id = EntityId::new(shape_id, material_id);
        assert_eq!(pool.get_shape(id).unwrap().shape_kind(), ShapeKind::Sphere);
        assert_eq!(
            pool.get_material(id).unwrap().material_kind(),
            MaterialKind::Diffuse,
        );
    }
}
