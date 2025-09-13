use enum_dispatch::enum_dispatch;

use crate::domain::color::Spectrum;
use crate::domain::medium::primitive::{HenyeyGreenstein, Isotropic, Vacuum};
use crate::domain::ray::Ray;
use crate::domain::ray::event::RaySegment;
use crate::domain::renderer::{Contribution, RtContext, RtState};

use super::{Medium, MediumKind};

macro_rules! impl_dispatch {
    ($type:tt, $self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            $type::HenyeyGreenstein(s) => s.$method($($arg),*),
            $type::Isotropic(s) => s.$method($($arg),*),
            $type::Vacuum(s) => s.$method($($arg),*),
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

#[enum_dispatch(Medium)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynMedium {
    HenyeyGreenstein(HenyeyGreenstein),
    Isotropic(Isotropic),
    Vacuum(Vacuum),
}

impl<'a> From<&'a DynMedium> for RefDynMedium<'a> {
    fn from(value: &'a DynMedium) -> Self {
        impl_dispatch!(DynMedium, value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefDynMedium<'a> {
    HenyeyGreenstein(&'a HenyeyGreenstein),
    Isotropic(&'a Isotropic),
    Vacuum(&'a Vacuum),
}

impl<'a> Medium for RefDynMedium<'a> {
    fn kind(&self) -> MediumKind {
        impl_dispatch!(Self, self.kind())
    }

    fn transmittance(&self, ray: &Ray, segment: &RaySegment) -> Spectrum {
        impl_dispatch!(Self, self.transmittance(ray, segment))
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        segment: &RaySegment,
    ) -> Contribution {
        impl_dispatch!(Self, self.shade(context, state, ray, segment))
    }
}

impl_from_ref_for_variant!('a, RefDynMedium<'a>, HenyeyGreenstein);
impl_from_ref_for_variant!('a, RefDynMedium<'a>, Isotropic);
impl_from_ref_for_variant!('a, RefDynMedium<'a>, Vacuum);
