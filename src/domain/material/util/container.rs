use std::fmt::Debug;

use crate::domain::material::def::{DynMaterial, MaterialKind, RefDynMaterial};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId {
    kind: MaterialKind,
    index: u32,
}

impl MaterialId {
    pub fn new(kind: MaterialKind, index: u32) -> Self {
        Self { kind, index }
    }

    pub fn kind(&self) -> MaterialKind {
        self.kind
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub trait MaterialContainer: Debug + Send + Sync + 'static {
    fn add_material(&mut self, material: DynMaterial) -> MaterialId;

    fn get_material(&self, id: MaterialId) -> Option<RefDynMaterial>;
}
