use std::any::Any;

use crate::domain::medium::def::medium::{Medium, MediumContainer, MediumId};
use crate::domain::scene::volume::BoundaryContainer;
use crate::domain::shape::def::{Shape, ShapeContainer, ShapeId};

use super::{MediumPool, ShapePool};

#[derive(Debug, Default)]
pub struct BoundaryPool {
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
    fn add_medium<M>(&mut self, medium: M) -> MediumId
    where
        Self: Sized,
        M: Medium + Any,
    {
        self.media.add_medium(medium)
    }

    fn get_medium(&self, id: MediumId) -> Option<&dyn Medium> {
        self.media.get_medium(id)
    }
}

impl BoundaryContainer for BoundaryPool {}
