use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::{Rotation, Transform, Transformation};
use crate::domain::math::numeric::Val;
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::{
    Contribution, PmContext, PmState, RtContext, RtState, StoragePolicy,
};
use crate::domain::sampling::coefficient::{BsdfSample, BsdfSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct Glossy {
    r0: Vector,
    alpha: Val,
}

impl Glossy {
    const DIELECTRIC_R0: Vector = Vector::broadcast(Val(0.04));

    pub fn new(color: Color, metalness: Val, roughness: Val) -> Result<Self, TryNewGlossyError> {
        ensure!(
            Val(0.0) <= metalness && metalness <= Val(1.0),
            InvalidMetalnessSnafu
        );
        ensure!(
            Val(0.0) < roughness && roughness <= Val(1.0),
            InvalidRoughnessSnafu
        );

        let r0 = Vector::lerp(Self::DIELECTRIC_R0, color.to_vector(), metalness);
        let alpha = roughness.powi(2);
        Ok(Self { r0, alpha })
    }

    pub fn lookup(
        predefinition: GlossyPredefinition,
        roughness: Val,
    ) -> Result<Self, TryNewGlossyError> {
        ensure!(
            Val(0.0) < roughness && roughness <= Val(1.0),
            InvalidRoughnessSnafu
        );

        let alpha = roughness.powi(2);
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
        let r0 = Vector::new(Val(r0_r), Val(r0_g), Val(r0_b));
        Ok(Self { r0, alpha })
    }

    fn generate_microfacet_normal(
        &self,
        dir: UnitVector,
        normal: UnitVector,
        rng: &mut dyn RngCore,
    ) -> UnitVector {
        let tr = Rotation::new(normal, UnitVector::z_direction(), Val(0.0));
        let local_dir = dir.transform(&tr);
        let local_mn = self.generate_local_microfacet_normal(local_dir, rng);
        local_mn.transform(&tr.inverse())
    }

    fn generate_local_microfacet_normal(
        &self,
        local_dir: UnitVector,
        rng: &mut dyn RngCore,
    ) -> UnitVector {
        let alpha = self.alpha;

        let ldir_tr = Vector::new(alpha * local_dir.x(), alpha * local_dir.y(), local_dir.z())
            .normalize()
            .unwrap();
        let (b1, b2) = ldir_tr.orthonormal_basis();

        let r = Val(rng.random()).sqrt();
        let phi = Val(2.0) * Val::PI * Val(rng.random());
        let (t1, t2) = (r * phi.cos(), r * phi.sin());
        let s = Val(0.5) * (Val(1.0) + ldir_tr.z());
        let t2 = (Val(1.0) - s) * (Val(1.0) - t1.powi(2)).sqrt() + s * t2;

        let t3 = (Val(1.0) - t1.powi(2) - t2.powi(2)).max(Val(0.0)).sqrt();
        let mn_tr = t1 * b1 + t2 * b2 + t3 * ldir_tr;
        let mn = Vector::new(
            alpha * mn_tr.x(),
            alpha * mn_tr.y(),
            mn_tr.z().max(Val(0.0)),
        );
        mn.normalize().unwrap()
    }

    fn calc_next_ray(&self, ray: &Ray, intersection: &RayIntersection, mn: UnitVector) -> Ray {
        let dir = ray.direction();
        let dir_next = dir - Val(2.0) * dir.dot(mn) * mn;
        Ray::new(intersection.position(), dir_next.normalize().unwrap())
    }

    fn calc_reflectance(&self, cos: Val) -> Vector {
        self.r0 + (Vector::broadcast(Val(1.0)) - self.r0) * (Val(1.0) - cos).powi(5)
    }

    fn calc_ndf(&self, normal: UnitVector, mn: UnitVector) -> Val {
        let alpha_squared = self.alpha.powi(2);
        let cos = normal.dot(mn);
        alpha_squared / (Val::PI * (cos.powi(2) * (alpha_squared - Val(1.0)) + Val(1.0)).powi(2))
    }

    fn calc_g1(&self, dir: UnitVector, normal: UnitVector) -> Val {
        let tan = (dir.dot(normal)).acos().tan();
        let tmp = (Val(1.0) + (self.alpha * tan).powi(2)).sqrt();
        Val(2.0) / (Val(1.0) + tmp)
    }

    fn calc_g2(&self, dir: UnitVector, dir_next: UnitVector, normal: UnitVector) -> Val {
        let tan = dir.dot(normal).acos().tan();
        let tan_next = dir_next.dot(normal).acos().tan();
        let tmp = (Val(1.0) + (self.alpha * tan).powi(2)).sqrt();
        let tmp_next = (Val(1.0) + (self.alpha * tan_next).powi(2)).sqrt();
        Val(2.0) / (tmp + tmp_next)
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

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl BsdfMaterial for Glossy {
    fn bsdf(
        &self,
        dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Vector {
        let normal = intersection.normal();
        if normal.dot(dir_in) > Val(0.0) {
            let mn = (dir_out + dir_in).normalize().unwrap();

            let reflectance = self.calc_reflectance(dir_in.dot(mn));
            let ndf = self.calc_ndf(normal, mn);
            let g2 = self.calc_g2(dir_out, dir_in, normal);
            let (cos, cos_next) = (dir_out.dot(normal), dir_in.dot(normal));

            (reflectance * ndf * g2) / (Val(4.0) * cos * cos_next).abs()
        } else {
            Vector::broadcast(Val(0.0))
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
        let ray_next = self.calc_next_ray(ray, intersection, mn);
        let dir_next = ray_next.direction();

        let reflectance = self.calc_reflectance(dir_next.dot(mn));
        let g2 = self.calc_g2(dir, dir_next, normal);
        let g1 = self.calc_g1(dir, normal);
        let coefficient = reflectance * g2 / g1;

        let ndf = self.calc_ndf(normal, mn);
        let pdf = g1 * ndf * Val(0.25) / dir.dot(normal);

        BsdfSample::new(ray_next, coefficient, pdf)
    }

    fn pdf_bsdf(&self, ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        let (dir, dir_next) = (-ray.direction(), ray_next.direction());
        let Ok(mn) = (dir + dir_next).normalize() else {
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
