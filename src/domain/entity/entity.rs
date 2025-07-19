use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem::ManuallyDrop;

use crate::domain::material::def::{Material, MaterialContainer, MaterialId, MaterialKind};
use crate::domain::material::primitive::{Diffuse, Emissive, Refractive, Scattering, Specular};
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId, ShapeKind};
use crate::domain::shape::instance::Instance;
use crate::domain::shape::mesh::{MeshPolygon, MeshTriangle};
use crate::domain::shape::primitive::{Plane, Polygon, Sphere, Triangle};

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
            shape_kind: shape_id.kind(),
            shape_index: shape_id.index(),
            material_kind: material_id.kind(),
            material_index: material_id.index(),
        }
    }

    pub fn shape_id(&self) -> ShapeId {
        ShapeId::new(self.shape_kind, self.shape_index)
    }

    pub fn material_id(&self) -> MaterialId {
        MaterialId::new(self.material_kind, self.material_index)
    }
}

pub trait EntityContainer: ShapeContainer + MaterialContainer {}

#[derive(Debug, Default)]
pub struct EntityPool {
    shapes: ShapePool,
    materials: MaterialPool,
}

impl EntityPool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeContainer for EntityPool {
    fn add_shape<S: Shape>(&mut self, shape: S) -> ShapeId
    where
        Self: Sized,
    {
        self.shapes.add_shape(shape)
    }

    fn get_shape(&self, id: ShapeId) -> Option<&dyn Shape> {
        self.shapes.get_shape(id)
    }
}

impl MaterialContainer for EntityPool {
    fn add_material<M: Material>(&mut self, material: M) -> MaterialId
    where
        Self: Sized,
    {
        self.materials.add_material(material)
    }

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material> {
        self.materials.get_material(id)
    }
}

impl EntityContainer for EntityPool {}

#[derive(Debug, Default)]
struct ShapePool {
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
        let kind = shape.shape_kind();
        let type_id = TypeId::of::<S>();

        if type_id == TypeId::of::<Instance>() {
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

#[derive(Debug, Default)]
struct MaterialPool {
    diffuse: Vec<Diffuse>,
    emissive: Vec<Emissive>,
    refractive: Vec<Refractive>,
    scattering: Vec<Scattering>,
    specular: Vec<Specular>,
}

impl MaterialPool {
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

    fn upcast<S: Material>(material: &S) -> &dyn Material {
        material
    }
}

impl MaterialContainer for MaterialPool {
    fn add_material<M: Material>(&mut self, material: M) -> MaterialId {
        let kind = material.material_kind();
        let type_id = TypeId::of::<M>();

        if type_id == TypeId::of::<Diffuse>() {
            let index = Self::downcast_and_push(material, &mut self.diffuse);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Emissive>() {
            let index = Self::downcast_and_push(material, &mut self.emissive);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Refractive>() {
            let index = Self::downcast_and_push(material, &mut self.refractive);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Scattering>() {
            let index = Self::downcast_and_push(material, &mut self.scattering);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Specular>() {
            let index = Self::downcast_and_push(material, &mut self.specular);
            MaterialId::new(kind, index)
        } else {
            unreachable!("all Material's subtypes should be exhausted")
        }
    }

    fn get_material(&self, material_id: MaterialId) -> Option<&dyn Material> {
        let index = material_id.index() as usize;
        match material_id.kind() {
            MaterialKind::Diffuse => self.diffuse.get(index).map(Self::upcast),
            MaterialKind::Emissive => self.emissive.get(index).map(Self::upcast),
            MaterialKind::Refractive => self.refractive.get(index).map(Self::upcast),
            MaterialKind::Scattering => self.scattering.get(index).map(Self::upcast),
            MaterialKind::Specular => self.specular.get(index).map(Self::upcast),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Color;
    use crate::domain::math::geometry::Point;
    use crate::domain::math::numeric::Val;

    use super::*;

    #[test]
    fn entity_pool_operation_succeeds() {
        let mut pool = EntityPool::new();
        let shape_id = pool
            .add_shape(Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(1.0)).unwrap());
        let material_id = pool.add_material(Diffuse::new(Color::WHITE));
        let id = EntityId::new(shape_id, material_id);
        assert_eq!(
            pool.get_shape(id.shape_id()).unwrap().shape_kind(),
            ShapeKind::Sphere
        );
        assert_eq!(
            pool.get_material(id.material_id()).unwrap().material_kind(),
            MaterialKind::Diffuse,
        );
    }
}
