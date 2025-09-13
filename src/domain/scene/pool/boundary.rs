use crate::domain::medium::def::{DynMedium, RefDynMedium};
use crate::domain::medium::util::{MediumContainer, MediumId};
use crate::domain::scene::volume::{BoundaryContainer, BoundaryId};
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{MediumPool, ShapePool};

#[derive(Debug, Default)]
pub struct BoundaryPool {
    ids: Vec<BoundaryId>,
    shapes: ShapePool,
    media: MediumPool,
}

impl BoundaryPool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeContainer for BoundaryPool {
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

impl MediumContainer for BoundaryPool {
    fn add_medium(&mut self, medium: DynMedium) -> MediumId {
        self.media.add_medium(medium)
    }

    fn get_medium(&self, id: MediumId) -> Option<RefDynMedium> {
        self.media.get_medium(id)
    }
}

impl BoundaryContainer for BoundaryPool {
    fn register_id(&mut self, id: BoundaryId) {
        self.ids.push(id);
    }

    fn get_ids(&self) -> &[BoundaryId] {
        &self.ids
    }
}
