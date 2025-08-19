use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem::ManuallyDrop;

use crate::domain::material::def::{Material, MaterialContainer, MaterialId, MaterialKind};
use crate::domain::material::primitive::{
    Blurry, Diffuse, Emissive, Glossy, Refractive, Scattering, Specular,
};

#[derive(Debug, Default)]
pub struct MaterialPool {
    blurry: Vec<Blurry>,
    diffuse: Vec<Diffuse>,
    emissive: Vec<Emissive>,
    glossy: Vec<Glossy>,
    refractive: Vec<Refractive>,
    scattering: Vec<Scattering>,
    specular: Vec<Specular>,
}

impl MaterialPool {
    fn downcast_and_push<MI, M>(material: MI, collection: &mut Vec<M>) -> u32
    where
        MI: Material + Any,
        M: Material + Any,
    {
        assert_eq!(TypeId::of::<M>(), material.type_id());
        // SAFETY: Already checked that M == impl Material + Any.
        let material = unsafe { std::mem::transmute_copy(&ManuallyDrop::new(material)) };

        collection.push(material);
        collection.len() as u32 - 1
    }

    fn upcast<M: Material>(material: &M) -> &dyn Material {
        material
    }
}

impl MaterialContainer for MaterialPool {
    fn add_material<M>(&mut self, material: M) -> MaterialId
    where
        M: Material + Any,
    {
        let kind = material.kind();
        let type_id = TypeId::of::<M>();

        if type_id == TypeId::of::<Blurry>() {
            let index = Self::downcast_and_push(material, &mut self.blurry);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Diffuse>() {
            let index = Self::downcast_and_push(material, &mut self.diffuse);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Emissive>() {
            let index = Self::downcast_and_push(material, &mut self.emissive);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Glossy>() {
            let index = Self::downcast_and_push(material, &mut self.glossy);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Refractive>() {
            let index = Self::downcast_and_push(material, &mut self.refractive);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Scattering>() {
            let index = Self::downcast_and_push(material, &mut self.scattering);
            MaterialId::new(kind, index)
        } else if type_id == TypeId::of::<Specular>() {
            let index = Self::downcast_and_push(material, &mut self.specular);
            MaterialId::new(kind, index)
        } else {
            unreachable!("all Material's subtypes should be exhausted")
        }
    }

    fn get_material(&self, material_id: MaterialId) -> Option<&dyn Material> {
        let index = material_id.index() as usize;
        match material_id.kind() {
            MaterialKind::Blurry => self.blurry.get(index).map(Self::upcast),
            MaterialKind::Diffuse => self.diffuse.get(index).map(Self::upcast),
            MaterialKind::Emissive => self.emissive.get(index).map(Self::upcast),
            MaterialKind::Glossy => self.glossy.get(index).map(Self::upcast),
            MaterialKind::Refractive => self.refractive.get(index).map(Self::upcast),
            MaterialKind::Scattering => self.scattering.get(index).map(Self::upcast),
            MaterialKind::Specular => self.specular.get(index).map(Self::upcast),
        }
    }
}
