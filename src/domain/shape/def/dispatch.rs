use enum_dispatch::enum_dispatch;

use crate::domain::material::primitive::Emissive;
use crate::domain::math::geometry::{Area, Normal, Point};
use crate::domain::math::numeric::DisRange;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayIntersectionPart};
use crate::domain::sampling::Sampleable;
use crate::domain::sampling::light::LightSampling;
use crate::domain::sampling::photon::PhotonSampling;
use crate::domain::sampling::point::PointSampling;
use crate::domain::shape::primitive::*;
use crate::domain::shape::util::{Instance, ShapeId};

use super::{BoundingBox, Shape, ShapeKind};

macro_rules! impl_dispatch {
    ($type:tt, $self:ident.$method:ident($($arg:ident),*)) => {
        match $self {
            $type::Aabb(s) => s.$method($($arg),*),
            $type::MeshPolygon(s) => s.$method($($arg),*),
            $type::MeshTriangle(s) => s.$method($($arg),*),
            $type::Plane(s) => s.$method($($arg),*),
            $type::Polygon(s) => s.$method($($arg),*),
            $type::Sphere(s) => s.$method($($arg),*),
            $type::Triangle(s) => s.$method($($arg),*),
            $type::Instance(s) => s.$method($($arg),*),
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

#[enum_dispatch(Shape)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynShape {
    Aabb(Aabb),
    MeshPolygon(MeshPolygon),
    MeshTriangle(MeshTriangle),
    Plane(Plane),
    Polygon(Polygon),
    Sphere(Sphere),
    Triangle(Triangle),
    Instance(Instance),
}

impl<'a> From<&'a DynShape> for RefDynShape<'a> {
    fn from(value: &'a DynShape) -> Self {
        impl_dispatch!(DynShape, value.into())
    }
}

impl Sampleable for DynShape {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        impl_dispatch!(Self, self.get_point_sampler(shape_id))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        impl_dispatch!(Self, self.get_light_sampler(shape_id))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        impl_dispatch!(Self, self.get_photon_sampler(shape_id, emissive))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefDynShape<'a> {
    Aabb(&'a Aabb),
    MeshPolygon(&'a MeshPolygon),
    MeshTriangle(&'a MeshTriangle),
    Plane(&'a Plane),
    Polygon(&'a Polygon),
    Sphere(&'a Sphere),
    Triangle(&'a Triangle),
    Instance(&'a Instance),
}

impl<'a> Shape for RefDynShape<'a> {
    fn kind(&self) -> ShapeKind {
        impl_dispatch!(Self, self.kind())
    }

    fn hit_part<'b>(&self, ray: &'b Ray, range: DisRange) -> Option<RayIntersectionPart<'b>> {
        impl_dispatch!(Self, self.hit_part(ray, range))
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        impl_dispatch!(Self, self.hit(ray, range))
    }

    fn hit_all(&self, ray: &Ray, range: DisRange) -> Vec<RayIntersection> {
        impl_dispatch!(Self, self.hit_all(ray, range))
    }

    fn complete_part(&self, part: RayIntersectionPart) -> RayIntersection {
        impl_dispatch!(Self, self.complete_part(part))
    }

    fn area(&self) -> Area {
        impl_dispatch!(Self, self.area())
    }

    fn normal(&self, position: Point) -> Normal {
        impl_dispatch!(Self, self.normal(position))
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        impl_dispatch!(Self, self.bounding_box())
    }
}

impl<'a> Sampleable for RefDynShape<'a> {
    fn get_point_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn PointSampling>> {
        impl_dispatch!(Self, self.get_point_sampler(shape_id))
    }

    fn get_light_sampler(&self, shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        impl_dispatch!(Self, self.get_light_sampler(shape_id))
    }

    fn get_photon_sampler(
        &self,
        shape_id: ShapeId,
        emissive: Emissive,
    ) -> Option<Box<dyn PhotonSampling>> {
        impl_dispatch!(Self, self.get_photon_sampler(shape_id, emissive))
    }
}

impl_from_ref_for_variant!('a, RefDynShape<'a>, Aabb);
impl_from_ref_for_variant!('a, RefDynShape<'a>, MeshPolygon);
impl_from_ref_for_variant!('a, RefDynShape<'a>, MeshTriangle);
impl_from_ref_for_variant!('a, RefDynShape<'a>, Plane);
impl_from_ref_for_variant!('a, RefDynShape<'a>, Polygon);
impl_from_ref_for_variant!('a, RefDynShape<'a>, Sphere);
impl_from_ref_for_variant!('a, RefDynShape<'a>, Triangle);
impl_from_ref_for_variant!('a, RefDynShape<'a>, Instance);
