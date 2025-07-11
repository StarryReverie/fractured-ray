use std::fmt::Debug;

use crate::domain::entity::{ShapeContainer, ShapeId};
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::{Ray, RayIntersection};

use super::BoundingBox;

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

pub trait Shape: Debug + Send + Sync + 'static {
    fn shape_kind(&self) -> ShapeKind;

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection>;

    fn bounding_box(&self) -> Option<BoundingBox>;
}

pub trait ShapeConstructor: Debug + Send + Sync + 'static {
    fn construct<C: ShapeContainer>(self, container: &mut C) -> Vec<ShapeId>;
}
