use std::any::Any;
use std::fmt::Debug;

use crate::domain::material::def::{Material, MaterialContainer, MaterialId};
use crate::domain::scene::entity::EntityContainer;
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{MaterialPool, ShapePool};

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
    fn add_material<M>(&mut self, material: M) -> MaterialId
    where
        Self: Sized,
        M: Material + Any,
    {
        self.materials.add_material(material)
    }

    fn get_material(&self, id: MaterialId) -> Option<&dyn Material> {
        self.materials.get_material(id)
    }
}

impl EntityContainer for EntityPool {}

#[cfg(test)]
mod tests {
    use crate::domain::color::Albedo;
    use crate::domain::material::def::MaterialKind;
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
        let material_id = pool.add_material(Diffuse::new(Albedo::WHITE));
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
