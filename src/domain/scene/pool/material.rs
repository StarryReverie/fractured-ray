use std::fmt::Debug;

use crate::domain::material::def::{DynMaterial, Material, MaterialKind, RefDynMaterial};
use crate::domain::material::primitive::*;
use crate::domain::material::util::{MaterialContainer, MaterialId};

#[derive(Debug, Default)]
pub struct MaterialPool {
    blurry: Vec<Blurry>,
    diffuse: Vec<Diffuse>,
    emissive: Vec<Emissive>,
    glossy: Vec<Glossy>,
    refractive: Vec<Refractive>,
    scattering: Vec<Scattering>,
    specular: Vec<Specular>,
    mixed: Vec<Mixed>,
}

impl MaterialPool {
    fn push<M>(material: M, collection: &mut Vec<M>) -> MaterialId
    where
        M: Material,
    {
        let kind = material.kind();
        collection.push(material);
        MaterialId::new(kind, collection.len() as u32 - 1)
    }
}

impl MaterialContainer for MaterialPool {
    fn add_material(&mut self, material: DynMaterial) -> MaterialId {
        match material {
            DynMaterial::Blurry(s) => Self::push(s, &mut self.blurry),
            DynMaterial::Diffuse(s) => Self::push(s, &mut self.diffuse),
            DynMaterial::Emissive(s) => Self::push(s, &mut self.emissive),
            DynMaterial::Glossy(s) => Self::push(s, &mut self.glossy),
            DynMaterial::Refractive(s) => Self::push(s, &mut self.refractive),
            DynMaterial::Scattering(s) => Self::push(s, &mut self.scattering),
            DynMaterial::Specular(s) => Self::push(s, &mut self.specular),
            DynMaterial::Mixed(s) => Self::push(s, &mut self.mixed),
        }
    }

    fn get_material(&self, material_id: MaterialId) -> Option<RefDynMaterial<'_>> {
        let index = material_id.index() as usize;
        match material_id.kind() {
            MaterialKind::Blurry => self.blurry.get(index).map(Into::into),
            MaterialKind::Diffuse => self.diffuse.get(index).map(Into::into),
            MaterialKind::Emissive => self.emissive.get(index).map(Into::into),
            MaterialKind::Glossy => self.glossy.get(index).map(Into::into),
            MaterialKind::Refractive => self.refractive.get(index).map(Into::into),
            MaterialKind::Scattering => self.scattering.get(index).map(Into::into),
            MaterialKind::Specular => self.specular.get(index).map(Into::into),
            MaterialKind::Mixed => self.mixed.get(index).map(Into::into),
        }
    }
}
