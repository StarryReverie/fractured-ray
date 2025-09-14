use std::fmt::Debug;

use crate::domain::material::def::{DynMaterial, RefDynMaterial};
use crate::domain::material::util::{MaterialContainer, MaterialId};
use crate::domain::scene::entity::{EntityContainer, EntityId};
use crate::domain::shape::def::Shape;
use crate::domain::shape::util::{ShapeContainer, ShapeId};

use super::{MaterialPool, ShapePool};

#[derive(Debug, Default)]
pub struct EntityPool {
    ids: Vec<EntityId>,
    shapes: ShapePool,
    materials: MaterialPool,
}

impl EntityPool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeContainer for EntityPool {
    fn add_shape<S: Shape + 'static>(&mut self, shape: S) -> ShapeId
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
    fn add_material(&mut self, material: DynMaterial) -> MaterialId {
        self.materials.add_material(material)
    }

    fn get_material(&self, id: MaterialId) -> Option<RefDynMaterial<'_>> {
        self.materials.get_material(id)
    }
}

impl EntityContainer for EntityPool {
    fn register_id(&mut self, id: EntityId) {
        self.ids.push(id);
    }

    fn get_ids(&self) -> &[EntityId] {
        &self.ids
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::color::Albedo;
    use crate::domain::material::def::{Material, MaterialKind};
    use crate::domain::material::primitive::Diffuse;
    use crate::domain::math::geometry::Point;
    use crate::domain::math::numeric::Val;
    use crate::domain::scene::entity::EntityId;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::primitive::Sphere;

    use super::*;

    #[test]
    fn entity_pool_operation_succeeds() {
        let mut pool = EntityPool::new();
        let shape_id = pool
            .add_shape(Sphere::new(Point::new(Val(0.0), Val(0.0), Val(0.0)), Val(1.0)).unwrap());
        let material_id = pool.add_material(Diffuse::new(Albedo::WHITE).into());
        let id = EntityId::new(shape_id, material_id);
        assert_eq!(
            pool.get_shape(id.shape_id()).unwrap().kind(),
            ShapeKind::Sphere
        );
        assert_eq!(
            pool.get_material(id.material_id()).unwrap().kind(),
            MaterialKind::Diffuse,
        );
    }
}
