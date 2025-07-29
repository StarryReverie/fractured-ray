use std::time::Duration;

use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use rand::prelude::*;
use rayon::prelude::*;
use snafu::prelude::*;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::Color;
use crate::domain::entity::{BvhScene, Scene};
use crate::domain::image::Image;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::photon::{PhotonMap, PhotonRay};

use super::{PmContext, PmState, RtContext, StoragePolicy};

#[cfg_attr(test, mockall::automock)]
pub trait Renderer: Send + Sync + 'static {
    fn render(&self) -> Image;

    fn trace<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        ray: Ray,
        range: DisRange,
        depth: usize,
    ) -> Color;

    fn emit<'a>(
        &'a self,
        context: &mut PmContext<'a>,
        state: PmState,
        photon: PhotonRay,
        range: DisRange,
    );
}

#[derive(Debug)]
pub struct CoreRenderer {
    camera: Camera,
    scene: BvhScene,
    config: Configuration,
}

impl CoreRenderer {
    pub fn new(
        camera: Camera,
        scene: BvhScene,
        config: Configuration,
    ) -> Result<Self, ConfigurationError> {
        ensure!(config.ssaa_samples > 0, InvalidSsaaSamplesSnafu);
        ensure!(config.tracing_depth > 0, InvalidTracingDepthSnafu);

        Ok(Self {
            camera,
            scene,
            config,
        })
    }

    fn render_pixel(
        &self,
        (row, column): (usize, usize),
        pm_global: &PhotonMap,
        pm_caustic: &PhotonMap,
    ) -> Color {
        let mut rng = rand::rng();

        let offset = Offset::new(Val(rng.random()), Val(rng.random()))
            .expect("offset range should be bounded to [0, 1)");

        let point = (self.camera)
            .calc_point_in_pixel(row, column, offset)
            .expect("row and column should not be out of bound");

        let direction = (point - self.camera.position())
            .normalize()
            .expect("focal length should be positive");

        let mut context = RtContext::new(self, &self.scene, &mut rng, pm_global, pm_caustic);
        self.trace(
            &mut context,
            Ray::new(point, direction),
            DisRange::positive(),
            1,
        )
    }

    fn init_progress_bar(&self) -> ProgressBar {
        const TEMPLATE: &str = "{msg:>12.green.bold} [{spinner:.yellow.bold}] [{bar:50.cyan.bold/blue.bold}] ({percent}%) [Elapsed: {elapsed_precise} ETA: {eta_precise}]";
        let style = ProgressStyle::with_template(TEMPLATE)
            .unwrap()
            .tick_chars(r#"|/-\|/-\+"#)
            .progress_chars("=>-");
        let bar = ProgressBar::new(self.config.ssaa_samples as u64)
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
                if let Some(photon) = self.scene.get_emitters().sample_photon(&mut rng) {
                    let mut context = PmContext::new(self, &self.scene, &mut rng, &mut photons);
                    let state = PmState::new(false, policy);
                    self.emit(
                        &mut context,
                        state,
                        photon.into_photon(),
                        DisRange::positive(),
                    );
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
        let mut image = Image::new(self.camera.resolution().clone());

        let pm_global =
            self.build_photon_map(StoragePolicy::Global, self.config.global_photon_number);
        let pm_caustic =
            self.build_photon_map(StoragePolicy::Caustic, self.config.caustic_photon_number);

        let meshgrid = (0..image.resolution().height())
            .flat_map(|r| (0..image.resolution().width()).map(move |c| (r, c)))
            .collect::<Vec<_>>();

        let pb = self.init_progress_bar();
        for _ in (0..self.config.ssaa_samples).progress_with(pb) {
            let res = (meshgrid.par_iter())
                .cloned()
                .map(|pos| (pos, self.render_pixel(pos, &pm_global, &pm_caustic)))
                .collect_vec_list();

            for ((row, column), color) in res.into_iter().flatten() {
                image.record(row, column, color);
            }
        }

        image
    }

    fn trace<'a>(
        &'a self,
        context: &mut RtContext<'a>,
        ray: Ray,
        range: DisRange,
        depth: usize,
    ) -> Color {
        if depth > self.config.tracing_depth {
            return Color::BLACK;
        }

        let res = context.scene().find_intersection(&ray, range);
        if let Some((intersection, id)) = res {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            material.shade(context, ray, intersection, depth)
        } else {
            self.config.background_color
        }
    }

    fn emit<'a>(
        &'a self,
        context: &mut PmContext<'a>,
        state: PmState,
        photon: PhotonRay,
        range: DisRange,
    ) {
        let res = context.scene().find_intersection(photon.ray(), range);
        if let Some((intersection, id)) = res {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            material.receive(context, state, photon, intersection);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub ssaa_samples: usize,
    pub tracing_depth: usize,
    pub background_color: Color,
    pub global_photon_number: usize,
    pub caustic_photon_number: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            ssaa_samples: 4,
            tracing_depth: 8,
            background_color: Color::BLACK,
            global_photon_number: 10000,
            caustic_photon_number: 10000,
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigurationError {
    #[snafu(display("SSAA samples for each pixel is not positive"))]
    InvalidSsaaSamples,
    #[snafu(display("tracing depth is not positive"))]
    InvalidTracingDepth,
}
