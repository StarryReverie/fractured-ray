use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::{Albedo, Spectrum};
use crate::domain::material::def::{
    BsdfMaterial, BsdfMaterialExt, BssrdfMaterial, BssrdfMaterialExt, Material, MaterialKind,
};
use crate::domain::material::primitive::Specular;
use crate::domain::math::algebra::Product;
use crate::domain::math::geometry::{Direction, Distance, Point, PositionedFrame};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, SurfaceSide};
use crate::domain::ray::photon::PhotonRay;
use crate::domain::ray::util as ray_util;
use crate::domain::renderer::{
    Contribution, PmContext, PmState, RtContext, RtState, StoragePolicy,
};
use crate::domain::sampling::coefficient::{
    BsdfSample, BsdfSampling, BssrdfDiffusionSample, BssrdfDirectionSample, BssrdfSampling,
};
use crate::domain::scene::entity::EntityScene;
use crate::domain::texture::def::DynAlbedoTexture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scattering {
    albedo: DynAlbedoTexture,
    refractive_index: Val,
    mean_free_path: Val,
}

impl Scattering {
    const PROJECTION_AXIS_PROB: [Val; 3] = [Val(0.5), Val(0.25), Val(0.25)];
    const COLOR_CHANNEL_PROB: [Val; 3] = [Val(1.0 / 3.0); 3];
    const MAX_RADIUS_CDF: Val = Val(0.999);
    const IGNORE_BACK_THRESHOLD: Val = Val(0.01);

    pub fn new<T>(
        albedo: T,
        mean_free_path: Val,
        refractive_index: Val,
    ) -> Result<Self, TryNewScatteringError>
    where
        T: Into<DynAlbedoTexture>,
    {
        ensure!(mean_free_path > Val(0.0), InvalidMeanFreePathSnafu);
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);

        Ok(Self {
            albedo: albedo.into(),
            refractive_index,
            mean_free_path,
        })
    }

    fn calc_scattering_distance(albedo: Albedo, mean_free_path: Val) -> Spectrum {
        Spectrum::new(
            Self::calc_scattering_distance_single(albedo.red(), mean_free_path),
            Self::calc_scattering_distance_single(albedo.green(), mean_free_path),
            Self::calc_scattering_distance_single(albedo.blue(), mean_free_path),
        )
    }

    fn calc_scattering_distance_single(albedo: Val, mean_free_path: Val) -> Val {
        let scaling_factor = Val(1.9) - albedo + Val(3.5) * (albedo - Val(0.8)).powi(2);
        mean_free_path / scaling_factor
    }

    fn calc_normalized_diffusion(&self, albedo: Albedo, d: Val, radius: Val) -> Spectrum {
        let exp_13 = (-radius / (Val(3.0) * d)).exp();
        let exp_sum = exp_13 * (Val(1.0) + exp_13.powi(2));
        albedo * (Val(8.0) * Val::PI * radius * d).recip() * exp_sum
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
        scene: &dyn EntityScene,
        frame: &PositionedFrame,
        (radius, phi): (Val, Val),
        radius_max: Val,
    ) -> Option<RayIntersection> {
        let (x_local, y_local) = (radius * phi.cos(), radius * phi.sin());
        let point_disk = frame.to_canonical(Point::new(x_local, y_local, Val(0.0)));

        let proj_ray_max_len =
            Distance::new(Val(2.0) * (radius_max.powi(2) - radius.powi(2)).sqrt()).unwrap();
        let proj_ray_start = point_disk + (Val(0.5) * proj_ray_max_len.value()) * frame.normal();
        let proj_ray = Ray::new(
            proj_ray_start,
            -Direction::from(frame.normal().to_unit_vector()),
        );

        let mut range = DisRange::inclusive(Distance::zero(), proj_ray_max_len);
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

    fn shade_front_face(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
        albedo: Albedo,
    ) -> Contribution {
        let cos = intersection.normal().dot(-ray.direction());
        if Val(context.rng().random()) < self.calc_transmittance(cos) {
            let state_next = state.with_visible(false);

            let scene = context.entity_scene();
            let back = self.determine_back_face(scene, ray, intersection, *context.rng());

            if let Some((ray_back, intersection_back)) = back {
                self.shade_back_face(context, state_next, &ray_back, &intersection_back, albedo)
            } else {
                self.shade_impl(context, state_next, ray, intersection)
            }
        } else {
            let specular = Specular::new(albedo);
            specular.shade(context, state, ray, intersection)
        }
    }

    fn shade_back_face(
        &self,
        context: &mut RtContext<'_>,
        state: RtState,
        ray: &Ray,
        intersection: &RayIntersection,
        albedo: Albedo,
    ) -> Contribution {
        let adapter = BackFaceTransmissionAdapter::new(self, albedo);
        let state_next = state.with_visible(false);
        adapter.shade(context, state_next, ray, intersection)
    }

    fn receive_front_face(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
        albedo: Albedo,
    ) {
        let cos = intersection.normal().dot(-photon.direction());
        if Val(context.rng().random()) < self.calc_transmittance(cos) {
            if state.policy() == StoragePolicy::Caustic {
                return;
            }

            let scene = context.scene();
            let back = self.determine_back_face(scene, photon.ray(), intersection, *context.rng());

            if let Some((ray_back, intersection_back)) = back {
                let photon_back = PhotonRay::new(ray_back, photon.throughput());
                self.receive_back_face(context, state, &photon_back, &intersection_back, albedo);
            } else {
                self.receive_impl(context, state, photon, intersection);
            }
        } else {
            let specular = Specular::new(albedo);
            specular.receive(context, state, photon, intersection);
        }
    }

    fn receive_back_face(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
        albedo: Albedo,
    ) {
        let adapter = BackFaceTransmissionAdapter::new(self, albedo);
        adapter.receive(context, state, photon, intersection);
    }

    fn determine_back_face(
        &self,
        scene: &dyn EntityScene,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<(Ray, RayIntersection)> {
        let ray_back = ray_util::pure_refract(ray, intersection, self.refractive_index)
            .expect("refractive ray always exists when incident ray hits front face");

        let res = scene.find_intersection(&ray_back, DisRange::positive());
        let intersection_back = if let Some((intersection_back, id)) = res {
            if id.material_id().kind() != MaterialKind::Scattering {
                return None;
            }
            if intersection_back.side() != SurfaceSide::Back {
                return None;
            }
            intersection_back
        } else {
            return None;
        };

        let distance = intersection_back.distance();
        let volume_transmittance = (-distance.value() / self.mean_free_path).exp();
        if volume_transmittance < Self::IGNORE_BACK_THRESHOLD {
            return None;
        }
        if Val(rng.random()) > volume_transmittance {
            return None;
        }

        Some((ray_back, intersection_back))
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
        ray: &Ray,
        intersection: &RayIntersection,
    ) -> Contribution {
        let albedo = self.albedo.lookup(intersection);
        if intersection.side() == SurfaceSide::Front {
            self.shade_front_face(context, state, ray, intersection, albedo)
        } else {
            self.shade_back_face(context, state, ray, intersection, albedo)
        }
    }

    fn receive(
        &self,
        context: &mut PmContext<'_>,
        state: PmState,
        photon: &PhotonRay,
        intersection: &RayIntersection,
    ) {
        let albedo = self.albedo.lookup(intersection);
        if intersection.side() == SurfaceSide::Front {
            self.receive_front_face(context, state, photon, intersection, albedo)
        } else {
            self.receive_back_face(context, state, photon, intersection, albedo);
        }
    }
}

impl BssrdfMaterial for Scattering {
    fn bssrdf_direction(&self, intersection_in: &RayIntersection, dir_in: Direction) -> Spectrum {
        let normal = if intersection_in.side() == SurfaceSide::Front {
            intersection_in.normal()
        } else {
            -intersection_in.normal()
        };
        let cos = dir_in.dot(normal);
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
        scene: &dyn EntityScene,
        intersection_out: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<BssrdfDiffusionSample> {
        let albedo = self.albedo.lookup(intersection_out);
        let scattering_distance = Self::calc_scattering_distance(albedo, self.mean_free_path);

        let d = scattering_distance.channel(rng.random_range(0..2));
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
        let bssrdf_diffusion = self.calc_normalized_diffusion(albedo, d, distance);
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
        let albedo = self.albedo.lookup(intersection_out);
        let scattering_distance = Self::calc_scattering_distance(albedo, self.mean_free_path);

        let normal = intersection_out.normal();
        let mut frame = PositionedFrame::new(intersection_out.position(), normal);

        let mut pdf = Val(0.0);
        for axis in 0..3 {
            let in_local = frame.to_local(intersection_in.position());
            let radius = (in_local.x().powi(2) + in_local.y().powi(2)).sqrt();
            let cos = intersection_in.normal().dot(frame.normal()).max(Val(0.0));

            for channel in 0..3 {
                let d = scattering_distance.channel(channel);
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
        let normal = if intersection_in.side() == SurfaceSide::Front {
            intersection_in.normal()
        } else {
            -intersection_in.normal()
        };
        let direction = Direction::random_cosine_hemisphere(normal, rng);
        let ray_next = intersection_in.spawn(direction);

        let cos = ray_next.direction().dot(normal);
        let transmittance = self.calc_transmittance(cos);
        let bssrdf_direction = Spectrum::broadcast(Val::FRAC_1_PI * transmittance);
        let pdf = Val::FRAC_1_PI * cos;
        BssrdfDirectionSample::new(ray_next, bssrdf_direction, pdf)
    }

    fn pdf_bssrdf_direction(&self, intersection_in: &RayIntersection, ray_next: &Ray) -> Val {
        let normal = if intersection_in.side() == SurfaceSide::Front {
            intersection_in.normal()
        } else {
            -intersection_in.normal()
        };
        let cos = ray_next.direction().dot(normal);
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

#[derive(Debug, Clone, PartialEq)]
struct BackFaceTransmissionAdapter<'a> {
    inner: &'a Scattering,
    albedo: Albedo,
}

impl<'a> BackFaceTransmissionAdapter<'a> {
    fn new(inner: &'a Scattering, albedo: Albedo) -> Self {
        Self { inner, albedo }
    }
}

impl<'a> Material for BackFaceTransmissionAdapter<'a> {
    fn kind(&self) -> MaterialKind {
        MaterialKind::Refractive
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

impl<'a> BsdfMaterial for BackFaceTransmissionAdapter<'a> {
    fn bsdf(
        &self,
        _dir_out: Direction,
        intersection: &RayIntersection,
        dir_in: Direction,
    ) -> Spectrum {
        self.albedo * self.inner.bssrdf_direction(intersection, dir_in)
    }
}

impl<'a> BsdfSampling for BackFaceTransmissionAdapter<'a> {
    fn sample_bsdf(
        &self,
        _ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> BsdfSample {
        let sample = self.inner.sample_bssrdf_direction(intersection, rng);
        let cos = (intersection.normal())
            .dot(sample.ray_next().direction())
            .abs();
        let pdf = sample.pdf();
        let coefficient = self.albedo * sample.bssrdf_direction() * cos / pdf;
        BsdfSample::new(sample.into_ray_next(), coefficient, pdf)
    }

    fn pdf_bsdf(&self, _ray: &Ray, intersection: &RayIntersection, ray_next: &Ray) -> Val {
        self.inner.pdf_bssrdf_direction(intersection, ray_next)
    }
}
