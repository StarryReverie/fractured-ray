use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::math::algebra::{UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::shape::def::Shape;
use crate::domain::shape::primitive::Aabb;
use crate::domain::shape::util::ShapeId;

use super::{PointSample, PointSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct AabbPointSampler {
    id: ShapeId,
    aabb: Aabb,
    area_inv: Val,
    face_sampler: WeightedIndex<WrappedVal>,
}

impl AabbPointSampler {
    const POSITIVE_X: usize = 0;
    const NEGATIVE_X: usize = 1;
    const POSITIVE_Y: usize = 2;
    const NEGATIVE_Y: usize = 3;
    const POSITIVE_Z: usize = 4;
    const NEGATIVE_Z: usize = 5;

    pub fn new(id: ShapeId, aabb: Aabb) -> Self {
        let diag = aabb.max() - aabb.min();
        let area_inv = aabb.area().recip();

        let mut areas = [0.0; 6];
        for (i, area) in areas.iter_mut().enumerate() {
            *area = Self::calc_face_area(i, diag).0;
        }
        let face_sampler = WeightedIndex::new(areas).unwrap();

        Self {
            id,
            aabb,
            area_inv,
            face_sampler,
        }
    }

    fn calc_face_area(face_index: usize, diag: Vector) -> Val {
        match face_index {
            Self::POSITIVE_X | Self::NEGATIVE_X => diag.y() * diag.z(),
            Self::POSITIVE_Y | Self::NEGATIVE_Y => diag.x() * diag.z(),
            Self::POSITIVE_Z | Self::NEGATIVE_Z => diag.x() * diag.y(),
            _ => unreachable!("face_index should be in [0, 6)"),
        }
    }

    fn sample_point_for_face(&self, face_index: usize, rng: &mut dyn RngCore) -> Point {
        match face_index {
            Self::POSITIVE_X => {
                let (y, z) = (Val(rng.random()), Val(rng.random()));
                Point::new(self.aabb.max().x(), y, z)
            }
            Self::NEGATIVE_X => {
                let (y, z) = (Val(rng.random()), Val(rng.random()));
                Point::new(self.aabb.min().x(), y, z)
            }
            Self::POSITIVE_Y => {
                let (x, z) = (Val(rng.random()), Val(rng.random()));
                Point::new(x, self.aabb.max().y(), z)
            }
            Self::NEGATIVE_Y => {
                let (x, z) = (Val(rng.random()), Val(rng.random()));
                Point::new(x, self.aabb.min().y(), z)
            }
            Self::POSITIVE_Z => {
                let (x, y) = (Val(rng.random()), Val(rng.random()));
                Point::new(x, y, self.aabb.max().z())
            }
            Self::NEGATIVE_Z => {
                let (x, y) = (Val(rng.random()), Val(rng.random()));
                Point::new(x, y, self.aabb.min().z())
            }
            _ => unreachable!("face_index should be in [0, 6)"),
        }
    }

    fn get_normal_for_face(face_index: usize) -> UnitVector {
        match face_index {
            Self::POSITIVE_X => UnitVector::x_direction(),
            Self::NEGATIVE_X => -UnitVector::x_direction(),
            Self::POSITIVE_Y => UnitVector::y_direction(),
            Self::NEGATIVE_Y => -UnitVector::y_direction(),
            Self::POSITIVE_Z => UnitVector::z_direction(),
            Self::NEGATIVE_Z => -UnitVector::z_direction(),
            _ => unreachable!("face_index should be in [0, 6)"),
        }
    }
}

impl PointSampling for AabbPointSampler {
    fn id(&self) -> Option<ShapeId> {
        Some(self.id)
    }

    fn shape(&self) -> Option<&dyn Shape> {
        Some(&self.aabb)
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let face_index = self.face_sampler.sample(rng);
        let point = self.sample_point_for_face(face_index, rng);
        let normal = Self::get_normal_for_face(face_index);
        Some(PointSample::new(point, normal, self.area_inv, self.id))
    }

    fn pdf_point(&self, point: Point, checked_inside: bool) -> Val {
        if checked_inside {
            self.area_inv
        } else {
            let (min, max) = (self.aabb.min(), self.aabb.max());

            let equals_x = point.x() == min.x() || point.x() == max.x();
            let equals_y = point.y() == min.y() || point.y() == max.y();
            let equals_z = point.z() == min.z() || point.z() == max.z();
            let contains_x = (min.x()..=max.x()).contains(&point.x());
            let contains_y = (min.y()..=max.y()).contains(&point.y());
            let contains_z = (min.z()..=max.z()).contains(&point.z());

            let on_face = (equals_x && contains_y && contains_z)
                || (equals_y && contains_x && contains_z)
                || (equals_z && contains_x && contains_y);
            if on_face { self.area_inv } else { Val(0.0) }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::shape::def::ShapeKind;

    use super::*;

    #[test]
    fn aabb_point_sampler_pdf_point_succeeds() {
        let aabb = Aabb::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(1.0), Val(2.0), Val(3.0)),
        );
        let pdf = aabb.area().recip();
        let sampler = AabbPointSampler::new(ShapeId::new(ShapeKind::Aabb, 0), aabb);

        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(0.0)), false),
            pdf,
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(1.0), Val(0.0)), false),
            pdf,
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(-1.0), Val(0.0), Val(0.0)), false),
            Val(0.0),
        );
    }
}
