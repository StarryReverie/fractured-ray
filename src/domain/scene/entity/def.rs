use std::fmt::Debug;

use crate::domain::material::def::{DynMaterial, MaterialKind};
use crate::domain::material::util::{MaterialContainer, MaterialId};
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::photon::PhotonSampling;
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::def::{Shape, ShapeConstructor, ShapeContainer, ShapeId, ShapeKind};

pub trait EntityScene: Send + Sync {
    fn get_entities(&self) -> &dyn EntityContainer;

    fn get_light_surfaces(&self) -> &dyn PointSampling;

    fn get_lights(&self) -> &dyn LightSampling;

    fn get_emitters(&self) -> &dyn PhotonSampling;

    fn find_intersection(&self, ray: &Ray, range: DisRange) -> Option<(RayIntersection, EntityId)>;

    fn test_intersection(
        &self,
        ray: &Ray,
        range: DisRange,
        shape_id: ShapeId,
    ) -> Option<(RayIntersection, EntityId)> {
        if let Some((intersection, id)) = self.find_intersection(ray, range) {
            if id.shape_id() == shape_id {
                Some((intersection, id))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait EntitySceneBuilder: Send + Sync + Sized {
    type Output: EntityScene;

    fn add<S, M>(&mut self, shape: S, material: M) -> &mut Self
    where
        S: Shape,
        M: Into<DynMaterial>;

    fn add_constructor<C, M>(&mut self, constructor: C, material: M) -> &mut Self
    where
        C: ShapeConstructor,
        M: Into<DynMaterial>;

    fn build(self) -> Self::Output;
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

impl From<EntityId> for ShapeId {
    fn from(value: EntityId) -> Self {
        value.shape_id()
    }
}

impl From<EntityId> for MaterialId {
    fn from(value: EntityId) -> Self {
        value.material_id()
    }
}

pub trait EntityContainer: ShapeContainer + MaterialContainer {
    fn register_id(&mut self, id: EntityId);

    fn get_ids(&self) -> &[EntityId];
}
