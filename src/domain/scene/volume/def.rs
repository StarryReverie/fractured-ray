use getset::CopyGetters;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::medium::def::medium::{MediumContainer, MediumId, MediumKind};
use crate::domain::ray::Ray;
use crate::domain::shape::def::{ShapeContainer, ShapeId, ShapeKind};

pub trait VolumeScene: Send + Sync {
    fn find_segments(&self, ray: &Ray, range: DisRange) -> Vec<MediumSegment>;
}

#[derive(Debug, Clone, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MediumSegment {
    start: Point,
    length: Val,
    medium: MediumId,
}

impl MediumSegment {
    pub fn new(start: Point, length: Val, medium: MediumId) -> Self {
        Self {
            start,
            length,
            medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundaryId {
    shape_kind: ShapeKind,
    shape_index: u32,
    medium_kind: MediumKind,
    medium_index: u32,
}

impl BoundaryId {
    pub fn new(shape_id: ShapeId, medium_id: MediumId) -> Self {
        Self {
            shape_kind: shape_id.kind(),
            shape_index: shape_id.index(),
            medium_kind: medium_id.kind(),
            medium_index: medium_id.index(),
        }
    }

    pub fn shape_id(&self) -> ShapeId {
        ShapeId::new(self.shape_kind, self.shape_index)
    }

    pub fn medium_id(&self) -> MediumId {
        MediumId::new(self.medium_kind, self.medium_index)
    }
}

impl From<BoundaryId> for ShapeId {
    fn from(value: BoundaryId) -> Self {
        value.shape_id()
    }
}

pub trait BoundaryContainer: ShapeContainer + MediumContainer {
    fn register_id(&mut self, id: BoundaryId);

    fn get_ids(&self) -> &[BoundaryId];
}
