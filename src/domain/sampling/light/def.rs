use std::fmt::Debug;

use getset::{CopyGetters, Getters};
use rand::prelude::*;

use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Direction, Normal, Point};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::math::transformation::{AtomTransformation, Transform};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RayScattering};
use crate::domain::sampling::point::{PointSample, PointSampling};
use crate::domain::shape::def::{RefDynShape, Shape};
use crate::domain::shape::util::ShapeId;

pub trait LightSampling: Debug + Send + Sync {
    fn id(&self) -> Option<ShapeId>;

    fn shape(&self) -> Option<RefDynShape>;

    fn sample_light_surface(
        &self,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample>;

    fn pdf_light_surface(&self, intersection: &RayIntersection, ray_next: &Ray) -> Val;

    fn sample_light_volume(
        &self,
        scattering: &RayScattering,
        preselected_light: Option<&PointSample>,
        rng: &mut dyn RngCore,
    ) -> Option<LightSample>;

    fn pdf_light_volume(&self, ray_next: &Ray, preselected_light: Option<&PointSample>) -> Val;

    fn has_nonzero_prob_given_preselected_light(
        &self,
        ray_next: &Ray,
        light: &PointSample,
    ) -> bool {
        if self.id().is_none_or(|id| id != light.shape_id()) {
            return false;
        }
        Direction::normalize(light.point() - ray_next.start())
            .is_ok_and(|dir| dir == ray_next.direction())
    }
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct LightSample {
    #[getset(get = "pub")]
    ray_next: Ray,
    #[getset(get_copy = "pub")]
    pdf: Val,
    #[getset(get_copy = "pub")]
    distance: Val,
    #[getset(get_copy = "pub")]
    shape_id: ShapeId,
}

impl LightSample {
    pub fn new(ray_next: Ray, pdf: Val, distance: Val, shape_id: ShapeId) -> Self {
        Self {
            ray_next,
            pdf,
            distance,
            shape_id,
        }
    }

    pub fn scale_pdf(self, multiplier: Val) -> Self {
        Self {
            pdf: self.pdf * multiplier,
            ..self
        }
    }

    pub fn convert_point_sample<RS>(
        position: Point,
        sample: &PointSample,
        ray_spawner: RS,
    ) -> Option<Self>
    where
        RS: Fn(Direction) -> Ray,
    {
        let Ok(direction) = Direction::normalize(sample.point() - position) else {
            return None;
        };
        let ray_next = ray_spawner(direction);

        let cos = sample.normal().dot(direction).abs();
        let dis_squared = (sample.point() - position).norm_squared();
        let pdf = sample.pdf() * dis_squared / cos;
        let distance = (sample.point() - position).norm();
        Some(LightSample::new(ray_next, pdf, distance, sample.shape_id()))
    }

    pub fn convert_point_pdf<PS>(position: Point, ray_next: &Ray, point_sampler: &PS) -> Val
    where
        PS: PointSampling,
    {
        let Some(shape) = &point_sampler.shape() else {
            return Val(0.0);
        };
        if let Some(intersection_next) = shape.hit(ray_next, DisRange::positive()) {
            let position_next = intersection_next.position();
            Self::point_pdf_to_solid_angle_pdf(
                position,
                ray_next.direction(),
                position_next,
                shape.normal(position_next),
                point_sampler.pdf_point(position_next, true),
            )
        } else {
            Val(0.0)
        }
    }

    #[inline]
    pub fn point_pdf_to_solid_angle_pdf(
        position: Point,
        direction_next: Direction,
        position_next: Point,
        normal_next: Normal,
        pdf_point: Val,
    ) -> Val {
        let cos = normal_next.dot(direction_next).abs();
        let dis_squared = (position_next - position).norm_squared();
        pdf_point * dis_squared / cos
    }
}

impl<T> Transform<T> for LightSample
where
    T: AtomTransformation,
    Ray: Transform<T>,
{
    fn transform(&self, transformation: &T) -> Self {
        LightSample::new(
            self.ray_next.transform(transformation),
            self.pdf,
            self.distance,
            self.shape_id,
        )
    }
}
