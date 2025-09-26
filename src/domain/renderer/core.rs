use std::time::Duration;

use getset::{CopyGetters, WithSetters};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use rand::prelude::*;
use rayon::prelude::*;
use snafu::prelude::*;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::core::Spectrum;
use crate::domain::image::{Image, ImageAccumulator};
use crate::domain::material::def::{FluxEstimation, Material, RefDynMaterial};
use crate::domain::math::geometry::Direction;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::medium::def::Medium;
use crate::domain::medium::util::AggregateMedium;
use crate::domain::ray::Ray;
use crate::domain::ray::event::{RayIntersection, RaySegment};
use crate::domain::ray::photon::{PhotonMap, PhotonRay, SearchPolicy};
use crate::domain::scene::entity::EntityScene;
use crate::domain::scene::volume::VolumeScene;

use super::{
    Contribution, PhotonInfo, PmContext, PmState, Renderer, RtContext, RtState, StoragePolicy,
};

pub struct CoreRenderer {
    camera: Camera,
    entity_scene: Box<dyn EntityScene>,
    volume_scene: Box<dyn VolumeScene>,
    config: CoreRendererConfiguration,
}

impl CoreRenderer {
    pub fn new(
        camera: Camera,
        entity_scene: Box<dyn EntityScene>,
        volume_scene: Box<dyn VolumeScene>,
        config: CoreRendererConfiguration,
    ) -> Result<Self, CoreRendererConfigurationError> {
        config.validate()?;
        Ok(Self {
            camera,
            entity_scene,
            volume_scene,
            config,
        })
    }

    fn render_pixel(
        &self,
        pos: (usize, usize),
        pixel: &mut Pixel,
        photon_global: PhotonInfo<'_>,
        photon_caustic: PhotonInfo<'_>,
    ) -> Spectrum {
        let mut rng = rand::rng();
        let mut context = RtContext::new(
            self,
            self.entity_scene.as_ref(),
            self.volume_scene.as_ref(),
            &mut rng,
            &self.config,
            photon_global,
            photon_caustic,
        );

        let contributions = (0..self.config.spp_per_iteration)
            .map(|_| self.start_tracing(&mut context, pos))
            .map(|c| c.clamp())
            .collect();
        pixel.radiance(
            Contribution::average(contributions),
            context.photon_global().emitted(),
            context.photon_casutic().emitted(),
        )
    }

    fn start_tracing<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        (row, column): (usize, usize),
    ) -> Contribution {
        let ray = self.generate_ray(row, column);
        self.trace(context, RtState::new(), &ray, DisRange::positive())
    }

    fn generate_ray(&self, row: usize, column: usize) -> Ray {
        let mut rng = rand::rng();
        let offset = Offset::new(Val(rng.random()), Val(rng.random()))
            .expect("offset range should be bounded to [0, 1)");
        let point = (self.camera)
            .calc_point_in_pixel(row, column, offset)
            .expect("row and column should not be out of bound");
        let direction = Direction::normalize(point - self.camera.position())
            .expect("focal length should be positive");
        Ray::new(point, direction)
    }

    fn init_progress_bar(&self, num_pixel: usize) -> ProgressBar {
        const TEMPLATE: &str = "{msg:>12.green.bold} [{spinner:.yellow.bold}] [{bar:50.cyan.bold/blue.bold}] ({percent}%) [Elapsed: {elapsed_precise} ETA: {eta_precise}]";
        let style = ProgressStyle::with_template(TEMPLATE)
            .unwrap()
            .tick_chars(r#"|/-\|/-\+"#)
            .progress_chars("=>-");
        let cnt = self.config.iterations * num_pixel;
        let bar = ProgressBar::new(cnt as u64)
            .with_style(style)
            .with_message("Rendering")
            .with_finish(ProgressFinish::WithMessage("Finished".into()));
        bar.enable_steady_tick(Duration::from_millis(50));
        bar
    }

    fn build_photon_map(&self, policy: StoragePolicy, total: usize) -> PhotonMap {
        let photons = (0..total)
            .into_par_iter()
            .map(|_| {
                let mut photons = Vec::new();
                let mut rng = rand::rng();
                if let Some(photon) = self.entity_scene.get_emitters().sample_photon(&mut rng) {
                    let mut context =
                        PmContext::new(self, self.entity_scene.as_ref(), &mut rng, &mut photons);
                    let state = PmState::new(false, policy);
                    self.emit(&mut context, state, photon.photon(), DisRange::positive());
                }
                photons
            })
            .flatten()
            .collect();
        PhotonMap::build(photons)
    }
}

impl Renderer for CoreRenderer {
    fn render(&self) -> Image {
        let image = Image::new(self.camera.resolution().clone());
        let mut image = ImageAccumulator::new(image);

        let height = image.resolution().height();
        let width = image.resolution().width();

        let mut pixels = vec![vec![Pixel::new(); width]; height];
        let mut num_global = 0;
        let mut num_caustic = 0;

        let pb = self.init_progress_bar(height * width);
        for _ in 0..self.config.iterations {
            let pmg = self.build_photon_map(StoragePolicy::Global, self.config.photons_global);
            let pmc = self.build_photon_map(StoragePolicy::Caustic, self.config.photons_caustic);
            num_global += self.config.photons_global;
            num_caustic += self.config.photons_caustic;

            let meshgrid = (pixels.par_iter_mut().enumerate())
                .map(|(r, p)| (r, p.par_iter_mut().enumerate()))
                .flat_map(|(r, pi)| pi.map(move |(c, p)| ((r, c), p)));
            let res = meshgrid
                .map(|(pos, pixel)| {
                    pb.inc(1);
                    let num = self.config.initial_num_nearest;
                    let pg = PhotonInfo::new(&pmg, pixel.get_policy_global(num), num_global);
                    let pc = PhotonInfo::new(&pmc, pixel.get_policy_caustic(num), num_caustic);
                    (pos, self.render_pixel(pos, pixel, pg, pc))
                })
                .collect_vec_list();

            for ((row, column), color) in res.into_iter().flatten() {
                image.record(row, column, color);
            }
        }

        image.into_inner()
    }

    fn trace<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        state: RtState,
        ray: &Ray,
        range: DisRange,
    ) -> Contribution {
        let state = state.increment_depth();
        if state.depth() > self.config.max_depth {
            return Contribution::new();
        }

        let res = self.entity_scene.find_intersection(ray, range);
        if let Some((intersection, id)) = res {
            let entities = self.entity_scene.get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            let target = Some((&intersection, material));
            self.trace_to(context, state, ray, target)
        } else {
            self.trace_to(context, state, ray, None)
        }
    }

    fn trace_to<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        state: RtState,
        ray: &Ray,
        target: Option<(&RayIntersection, RefDynMaterial)>,
    ) -> Contribution {
        let (surface_res, vis_range) = if let Some((intersection, material)) = target {
            let res = material.shade(context, state.clone(), ray, intersection);
            let vis_range = DisRange::positive().shrink_end(intersection.distance());
            (res, vis_range)
        } else {
            let res = Contribution::from_light(self.config.background_color);
            (res, DisRange::positive())
        };

        if !state.visible() {
            return surface_res;
        }
        let segments = self.volume_scene.find_segments(ray, vis_range);
        let aggregator = AggregateMedium::new(self.volume_scene.as_ref(), &segments);

        let segment = RaySegment::from(vis_range);
        let volume_res = aggregator.shade(context, state, ray, &segment);
        let transmittance = aggregator.transmittance(ray, &segment);
        transmittance * surface_res + volume_res
    }

    fn emit<'a>(
        &'a self,
        context: &mut PmContext<'a>,
        state: PmState,
        photon: &PhotonRay,
        range: DisRange,
    ) {
        let res = context.scene().find_intersection(photon.ray(), range);
        if let Some((intersection, id)) = res {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            material.receive(context, state, photon, &intersection);
        }
    }
}

#[derive(Debug, Clone, PartialEq, CopyGetters, WithSetters)]
#[getset(get_copy = "pub", set_with = "pub")]
pub struct CoreRendererConfiguration {
    iterations: usize,
    spp_per_iteration: usize,
    max_depth: usize,
    max_invisible_depth: usize,
    photons_global: usize,
    photons_caustic: usize,
    initial_num_nearest: usize,
    background_color: Spectrum,
}

impl CoreRendererConfiguration {
    pub fn validate(&self) -> Result<(), CoreRendererConfigurationError> {
        ensure!(self.iterations > 0, InvalidIterationsSnafu);
        ensure!(self.spp_per_iteration > 0, InvalidSppPerIterationSnafu);
        ensure!(self.max_depth > 0, InvalidMaxDepthSnafu);
        ensure!(
            self.max_invisible_depth > 0,
            NonPositiveMaxInvisibleDepthSnafu,
        );
        ensure!(
            self.max_invisible_depth <= self.max_depth,
            ExceededMaxInvisibleDepthSnafu,
        );
        Ok(())
    }
}

impl Default for CoreRendererConfiguration {
    fn default() -> Self {
        Self {
            iterations: 4,
            spp_per_iteration: 4,
            max_depth: 12,
            max_invisible_depth: 4,
            photons_global: 200000,
            photons_caustic: 1000000,
            initial_num_nearest: 100,
            background_color: Spectrum::zero(),
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq)]
#[non_exhaustive]
pub enum CoreRendererConfigurationError {
    #[snafu(display("number of iterations is not positive"))]
    InvalidIterations,
    #[snafu(display("sample per pixel per iteration is not positive"))]
    InvalidSppPerIteration,
    #[snafu(display("max depth is not positive"))]
    InvalidMaxDepth,
    #[snafu(display("max invisible depth is not positive"))]
    NonPositiveMaxInvisibleDepth,
    #[snafu(display("max invisible depth is larger than max depth"))]
    ExceededMaxInvisibleDepth,
    #[snafu(display("initial number of nearest is not positive"))]
    InvalidInitialNumNearest,
}

#[derive(Debug, Clone, PartialEq)]
struct Pixel {
    global: Option<Observation>,
    caustic: Option<Observation>,
}

impl Pixel {
    fn new() -> Self {
        Self {
            global: None,
            caustic: None,
        }
    }

    fn radiance(
        &mut self,
        cont: Contribution,
        emitted_global: usize,
        emitted_caustic: usize,
    ) -> Spectrum {
        if let Some(flux) = cont.global() {
            if let Some(global) = &mut self.global {
                global.accumulate(flux);
            } else if !flux.is_empty() {
                self.global = Some(Observation::new(flux));
            }
        }
        if let Some(flux) = cont.caustic() {
            if let Some(caustic) = &mut self.caustic {
                caustic.accumulate(flux);
            } else if !flux.is_empty() {
                self.caustic = Some(Observation::new(flux));
            }
        }
        cont.light()
            + (self.global.as_ref()).map_or(Spectrum::zero(), |o| o.radiance(emitted_global))
            + (self.caustic.as_ref()).map_or(Spectrum::zero(), |o| o.radiance(emitted_caustic))
    }

    fn get_policy_global(&self, default_num: usize) -> SearchPolicy {
        if let Some(observation) = &self.global {
            SearchPolicy::Radius(observation.radius)
        } else {
            SearchPolicy::Nearest(default_num)
        }
    }

    fn get_policy_caustic(&self, default_num: usize) -> SearchPolicy {
        if let Some(observation) = &self.caustic {
            SearchPolicy::Radius(observation.radius)
        } else {
            SearchPolicy::Nearest(default_num)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Observation {
    flux: Spectrum,
    num: usize,
    radius: Val,
}

impl Observation {
    const NUM_ATTENUATION: Val = Val(0.75);

    fn new(flux: &FluxEstimation) -> Self {
        Self {
            flux: flux.flux(),
            num: flux.num().into(),
            radius: flux.radius(),
        }
    }

    fn accumulate(&mut self, flux: &FluxEstimation) {
        let total = self.num + usize::from(flux.num() * Self::NUM_ATTENUATION);
        let fraction = Val::from(total) / (Val::from(self.num) + flux.num());
        self.flux = (self.flux + flux.flux()) * fraction;
        self.num = total;
        self.radius *= fraction.sqrt();
    }

    fn radiance(&self, num_emitted: usize) -> Spectrum {
        let area = Val::PI * self.radius.powi(2);
        self.flux / (area * Val::from(num_emitted))
    }
}
