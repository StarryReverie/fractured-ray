use rand::prelude::*;
use rand_distr::weighted::WeightedIndex;

use crate::domain::color::Color;
use crate::domain::math::numeric::{Val, WrappedVal};

use super::{EmptyPhotonSampler, PhotonSample, PhotonSampling};

#[derive(Debug)]
pub struct AggregatePhotonSampler {
    samplers: Vec<Box<dyn PhotonSampling>>,
    weights: Vec<Val>,
    index_sampler: WeightedIndex<WrappedVal>,
}

impl AggregatePhotonSampler {
    pub fn new(mut samplers: Vec<Box<dyn PhotonSampling>>) -> Self {
        if samplers.is_empty() {
            samplers.push(Box::new(EmptyPhotonSampler::new()));
        }
        let weights = (samplers.iter())
            .map(|sampler| sampler.radiance().to_vector().norm() * sampler.area())
            .map(|weight| weight.0.max(Val::PRECISION))
            .collect::<Vec<_>>();
        let index_sampler = WeightedIndex::new(weights).unwrap();
        let weights = (index_sampler.weights())
            .map(|weight| Val(weight / index_sampler.total_weight()))
            .collect();
        Self {
            samplers,
            weights,
            index_sampler,
        }
    }
}

impl PhotonSampling for AggregatePhotonSampler {
    fn radiance(&self) -> Color {
        unimplemented!("AggregatePhotonSampler doesn't have a unique radiance")
    }

    fn area(&self) -> Val {
        unimplemented!("AggregatePhotonSampler doesn't have a unique area")
    }

    fn sample_photon(&self, rng: &mut dyn RngCore) -> Option<PhotonSample> {
        let which = self.index_sampler.sample(rng);
        (self.samplers.get(which))
            .and_then(|sampler| sampler.sample_photon(rng))
            .map(|sample| sample.scale_pdf(self.weights[which]))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::material::primitive::Emissive;
    use crate::domain::math::geometry::{Point, SpreadAngle};
    use crate::domain::sampling::photon::PhotonSamplerAdapter;
    use crate::domain::sampling::point::TrianglePointSampler;
    use crate::domain::shape::def::{ShapeId, ShapeKind};
    use crate::domain::shape::primitive::Triangle;

    use super::*;

    #[test]
    fn aggregate_photon_sampler_sample_photon_succeeds() {
        let sampler1: Box<dyn PhotonSampling> = Box::new(PhotonSamplerAdapter::new(
            TrianglePointSampler::new(
                ShapeId::new(ShapeKind::Triangle, 0),
                Triangle::new(
                    Point::new(Val(0.0), Val(0.0), Val(0.0)),
                    Point::new(Val(1.0), Val(0.0), Val(0.0)),
                    Point::new(Val(0.0), Val(1.0), Val(0.0)),
                )
                .unwrap(),
            ),
            Emissive::new(Color::WHITE, SpreadAngle::hemisphere()),
        ));
        let sampler2: Box<dyn PhotonSampling> = Box::new(PhotonSamplerAdapter::new(
            TrianglePointSampler::new(
                ShapeId::new(ShapeKind::Triangle, 0),
                Triangle::new(
                    Point::new(Val(0.0), Val(0.0), Val(0.0)),
                    Point::new(Val(2.0), Val(0.0), Val(0.0)),
                    Point::new(Val(0.0), Val(2.0), Val(0.0)),
                )
                .unwrap(),
            ),
            Emissive::new(Color::WHITE * Val(0.25), SpreadAngle::hemisphere()),
        ));
        let sampler = AggregatePhotonSampler::new(vec![sampler1, sampler2]);

        let photon = sampler.sample_photon(&mut rand::rng()).unwrap();
        assert_eq!(photon.photon().throughput().x(), Val::PI);
        assert_eq!(photon.photon().throughput().y(), Val::PI);
        assert_eq!(photon.photon().throughput().z(), Val::PI);
    }
}
