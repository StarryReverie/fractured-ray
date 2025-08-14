use std::any::Any;

use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::entity::Scene;
use crate::domain::material::def::{BssrdfMaterial, BssrdfMaterialExt, Material, MaterialKind};
use crate::domain::material::primitive::Specular;
use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::{Point, PositionedFrame};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::{Contribution, PmContext, PmState, RtContext, RtState};
use crate::domain::sampling::coefficient::{
    BssrdfDiffusionSample, BssrdfDirectionSample, BssrdfSampling,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Scattering {
    albedo: Albedo,
    scattering_distance: Spectrum,
    refractive_index: Val,
}

impl Scattering {
    const PROJECTION_AXIS_PROB: [Val; 3] = [Val(0.5), Val(0.25), Val(0.25)];
    const COLOR_CHANNEL_PROB: [Val; 3] = [Val(1.0 / 3.0); 3];
    const MAX_RADIUS_CDF: Val = Val(0.999);

    pub fn new(
        albedo: Albedo,
        mean_free_path: Val,
        refractive_index: Val,
    ) -> Result<Self, TryNewScatteringError> {
        ensure!(mean_free_path > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);

        let scattering_distance = Spectrum::new(
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

    fn generate_projection_frame(
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> PositionedFrame {
        let frame = PositionedFrame::new(intersection.position(), intersection.normal());
        let r = Val(rng.random());
        if r < Self::PROJECTION_AXIS_PROB[0] {
            frame
        } else if r < Self::PROJECTION_AXIS_PROB[0] + Self::PROJECTION_AXIS_PROB[1] {
            frame.permute_axes()
        } else {
            frame.permute_axes().permute_axes()
        }
    }

    fn project_normalized_diffusion_point(
        scene: &dyn Scene,
        frame: &PositionedFrame,
        (radius, phi): (Val, Val),
        radius_max: Val,
    ) -> Option<RayIntersection> {
        let (x_local, y_local) = (radius * phi.cos(), radius * phi.sin());
        let point_disk = frame.to_canonical(Point::new(x_local, y_local, Val(0.0)));

        let proj_ray_max_len = Val(2.0) * (radius_max.powi(2) - radius.powi(2)).sqrt();
        let proj_ray_start = point_disk + (Val(0.5) * proj_ray_max_len) * frame.normal();
        let proj_ray = Ray::new(proj_ray_start, -frame.normal());

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
            self.shade_impl(context, state, ray, intersection)
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
            self.receive_impl(context, state, photon, intersection);
        } else {
            let specular = Specular::new(self.albedo);
            specular.receive(context, state, photon, intersection);
        }
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl BssrdfMaterial for Scattering {
    fn bssrdf_direction(&self, intersection_in: &RayIntersection, dir_in: UnitVector) -> Spectrum {
        let cos = dir_in.dot(intersection_in.normal());
        if cos > Val(0.0) {
            let transmittance = self.calc_transmittance(cos);
            Spectrum::broadcast(Val::FRAC_1_PI * transmittance)
        } else {
            Spectrum::zero()
        }
    }
}

impl BssrdfSampling for Scattering {
    fn sample_bssrdf_diffusion(
        &self,
        scene: &dyn Scene,
        intersection_out: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<BssrdfDiffusionSample> {
        let d = self.scattering_distance.channel(rng.random_range(0..2));
        let radius = Self::generate_normailzed_diffusion_radius(d, rng)?;
        let phi = Val(2.0) * Val::PI * Val(rng.random());

        let frame = Self::generate_projection_frame(intersection_out, rng);
        let intersection_in = Self::project_normalized_diffusion_point(
            scene,
            &frame,
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
        let normal = intersection_out.normal();
        let mut frame = PositionedFrame::new(intersection_out.position(), normal);

        let mut pdf = Val(0.0);
        for axis in 0..3 {
            let in_local = frame.to_local(intersection_in.position());
            let radius = (in_local.x().powi(2) + in_local.y().powi(2)).sqrt();
            let cos = intersection_in.normal().dot(frame.normal()).max(Val(0.0));

            for channel in 0..3 {
                let d = self.scattering_distance.channel(channel);
                pdf += Self::calc_normailzed_diffusion_pdf(d, radius)
                    * cos
                    * Self::PROJECTION_AXIS_PROB[axis]
                    * Self::COLOR_CHANNEL_PROB[channel]
            }

            frame = frame.permute_axes();
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
