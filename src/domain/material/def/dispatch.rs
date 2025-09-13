use enum_dispatch::enum_dispatch;

use crate::domain::material::primitive::*;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};

use super::{Material, MaterialKind};

macro_rules! impl_dispatch {
    ($type:tt, $self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            $type::Blurry(s) => s.$method($($arg),*),
            $type::Diffuse(s) => s.$method($($arg),*),
            $type::Emissive(s) => s.$method($($arg),*),
            $type::Glossy(s) => s.$method($($arg),*),
            $type::Refractive(s) => s.$method($($arg),*),
            $type::Scattering(s) => s.$method($($arg),*),
            $type::Specular(s) => s.$method($($arg),*),
        }
    };
}

macro_rules! impl_from_ref_for_variant {
    ($lifetime:tt, $ref:ty, $owned:tt) => {
        impl<$lifetime> From<&'a $owned> for $ref {
            fn from(value: &'a $owned) -> Self {
                Self::$owned(value)
            }
        }
    };
}

#[enum_dispatch(Material)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynMaterial {
    Blurry(Blurry),
    Diffuse(Diffuse),
    Emissive(Emissive),
    Glossy(Glossy),
    Refractive(Refractive),
    Scattering(Scattering),
    Specular(Specular),
}

impl<'a> From<&'a DynMaterial> for RefDynMaterial<'a> {
    fn from(value: &'a DynMaterial) -> Self {
        impl_dispatch!(DynMaterial, value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefDynMaterial<'a> {
    Blurry(&'a Blurry),
    Diffuse(&'a Diffuse),
    Emissive(&'a Emissive),
    Glossy(&'a Glossy),
    Refractive(&'a Refractive),
    Scattering(&'a Scattering),
    Specular(&'a Specular),
}

impl<'a> Material for RefDynMaterial<'a> {
    fn kind(&self) -> MaterialKind {
        impl_dispatch!(Self, self.kind())
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        impl_dispatch!(Self, self.shade(context, state, ray, intersection))
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        impl_dispatch!(Self, self.receive(context, state, photon, intersection))
    }
}

impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Blurry);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Diffuse);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Emissive);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Glossy);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Refractive);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Scattering);
impl_from_ref_for_variant!('a, RefDynMaterial<'a>, Specular);
