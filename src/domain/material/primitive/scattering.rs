use std::any::Any;

use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::entity::Scene;
use crate::domain::material::def::{BsdfMaterial, BsdfMaterialExt, Material, MaterialKind};
use crate::domain::material::primitive::Specular;
use crate::domain::math::algebra::{Product, UnitVector, Vector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{
    BsdfSample, BsdfSampling, BssrdfDiffusionSample, BssrdfDirectionSample, BssrdfSampling,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Scattering {
    albedo: Albedo,
    scattering_distance: Vector,
    refractive_index: Val,
}

impl Scattering {
    const PROJECTION_AXIS_PROB: [Val; 3] = [Val(0.25), Val(0.25), Val(0.5)];
    const COLOR_CHANNEL_PROB: [Val; 3] = [Val(1.0 / 3.0); 3];
    const MAX_RADIUS_CDF: Val = Val(0.999);

    pub fn new(
        albedo: Albedo,
        mean_free_path: Val,
        refractive_index: Val,
    ) -> Result<Self, TryNewScatteringError> {
        ensure!(mean_free_path > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);

        let scattering_distance = Vector::new(
            Self::calc_scattering_distance(albedo.red(), mean_free_path),
            Self::calc_scattering_distance(albedo.green(), mean_free_path),
            Self::calc_scattering_distance(albedo.blue(), mean_free_path),
        );
        Ok(Self {
            albedo,
            scattering_distance,
            refractive_index,
        })
    }

    fn calc_scattering_distance(albedo: Val, mean_free_path: Val) -> Val {
        let scaling_factor = Val(1.9) - albedo + Val(3.5) * (albedo - Val(0.8)).powi(2);
        mean_free_path / scaling_factor
    }

    fn calc_normalized_diffusion(&self, d: Val, radius: Val) -> Spectrum {
        let exp_13 = (-radius / (Val(3.0) * d)).exp();
        let exp_sum = exp_13 * (Val(1.0) + exp_13.powi(2));
        self.albedo * (Val(8.0) * Val::PI * radius * d).recip() * exp_sum
    }

    fn generate_normailzed_diffusion_radius(d: Val, rng: &mut dyn RngCore) -> Option<Val> {
        let cdf = Val(rng.random());
        if cdf <= Self::MAX_RADIUS_CDF {
            Some(Self::calc_normailzed_diffusion_radius(d, cdf))
        } else {
            None
        }
    }

    fn calc_max_normailzed_diffusion_radius(d: Val) -> Val {
        Self::calc_normailzed_diffusion_radius(d, Self::MAX_RADIUS_CDF)
    }

    fn calc_normailzed_diffusion_radius(d: Val, cdf: Val) -> Val {
        let u = Val(1.0) - cdf;
        let g = Val(1.0) + (Val(4.0) * u) * (Val(2.0) * u + (Val(1.0) + (Val(4.0) * u) * u).sqrt());
        let gn13 = g.powf(Val(-1.0 / 3.0));
        let gp13 = (g * gn13) * gn13;
        let c = Val(1.0) + gp13 + gn13;
        let x = Val(3.0) * (c / (Val(4.0) * u)).ln();
        x * d
    }

    fn calc_normailzed_diffusion_pdf(d: Val, radius: Val) -> Val {
        let exp13 = (-radius / (Val(3.0) * d)).exp();
        let exp_sum = exp13 * (Val(1.0) + exp13 * exp13);
        exp_sum / (Val(8.0) * Val::PI * d * radius)
    }

    fn generate_projection_axes(normal: UnitVector, rng: &mut dyn RngCore) -> [UnitVector; 3] {
        let (b1, b2) = normal.orthonormal_basis();
        let r = Val(rng.random());
        if r < Self::PROJECTION_AXIS_PROB[0] {
            [normal, b1, b2]
        } else if r < Self::PROJECTION_AXIS_PROB[0] + Self::PROJECTION_AXIS_PROB[1] {
            [b2, normal, b1]
        } else {
            [b1, b2, normal]
        }
    }

    fn project_normalized_diffusion_point(
        position_out: Point,
        scene: &dyn Scene,
        axes: &[UnitVector; 3],
        (radius, phi): (Val, Val),
        radius_max: Val,
    ) -> Option<RayIntersection> {
        let (x_local, y_local) = (radius * phi.cos(), radius * phi.sin());
        let point_disk = position_out + x_local * axes[0] + y_local * axes[1];

        let proj_ray_max_len = Val(2.0) * (radius_max.powi(2) - radius.powi(2)).sqrt();
        let proj_ray_start = point_disk + (Val(0.5) * proj_ray_max_len) * axes[2];
        let proj_ray = Ray::new(proj_ray_start, -axes[2]);

        let mut range = DisRange::inclusive(Val(0.0), proj_ray_max_len);
        while let Some((intersection, id)) = scene.find_intersection(&proj_ray, range) {
            range = range.advance_start(intersection.distance());
            let is_scattering = id.material_id().kind() == MaterialKind::Scattering;
            let is_front = intersection.side() == SurfaceSide::Front;
            if is_scattering && is_front {
                return Some(intersection);
            }
        }

        None
    }

    fn calc_transmittance(&self, cos_i: Val) -> Val {
        let ri = self.refractive_index;
        let r0_sqrt = (Val(1.0) - ri) / (Val(1.0) + ri);
        let r0 = r0_sqrt * r0_sqrt;
        let reflectance = r0 + (Val(1.0) - r0) * (Val(1.0) - cos_i).powi(5);
        Val(1.0) - reflectance
    }
}

impl Material for Scattering {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Scattering
    }

    fn shade(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: Ray,
        intersection: RayIntersection,
    ) -> Contribution {
        let cos = intersection.normal().dot(-ray.direction());
        if Val(context.rng().random()) < self.calc_transmittance(cos) {
            let scene = context.scene();
            let sample = self.sample_bssrdf_diffusion(scene, &intersection, *context.rng());
            if let Some(diffusion) = sample {
                let adapter = ScatteringBsdfMaterialAdapter::new(self, &diffusion);
                adapter.shade(context, state, ray, intersection)
            } else {
                Contribution::new()
            }
        } else {
            let specular = Specular::new(self.albedo);
            specular.shade(context, state, ray, intersection)
        }
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let cos = intersection.normal().dot(-photon.direction());
        if Val(context.rng().random()) < self.calc_transmittance(cos) {
            let scene = context.scene();
            let res = self.sample_bssrdf_diffusion(scene, &intersection, *context.rng());
            if let Some(diffusion) = res {
                let adapter = ScatteringBsdfMaterialAdapter::new(self, &diffusion);
                adapter.receive(context, state, photon, intersection)
            }
        } else {
            let specular = Specular::new(self.albedo);
            specular.receive(context, state, photon, intersection);
        }
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl BssrdfSampling for Scattering {
    fn sample_bssrdf_diffusion(
        &self,
        scene: &dyn Scene,
        intersection_out: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<BssrdfDiffusionSample> {
        let d = self.scattering_distance.axis(rng.random_range(0..2));
        let radius = Self::generate_normailzed_diffusion_radius(d, rng)?;
        let phi = Val(2.0) * Val::PI * Val(rng.random());

        let axes = Self::generate_projection_axes(intersection_out.normal(), rng);
        let intersection_in = Self::project_normalized_diffusion_point(
            intersection_out.position(),
            scene,
            &axes,
            (radius, phi),
            Self::calc_max_normailzed_diffusion_radius(d),
        )?;

        let distance = (intersection_in.position() - intersection_out.position()).norm();
        let bssrdf_diffusion = self.calc_normalized_diffusion(d, distance);
        let pdf = self.pdf_bssrdf_diffusion(intersection_out, &intersection_in);
        Some(BssrdfDiffusionSample::new(
            distance,
            intersection_in,
            bssrdf_diffusion,
            pdf,
        ))
    }

    fn pdf_bssrdf_diffusion(
        &self,
        intersection_out: &RayIntersection,
        intersection_in: &RayIntersection,
    ) -> Val {
        let radius_vec = intersection_in.position() - intersection_out.position();
        let normal = intersection_out.normal();
        let (tangent1, tangent2) = normal.orthonormal_basis();
        let mut axes = [normal, tangent1, tangent2];

        let mut pdf = Val(0.0);
        for axis in 0..3 {
            let (x_local, y_local) = (radius_vec.dot(axes[0]), radius_vec.dot(axes[1]));
            let radius = (x_local.powi(2) + y_local.powi(2)).sqrt();
            let cos = intersection_in.normal().dot(axes[2]).max(Val(0.0));

            for channel in 0..3 {
                let d = self.scattering_distance.axis(channel);
                pdf += Self::calc_normailzed_diffusion_pdf(d, radius)
                    * cos
                    * Self::PROJECTION_AXIS_PROB[axis]
                    * Self::COLOR_CHANNEL_PROB[channel]
            }
            axes.rotate_right(1);
        }

        pdf
    }

    fn sample_bssrdf_direction(
        &self,
        intersection_in: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BssrdfDirectionSample {
        let normal = intersection_in.normal();
        let direction = UnitVector::random_cosine_hemisphere(normal, rng);
        let ray_next = Ray::new(intersection_in.position(), direction);

        let cos = ray_next.direction().dot(intersection_in.normal());
        let transmittance = self.calc_transmittance(cos);
        let bssrdf_direction = Spectrum::broadcast(Val::FRAC_1_PI * transmittance);
        let pdf = Val::FRAC_1_PI * cos;
        BssrdfDirectionSample::new(ray_next, bssrdf_direction, pdf)
    }

    fn pdf_bssrdf_direction(&self, intersection_in: &RayIntersection, ray_next: &Ray) -> Val {
        let cos = ray_next.direction().dot(intersection_in.normal());
        Val::FRAC_1_PI * cos.max(Val(0.0))
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewScatteringError {
    #[snafu(display("mean free path is not positive"))]
    InvalidMeanFreePath,
    #[snafu(display("refractive index is not positive"))]
    InvalidRefractiveIndex,
}

#[derive(Debug)]
struct ScatteringBsdfMaterialAdapter<'a> {
    inner: &'a Scattering,
    diffusion: &'a BssrdfDiffusionSample,
}

impl<'a> ScatteringBsdfMaterialAdapter<'a> {
    fn new(inner: &'a Scattering, diffusion: &'a BssrdfDiffusionSample) -> Self {
        Self { inner, diffusion }
    }
}

impl<'a> Material for ScatteringBsdfMaterialAdapter<'a> {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Refractive
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
        res * (self.diffusion.bssrdf_diffusion() / self.diffusion.pdf())
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: PhotonRay,
        intersection: RayIntersection,
    ) {
        let photon_next = PhotonRay::new(
            Ray::new(photon.start(), photon.direction()),
            photon.throughput() * (self.diffusion.bssrdf_diffusion() / self.diffusion.pdf()),
        );
        self.maybe_bounce_next_photon(context, state, photon_next, intersection);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

impl<'a> BsdfMaterial for ScatteringBsdfMaterialAdapter<'a> {
    fn bsdf(
        &self,
        _dir_out: UnitVector,
        intersection: &RayIntersection,
        dir_in: UnitVector,
    ) -> Spectrum {
        let cos = dir_in.dot(intersection.normal());
        if cos > Val(0.0) {
            let transmittance = self.inner.calc_transmittance(cos);
            Spectrum::broadcast(Val::FRAC_1_PI * transmittance)
        } else {
            Spectrum::zero()
        }
    }
}

impl<'a> BsdfSampling for ScatteringBsdfMaterialAdapter<'a> {
    fn sample_bsdf(
        &self,
        _ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let sample = self.inner.sample_bssrdf_direction(intersection, rng);
        let cos = intersection.normal().dot(sample.ray_next().direction());
        let pdf = sample.pdf();
        let coefficient = sample.bssrdf_direction() * cos / pdf;
        BsdfSample::new(sample.into_ray_next(), coefficient, pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.inner.pdf_bssrdf_direction(intersection, ray_next)
    }
}
