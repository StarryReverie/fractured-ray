use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::core::{Albedo, Color, Spectrum};
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::geometry::{Direction, Frame, Normal};
use crate::domain::math::numeric::Val;
use crate::domain::ray::Ray;
use crate::domain::ray::event::RayIntersection;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::util as ray_util;
use crate::domain::renderer::{
    Contribution, PmContext, PmState, RtContext, RtState, StoragePolicy,
};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};
use crate::domain::texture::def::DynAlbedoTexture;

pub(super) trait MicrofacetMaterial: Material {
    fn r0(&self, intersection: &RayIntersection) -> Spectrum;

    fn alpha(&self) -> Val;

    fn generate_microfacet_normal(
        &self,
        dir: Direction,
        normal: Normal,
        rng: &mut dyn RngCore,
    ) -> Normal {
        let frame = Frame::new(normal);
        let local_dir = frame.to_local_unit(dir.into()).into();
        let local_mn = self.generate_local_microfacet_normal(local_dir, rng);
        frame.to_canonical_unit(local_mn.to_unit_vector()).into()
    }

    fn generate_local_microfacet_normal(
        &self,
        local_dir: Direction,
        rng: &mut dyn RngCore,
    ) -> Normal {
        let alpha = self.alpha();

        let ldir_tr = Vector::new(alpha * local_dir.x(), alpha * local_dir.y(), local_dir.z());

        let r = Val(rng.random()).sqrt();
        let phi = Val(2.0) * Val::PI * Val(rng.random());
        let (t1, t2) = (r * phi.cos(), r * phi.sin());
        let s = Val(0.5) * (Val(1.0) + ldir_tr.z());
        let t2 = (Val(1.0) - s) * (Val(1.0) - t1.powi(2)).sqrt() + s * t2;

        let t3 = (Val(1.0) - t1.powi(2) - t2.powi(2)).max(Val(0.0)).sqrt();
        let mn_tr =
            Frame::new(Normal::normalize(ldir_tr).unwrap()).to_canonical(Vector::new(t1, t2, t3));
        let mn = Vector::new(
            alpha * mn_tr.x(),
            alpha * mn_tr.y(),
            mn_tr.z().max(Val(0.0)),
        );
        Normal::normalize(mn).unwrap()
    }

    fn calc_reflectance(&self, cos: Val, intersection: &RayIntersection) -> Spectrum {
        let r0 = self.r0(intersection);
        r0 + (Spectrum::broadcast(Val(1.0)) - r0) * (Val(1.0) - cos).powi(5)
    }

    fn calc_ndf(&self, normal: Normal, mn: Normal) -> Val {
        let alpha_squared = self.alpha().powi(2);
        let cos = normal.dot(mn);
        alpha_squared / (Val::PI * (cos.powi(2) * (alpha_squared - Val(1.0)) + Val(1.0)).powi(2))
    }

    fn calc_g1(&self, dir: Direction, normal: Normal) -> Val {
        let tan = dir.dot(normal).abs().acos().tan();
        let tmp = (Val(1.0) + (self.alpha() * tan).powi(2)).sqrt();
        Val(2.0) / (Val(1.0) + tmp)
    }

    fn calc_g2(&self, dir: Direction, dir_next: Direction, normal: Normal) -> Val {
        let alpha = self.alpha();
        let tan = dir.dot(normal).abs().acos().tan();
        let tan_next = dir_next.dot(normal).abs().acos().tan();
        let tmp = (Val(1.0) + (alpha * tan).powi(2)).sqrt();
        let tmp_next = (Val(1.0) + (alpha * tan_next).powi(2)).sqrt();
        Val(2.0) / (tmp + tmp_next)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glossy {
    albedo: DynAlbedoTexture,
    metalness: Val,
    alpha: Val,
}

impl Glossy {
    const DIELECTRIC_R0: Spectrum = Spectrum::broadcast(Val(0.04));

    pub fn new<T>(albedo: T, metalness: Val, roughness: Val) -> Result<Self, TryNewGlossyError>
    where
        T: Into<DynAlbedoTexture>,
    {
        ensure!(
            Val(0.0) <= metalness && metalness <= Val(1.0),
            InvalidMetalnessSnafu
        );
        ensure!(
            Val(0.0) < roughness && roughness <= Val(1.0),
            InvalidRoughnessSnafu
        );

        Ok(Self {
            albedo: albedo.into(),
            metalness,
            alpha: roughness.powi(2),
        })
    }

    pub fn lookup(
        predefinition: GlossyPredefinition,
        roughness: Val,
    ) -> Result<Self, TryNewGlossyError> {
        ensure!(
            Val(0.0) < roughness && roughness <= Val(1.0),
            InvalidRoughnessSnafu
        );

        let (r0_r, r0_g, r0_b) = match predefinition {
            GlossyPredefinition::Aluminum => (0.913, 0.922, 0.924),
            GlossyPredefinition::Brass => (0.910, 0.778, 0.423),
            GlossyPredefinition::Chromium => (0.549, 0.556, 0.554),
            GlossyPredefinition::Copper => (0.955, 0.638, 0.538),
            GlossyPredefinition::Gold => (1.000, 0.782, 0.344),
            GlossyPredefinition::Iron => (0.562, 0.565, 0.578),
            GlossyPredefinition::Mercury => (0.781, 0.780, 0.778),
            GlossyPredefinition::Nickel => (0.660, 0.609, 0.526),
            GlossyPredefinition::Palladium => (0.733, 0.697, 0.652),
            GlossyPredefinition::Platinum => (0.673, 0.637, 0.585),
            GlossyPredefinition::Silver => (0.972, 0.960, 0.915),
            GlossyPredefinition::Titanium => (0.542, 0.497, 0.449),
            GlossyPredefinition::Zinc => (0.664, 0.824, 0.850),
        };
        let albedo = Albedo::new(Val(r0_r), Val(r0_g), Val(r0_b)).unwrap();
        Self::new(albedo, Val(1.0), roughness)
    }
}

impl Material for Glossy {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Glossy
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let light = self.shade_light(context, ray, intersection);
        let state_next = state.with_skip_emissive(true);
        let scattering = self.shade_scattering(context, state_next, ray, intersection);
        light + scattering
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        match state.policy() {
            StoragePolicy::Global => {
                self.maybe_bounce_next_photon(context, state, photon, intersection);
            }
            StoragePolicy::Caustic => {}
        }
    }
}

impl MicrofacetMaterial for Glossy {
    #[inline]
    fn r0(&self, intersection: &RayIntersection) -> Spectrum {
        let albedo = self.albedo.lookup(intersection).into();
        Spectrum::lerp(Self::DIELECTRIC_R0, albedo, self.metalness)
    }

    #[inline]
    fn alpha(&self) -> Val {
        self.alpha
    }
}

impl BsdfMaterial for Glossy {
    fn bsdf(
        &self,
        dir_out: Direction,
        intersection: &RayIntersection,
        dir_in: Direction,
    ) -> Spectrum {
        let normal = intersection.normal();
        if normal.dot(dir_in) > Val(0.0) {
            let mn = Normal::normalize(dir_out + dir_in).unwrap();

            let reflectance = self.calc_reflectance(dir_in.dot(mn), intersection);
            let ndf = self.calc_ndf(normal, mn);
            let g2 = self.calc_g2(dir_out, dir_in, normal);
            let (cos, cos_next) = (dir_out.dot(normal), dir_in.dot(normal));

            (reflectance * ndf * g2) / (Val(4.0) * cos * cos_next).abs()
        } else {
            Spectrum::zero()
        }
    }
}

impl BsdfSampling for Glossy {
    fn sample_bsdf(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let dir = -ray.direction();
        let normal = intersection.normal();

        let mn = self.generate_microfacet_normal(dir, normal, rng);
        let ray_next = ray_util::reflect_microfacet(ray, intersection, mn);
        let dir_next = ray_next.direction();

        let reflectance = self.calc_reflectance(dir.dot(mn), intersection);
        let g2 = self.calc_g2(dir, dir_next, normal);
        let g1 = self.calc_g1(dir, normal);
        let coefficient = reflectance * g2 / g1;

        let ndf = self.calc_ndf(normal, mn);
        let pdf = g1 * ndf * Val(0.25) / dir.dot(normal);

        BsdfSample::new(ray_next, coefficient, pdf)
    }

    fn pdf_bsdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let (dir, dir_next) = (-ray.direction(), ray_next.direction());
        let Ok(mn) = Normal::normalize(dir + dir_next) else {
            return Val(0.0);
        };

        let normal = intersection.normal();
        if dir_next.dot(normal) <= Val(0.0) {
            return Val(0.0);
        }

        let g1 = self.calc_g1(dir, normal);
        let ndf = self.calc_ndf(normal, mn);
        g1 * ndf * Val(0.25) / dir.dot(normal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlossyPredefinition {
    Aluminum,
    Brass,
    Chromium,
    Copper,
    Gold,
    Iron,
    Mercury,
    Nickel,
    Palladium,
    Platinum,
    Silver,
    Titanium,
    Zinc,
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewGlossyError {
    #[snafu(display("metalness should be in [0, 1]"))]
    InvalidMetalness,
    #[snafu(display("roughness should be in (0, 1]"))]
    InvalidRoughness,
}
