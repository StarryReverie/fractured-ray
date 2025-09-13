use std::fmt::Debug;

use crate::domain::medium::def::{
    DynMedium, Medium, MediumContainer, MediumId, MediumKind, RefDynMedium,
};
use crate::domain::medium::primitive::{HenyeyGreenstein, Isotropic, Vacuum};

#[derive(Debug, Default)]
pub struct MediumPool {
    henyey_greenstein: Vec<HenyeyGreenstein>,
    isotropic: Vec<Isotropic>,
    vacuum: Vec<Vacuum>,
}

impl MediumPool {
    fn push<M>(medium: M, collection: &mut Vec<M>) -> MediumId
    where
        M: Medium,
    {
        let kind = medium.kind();
        collection.push(medium);
        MediumId::new(kind, collection.len() as u32 - 1)
    }
}

impl MediumContainer for MediumPool {
    fn add_medium(&mut self, medium: DynMedium) -> MediumId {
        match medium {
            DynMedium::HenyeyGreenstein(s) => Self::push(s, &mut self.henyey_greenstein),
            DynMedium::Isotropic(s) => Self::push(s, &mut self.isotropic),
            DynMedium::Vacuum(s) => Self::push(s, &mut self.vacuum),
        }
    }

    fn get_medium(&self, medium_id: MediumId) -> Option<RefDynMedium> {
        let index = medium_id.index() as usize;
        match medium_id.kind() {
            MediumKind::HenyeyGreenstein => self.henyey_greenstein.get(index).map(Into::into),
            MediumKind::Isotropic => self.isotropic.get(index).map(Into::into),
            MediumKind::Vacuum => self.vacuum.get(index).map(Into::into),
        }
    }
}
