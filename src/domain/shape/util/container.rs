use std::fmt::Debug;

use getset::CopyGetters;

use crate::domain::shape::def::{DynShape, RefDynShape, ShapeKind};

pub trait ShapeConstructor: Debug + Send + Sync + 'static {
    fn construct(self: Box<Self>, container: &mut dyn ShapeContainer) -> Vec<ShapeId>;
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
    fn add_shape(&mut self, shape: DynShape) -> ShapeId;

    fn get_shape(&self, id: ShapeId) -> Option<RefDynShape>;
}
