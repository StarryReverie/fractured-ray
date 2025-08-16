use std::any::Any;

use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::util as ray_util;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{
    Contribution, PmContext, PmState, RtContext, RtState, StoragePolicy,
};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

use super::MicrofacetMaterial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blurry {
    albedo: Albedo,
    refractive_index: Val,
    alpha: Val,
}

impl Blurry {
    pub fn new(
        albedo: Albedo,
        refractive_index: Val,
        roughness: Val,
    ) -> Result<Self, TryNewBlurryError> {
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);
        ensure!(
            Val(0.0) < roughness && roughness <= Val(1.0),
            InvalidRoughnessSnafu,
        );
        Ok(Self {
            albedo,
            refractive_index,
            alpha: roughness.powi(2),
        })
    }

    fn calc_current_refractive_index(&self, side: SurfaceSide) -> Val {
        if side == SurfaceSide::Front {
            self.refractive_index
        } else {
            self.refractive_index.recip()
        }
    }
}

impl Material for Blurry {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Blurry
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        let light = self.shade_light(context, &ray, &intersection);
        let state_next = state.with_skip_emissive(true);
        let mut res = self.shade_scattering(context, state_next, &ray, &intersection);
        res.add_light(light.light());
        res
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        match state.policy() {
            StoragePolicy::Global => {
                self.maybe_bounce_next_photon(context, state, photon, intersection);
            }
            StoragePolicy::Caustic => {}
        }
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl MicrofacetMaterial for Blurry {
    fn r0(&self, side: SurfaceSide) -> Spectrum {
        let ri = if side == SurfaceSide::Front {
            self.refractive_index
        } else {
            self.refractive_index.recip()
        };
        let r0 = ((Val(1.0) - ri) / (Val(1.0) + ri)).powi(2);
        Spectrum::broadcast(r0)
    }

    fn alpha(&self) -> Val {
        self.alpha
    }
}

impl BsdfMaterial for Blurry {
    fn bsdf(
        &self,
        dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Spectrum {
        let normal = intersection.normal();
        let side = intersection.side();
        let ri = self.calc_current_refractive_index(side);

        if dir_in.dot(normal) > Val(0.0) {
            let Ok(mn) = (dir_out + dir_in).normalize() else {
                return Spectrum::zero();
            };

            let reflectance = self.calc_reflectance(dir_out.dot(mn), side);
            let reflectance = reflectance.channel(0).min(Val(1.0));

            let ndf = self.calc_ndf(normal, mn);
            let g2 = self.calc_g2(dir_out, dir_in, normal);
            let (cos, cos_next) = (dir_out.dot(normal), dir_in.dot(normal));

            self.albedo * (reflectance * ndf * g2) / (Val(4.0) * cos * cos_next).abs()
        } else {
            let Ok(mn) = (-dir_out - ri * dir_in).normalize() else {
                return Spectrum::zero();
            };

            let reflectance = self.calc_reflectance(dir_out.dot(mn), side);
            let transmittance = Val(1.0) - reflectance.channel(0).min(Val(1.0));

            let ndf = self.calc_ndf(normal, mn);
            let g2 = self.calc_g2(dir_out, dir_in, normal);
            let (cos, cos_next) = (dir_out.dot(normal), dir_in.dot(normal));
            let (cos_mn, cos_mn_next) = (dir_out.dot(mn), dir_in.dot(mn));

            let cos_term = ((cos_mn * cos_mn_next) / (cos * cos_next)).abs();
            let denominator = (cos_mn.abs() / ri + cos_mn_next.abs()).powi(2);
            self.albedo * cos_term * transmittance * ndf * g2 / denominator
        }
    }
}

impl BsdfSampling for Blurry {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let dir = -ray.direction();
        let normal = intersection.normal();
        let ri = self.calc_current_refractive_index(intersection.side());
        let mn = self.generate_microfacet_normal(dir, normal, rng);

        let (ray_next, scatter_kind) =
            ray_util::fresnel_refract_microfacet(ray, intersection, mn, ri, rng);
        let reflectance = scatter_kind.reflectance();
        let dir_next = ray_next.direction();

        let g2 = self.calc_g2(dir, dir_next, normal);
        let g1 = self.calc_g1(dir, normal);
        let coefficient = if scatter_kind.is_reflective() {
            self.albedo * g2 / g1
        } else {
            let (o_mn, i_mn) = (mn.dot(dir).abs(), mn.dot(dir_next).abs());
            let extra = o_mn.abs() * i_mn.abs() * Val(4.0) / (o_mn / ri + i_mn).powi(2);
            self.albedo * extra * g2 / g1
        };

        let ndf = self.calc_ndf(normal, mn);
        let pdf_vndf = g1 * ndf * Val(0.25) / dir.dot(normal);
        let pdf = if scatter_kind.is_reflective() {
            reflectance * pdf_vndf
        } else {
            (Val(1.0) - reflectance) * pdf_vndf
        };

        BsdfSample::new(ray_next, coefficient, pdf)
    }

    fn pdf_bsdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let side = intersection.side();
        let ri = self.calc_current_refractive_index(side);
        let (dir, dir_next) = (-ray.direction(), ray_next.direction());

        let normal = intersection.normal();
        let (mn, is_reflective) = if dir_next.dot(normal) > Val(0.0) {
            let Ok(mn) = (dir + dir_next).normalize() else {
                return Val(0.0);
            };
            (mn, true)
        } else {
            let Ok(mn) = (-dir - ri * dir_next).normalize() else {
                return Val(0.0);
            };
            (mn, false)
        };

        let reflectance = self.calc_reflectance(dir.dot(normal), side);
        let reflectance = reflectance.channel(0).min(Val(1.0));

        let g1 = self.calc_g1(dir, normal);
        let ndf = self.calc_ndf(normal, mn);
        let pdf_vndf = g1 * ndf * Val(0.25) / dir.dot(normal);
        if is_reflective {
            reflectance * pdf_vndf
        } else {
            (Val(1.0) - reflectance) * pdf_vndf
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewBlurryError {
    #[snafu(display("refractive index is not positive"))]
    InvalidRefractiveIndex,
    #[snafu(display("roughness should be in (0, 1]"))]
    InvalidRoughness,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blurry_new_fails_when_refractive_index_is_not_positive() {
        assert!(matches!(
            Blurry::new(Albedo::WHITE, Val(0.0), Val(0.5)),
            Err(TryNewBlurryError::InvalidRefractiveIndex),
        ));
        assert!(matches!(
            Blurry::new(Albedo::WHITE, Val(-1.0), Val(0.5)),
            Err(TryNewBlurryError::InvalidRefractiveIndex),
        ));
    }

    #[test]
    fn blurry_new_fails_when_roughness_is_invalid() {
        assert!(matches!(
            Blurry::new(Albedo::WHITE, Val(2.0), Val(-1.0)),
            Err(TryNewBlurryError::InvalidRoughness),
        ));
        assert!(matches!(
            Blurry::new(Albedo::WHITE, Val(2.0), Val(1.5)),
            Err(TryNewBlurryError::InvalidRoughness),
        ));
    }
}
