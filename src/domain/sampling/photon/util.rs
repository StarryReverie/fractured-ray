use rand::prelude::*;

use crate::domain::color::Spectrum;
use crate::domain::material::primitive::Emissive;
use crate::domain::math::algebra::UnitVector;
use crate::domain::math::algebra::Vector;
use crate::domain::math::geometry::Frame;
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
    fn radiance(&self) -> Spectrum {
        Spectrum::zero()
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
        let area = inner.shape().map_or(Val(0.0), |shape| shape.area());
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
    fn radiance(&self) -> Spectrum {
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
        let beam_angle = self.emissive.beam_angle();
        let (dir, pdf_dir_div_cos) = if beam_angle.is_hemisphere() {
            let dir = UnitVector::random_cosine_hemisphere(normal, rng);
            (dir, Val::FRAC_1_PI)
        } else if beam_angle.is_directional() {
            (normal, Val(1.0))
        } else {
            let sin2_beam = beam_angle.angle().sin().powi(2);
            let (u1_sin, u2) = (Val(rng.random()) * sin2_beam, Val(rng.random()));
            let (sin_theta, cos_theta) = (u1_sin.sqrt(), (Val(1.0) - u1_sin).sqrt());
            let (sin_phi, cos_phi) = (Val(0.5) * u2 * Val::FRAC_1_PI).sin_cos();
            let (x, y, z) = (sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
            let local_dir = Vector::new(x, y, z).normalize().unwrap();

            let frame = Frame::new(normal);
            let dir = frame.to_canonical_unit(local_dir);
            (dir, Val::FRAC_1_PI / sin2_beam)
        };

        let ray = Ray::new(point, dir);
        let throughput = self.radiance() / (pdf_point * pdf_dir_div_cos);
        let photon = PhotonRay::new(ray, throughput);
        Some(PhotonSample::new(photon))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::geometry::{Point, SpreadAngle};
    use crate::domain::sampling::point::TrianglePointSampler;
    use crate::domain::shape::def::ShapeKind;
    use crate::domain::shape::primitive::Triangle;
    use crate::domain::shape::util::ShapeId;

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
            Emissive::new(Spectrum::broadcast(Val(1.0)), SpreadAngle::hemisphere()),
        );

        let photon = sampler.sample_photon(&mut rand::rng()).unwrap();
        assert_eq!(photon.photon().throughput().red(), Val::PI * Val(0.5));
        assert_eq!(photon.photon().throughput().green(), Val::PI * Val(0.5));
        assert_eq!(photon.photon().throughput().blue(), Val::PI * Val(0.5));
    }
}
