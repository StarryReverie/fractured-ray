use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{Val, WrappedVal};
use crate::domain::shape::def::Shape;
use crate::domain::shape::util::ShapeId;

use super::{PointSample, PointSampling};

#[derive(Debug)]
pub struct AggregatePointSampler {
    samplers: Vec<Box<dyn PointSampling>>,
    weights: Vec<Val>,
    index_sampler: WeightedIndex<WrappedVal>,
}

impl AggregatePointSampler {
    pub fn new(samplers: Vec<(Box<dyn PointSampling>, Val)>) -> Self {
        let weights = samplers
            .iter()
            .map(|(_, Val(w))| w.max(0.0))
            .collect::<Vec<_>>();
        let index_sampler = WeightedIndex::new(weights).unwrap();

        let total = index_sampler.total_weight();
        let weights = index_sampler.weights().map(|w| Val(w / total)).collect();

        let samplers = samplers.into_iter().map(|(s, _)| s).collect();
        Self {
            samplers,
            weights,
            index_sampler,
        }
    }
}

impl PointSampling for AggregatePointSampler {
    fn id(&self) -> Option<ShapeId> {
        unreachable!("AggregatePointSampler::id() doesn't have a unique ID")
    }

    fn shape(&self) -> Option<&dyn Shape> {
        unreachable!("AggregatePointSampler doesn't have a unique inner shape")
    }

    fn sample_point(&self, rng: &mut dyn RngCore) -> Option<PointSample> {
        let which = self.index_sampler.sample(rng);
        (self.samplers.get(which))
            .and_then(|sampler| sampler.sample_point(rng))
            .map(|sample| sample.scale_pdf(self.weights[which]))
    }

    fn pdf_point(&self, point: Point, _checked_inside: bool) -> Val {
        for (i, sampler) in self.samplers.iter().enumerate() {
            let pdf = sampler.pdf_point(point, false);
            if pdf != Val(0.0) {
                return pdf * self.weights[i];
            }
        }
        Val(0.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::sampling::point::TrianglePointSampler;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::primitive::Triangle;

    use super::*;

    #[test]
    fn aggregate_point_sampler_pdf_point_succeeds() {
        let triangle1 = Triangle::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(1.0), Val(0.0)),
        )
        .unwrap();
        let area1 = triangle1.area();
        let sampler1: Box<dyn PointSampling> = Box::new(TrianglePointSampler::new(
            ShapeId::new(ShapeKind::Triangle, 0),
            triangle1,
        ));

        let triangle2 = Triangle::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(2.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(2.0), Val(0.0)),
        )
        .unwrap();
        let area2 = triangle2.area();
        let sampler2: Box<dyn PointSampling> = Box::new(TrianglePointSampler::new(
            ShapeId::new(ShapeKind::Triangle, 1),
            triangle2,
        ));

        let samplers = vec![(sampler1, area1), (sampler2, area2)];
        let sampler = AggregatePointSampler::new(samplers);

        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(1.0)), false),
            Val(0.0)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.0), Val(0.0), Val(0.0)), false),
            Val(0.4)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(-0.5), Val(0.5), Val(0.0)), false),
            Val(0.4)
        );
        assert_eq!(
            sampler.pdf_point(Point::new(Val(0.5), Val(0.5), Val(0.0)), false),
            Val(0.4)
        );
    }
}
