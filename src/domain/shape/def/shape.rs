use std::fmt::Debug;

use crate::domain::math::numeric::DisRange;
use crate::domain::ray::sampling::Sampleable;
use crate::domain::ray::{Ray, RayIntersection};

use super::BoundingBox;

pub trait Shape: Sampleable + Debug + Send + Sync + 'static {
    fn kind(&self) -> ShapeKind;

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection>;

    fn bounding_box(&self) -> Option<BoundingBox>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeKind {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeId {
    kind: ShapeKind,
    index: u32,
}

impl ShapeId {
    pub fn new(kind: ShapeKind, index: u32) -> Self {
        Self { kind, index }
    }

    pub fn kind(&self) -> ShapeKind {
        self.kind
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub trait ShapeContainer: Debug + Send + Sync + 'static {
    fn add_shape<S: Shape>(&mut self, shape: S) -> ShapeId
    where
        Self: Sized;

    fn get_shape(&self, id: ShapeId) -> Option<&dyn Shape>;
}
