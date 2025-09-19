use rand::prelude::*;

use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Distance, Point};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayScattering, RaySegment};

use super::{DistanceSample, DistanceSampling};

#[derive(Debug, Clone)]
pub struct EquiAngularDistanceSampler {
    vertex: Point,
}

impl EquiAngularDistanceSampler {
    pub fn new(vertex: Point) -> Self {
        Self { vertex }
    }
}

impl DistanceSampling for EquiAngularDistanceSampler {
    fn sample_distance(
        &self,
        ray: &Ray,
        segment: &RaySegment,
        rng: &mut dyn RngCore,
    ) -> DistanceSample {
        let (start, end) = (segment.start(), segment.end());

        let to_vertex = self.vertex - ray.start();
        let vertex_proj = to_vertex.dot(ray.direction());
        let perp_dis = (to_vertex.norm_squared() - vertex_proj.powi(2)).sqrt();

        let angle_start = ((start - vertex_proj) / perp_dis).atan();
        let angle_end = ((end - vertex_proj) / perp_dis).atan();

        let u = Val(rng.random());
        let angle_sample = Val::lerp(angle_start, angle_end, u);
        let bottom_len = perp_dis * angle_sample.tan();
        let distance = Distance::new(vertex_proj + bottom_len).unwrap();

        let scattering = RayScattering::new(distance, ray.at(distance));
        let pdf = perp_dis / ((angle_end - angle_start) * (perp_dis.powi(2) + bottom_len.powi(2)));
        DistanceSample::new(scattering, pdf)
    }

    fn pdf_distance(&self, ray: &Ray, segment: &RaySegment, distance: Distance) -> Val {
        if segment.contains(distance) {
            let (start, end) = (segment.start(), segment.end());

            let to_vertex = self.vertex - ray.start();
            let vertex_proj = to_vertex.dot(ray.direction());
            let perp_dis = (to_vertex.norm_squared() - vertex_proj.powi(2)).sqrt();

            let angle_start = ((start - vertex_proj) / perp_dis).atan();
            let angle_end = ((end - vertex_proj) / perp_dis).atan();

            let bottom_len = distance - vertex_proj;
            perp_dis / ((angle_end - angle_start) * (perp_dis.powi(2) + bottom_len.powi(2)))
        } else {
            Val(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::Direction;

    use super::*;

    #[test]
    fn equi_angular_distance_sampler_pdf_distance_succeeds() {
        let sampler = EquiAngularDistanceSampler::new(Point::new(Val(0.0), Val(0.0), Val(0.0)));
        let ray = Ray::new(
            Point::new(Val(0.0), Val(-1.0), Val(1.0)),
            Direction::y_direction(),
        );

        let segment = RaySegment::new(
            Distance::new(Val(0.5)).unwrap(),
            Distance::new(Val(2.5)).unwrap(),
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(1.0)).unwrap()),
            Val(0.63661977),
        );
        assert_eq!(
            sampler.pdf_distance(&ray, &segment, Distance::new(Val(0.2)).unwrap()),
            Val(0.0)
        );
    }
}
