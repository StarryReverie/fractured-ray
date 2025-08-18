use std::fmt::Debug;

use getset::CopyGetters;

use crate::domain::math::algebra::UnitVector;
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::sampling::Sampleable;

use super::BoundingBox;

pub trait Shape: Sampleable + Debug + Send + Sync + 'static {
    fn kind(&self) -> ShapeKind;

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection>;

    fn area(&self) -> Val;

    fn normal(&self, position: Point) -> UnitVector;

    fn bounding_box(&self) -> Option<BoundingBox>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeKind {
    Aabb,
    Instance,
    MeshPolygon,
    MeshTriangle,
    Plane,
    Polygon,
    Sphere,
    Triangle,
}

pub trait ShapeConstructor: Debug + Send + Sync + 'static {
    fn construct<C: ShapeContainer>(self, container: &mut C) -> Vec<ShapeId>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ShapeId {
    kind: ShapeKind,
    index: u32,
}

impl ShapeId {
    pub fn new(kind: ShapeKind, index: u32) -> Self {
        Self { kind, index }
    }
}

pub trait ShapeContainer: Debug + Send + Sync + 'static {
    fn add_shape<S: Shape>(&mut self, shape: S) -> ShapeId
    where
        Self: Sized;

    fn get_shape(&self, id: ShapeId) -> Option<&dyn Shape>;
}
