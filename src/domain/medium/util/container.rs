use std::fmt::Debug;

use getset::CopyGetters;

use crate::domain::medium::def::{DynMedium, MediumKind, RefDynMedium};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MediumId {
    kind: MediumKind,
    index: u32,
}

impl MediumId {
    pub fn new(kind: MediumKind, index: u32) -> Self {
        Self { kind, index }
    }
}

pub trait MediumContainer: Debug + Send + Sync + 'static {
    fn add_medium(&mut self, medium: DynMedium) -> MediumId;

    fn get_medium(&self, id: MediumId) -> Option<RefDynMedium>;
}
