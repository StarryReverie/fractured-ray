use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::sampling::point::PointSampling;

use super::{PhotonSample, PhotonSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyPhotonSampler {}

impl EmptyPhotonSampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl PhotonSampling for EmptyPhotonSampler {
    fn radiance(&self) -> Color {
        Color::BLACK
    }

    fn area(&self) -> Val {
        Val(0.0)
    }

    fn sample_photon(&self, _rng: &mut dyn RngCore) -> Option<PhotonSample> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhotonSamplerAdapter<PS>
where
    PS: PointSampling,
{
    inner: PS,
    emissive: Emissive,
    area: Val,
}

impl<PS> PhotonSamplerAdapter<PS>
where
    PS: PointSampling,
{
    pub fn new(inner: PS, emissive: Emissive) -> Self {
        let area = inner.shape().map(|shape| shape.area()).unwrap_or(Val(0.0));
        Self {
            inner,
            emissive,
            area,
        }
    }
}

impl<PS> PhotonSampling for PhotonSamplerAdapter<PS>
where
    PS: PointSampling,
{
    fn radiance(&self) -> Color {
        self.emissive.radiance()
    }

    fn area(&self) -> Val {
        self.area
    }

    fn sample_photon(&self, rng: &mut dyn RngCore) -> Option<PhotonSample> {
        let sample = self.inner.sample_point(rng)?;
        let point = sample.point();
        let pdf_point = sample.pdf();

        let normal = sample.normal();
        let dir = UnitVector::random_cosine_hemisphere(normal, rng);

        let ray = Ray::new(point, dir);
        let throughput = self.radiance().to_vector() * Val::PI / pdf_point;
        let photon = PhotonRay::new(ray, throughput);
        Some(PhotonSample::new(photon))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::Point;
    use crate::domain::sampling::point::TrianglePointSampler;
    use crate::domain::shape::def::{ShapeId, ShapeKind};
    use crate::domain::shape::primitive::Triangle;

    use super::*;

    #[test]
    fn photon_sampler_adapter_sample_photon_succeeds() {
        let sampler = PhotonSamplerAdapter::new(
            TrianglePointSampler::new(
                ShapeId::new(ShapeKind::Triangle, 0),
                Triangle::new(
                    Point::new(Val(0.0), Val(0.0), Val(0.0)),
                    Point::new(Val(1.0), Val(0.0), Val(0.0)),
                    Point::new(Val(0.0), Val(1.0), Val(0.0)),
                )
                .unwrap(),
            ),
            Emissive::new(Color::WHITE),
        );

        let photon = sampler.sample_photon(&mut rand::rng()).unwrap();
        assert_eq!(photon.photon().throughput().x(), Val::PI * Val(0.5));
        assert_eq!(photon.photon().throughput().y(), Val::PI * Val(0.5));
        assert_eq!(photon.photon().throughput().z(), Val::PI * Val(0.5));
    }
}
