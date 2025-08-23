use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem::ManuallyDrop;

use crate::domain::medium::def::{Medium, MediumContainer, MediumId, MediumKind};
use crate::domain::medium::primitive::Isotropic;

#[derive(Debug, Default)]
pub struct MediumPool {
    isotropic: Vec<Isotropic>,
}

impl MediumPool {
    fn downcast_and_push<MI, M>(medium: MI, collection: &mut Vec<M>) -> u32
    where
        MI: Medium + Any,
        M: Medium + Any,
    {
        assert_eq!(TypeId::of::<M>(), medium.type_id());
        // SAFETY: Already checked that M == impl Medium + Any.
        let medium = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(medium)) };

        collection.push(medium);
        collection.len() as u32 - 1
    }

    fn upcast<M: Medium>(medium: &M) -> &dyn Medium {
        medium
    }
}

impl MediumContainer for MediumPool {
    fn add_medium<M>(&mut self, medium: M) -> MediumId
    where
        Self: Sized,
        M: Medium + Any,
    {
        let kind = medium.kind();
        let type_id = TypeId::of::<M>();

        if type_id == TypeId::of::<Isotropic>() {
            let index = Self::downcast_and_push(medium, &mut self.isotropic);
            MediumId::new(kind, index)
        } else {
            unreachable!("all Medium's subtypes should be exhausted")
        }
    }

    fn get_medium(&self, medium_id: MediumId) -> Option<&dyn Medium> {
        let index = medium_id.index() as usize;
        match medium_id.kind() {
            MediumKind::Isotropic => self.isotropic.get(index).map(Self::upcast),
        }
    }
}
