use std::time::Duration;

use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use rand::prelude::*;
use rayon::prelude::*;
use snafu::prelude::*;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::Color;
use crate::domain::entity::{BvhScene, Scene};
use crate::domain::image::Image;
use crate::domain::math::algebra::Vector;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::Ray;
use crate::domain::ray::photon::{PhotonMap, PhotonRay};

use super::{
    Contribution, PhotonInfo, PmContext, PmState, Renderer, RtContext, RtState, StoragePolicy,
};

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
        ensure!(config.iterations > 0, InvalidIterationsSnafu);
        ensure!(config.max_depth > 0, InvalidMaxDepthSnafu);
        ensure!(
            config.max_invisible_depth > 0,
            NonPositiveMaxInvisibleDepthSnafu,
        );
        ensure!(
            config.max_invisible_depth <= config.max_depth,
            ExceededMaxInvisibleDepthSnafu,
        );

        Ok(Self {
            camera,
            scene,
            config,
        })
    }

    fn render_pixel(
        &self,
        (row, column): (usize, usize),
        pixel: &mut Pixel,
        photon_global: PhotonInfo<'_>,
        photon_caustic: PhotonInfo<'_>,
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

        let mut context = RtContext::new(
            self,
            &self.scene,
            &mut rng,
            &self.config,
            photon_global,
            photon_caustic,
        );
        let state = RtState::new();
        let res = self.trace(
            &mut context,
            state,
            Ray::new(point, direction),
            DisRange::positive(),
        );
        pixel.radiance(
            &res,
            context.photon_global().emitted(),
            context.photon_casutic().emitted(),
        )
    }

    fn init_progress_bar(&self) -> ProgressBar {
        const TEMPLATE: &str = "{msg:>12.green.bold} [{spinner:.yellow.bold}] [{bar:50.cyan.bold/blue.bold}] ({percent}%) [Elapsed: {elapsed_precise} ETA: {eta_precise}]";
        let style = ProgressStyle::with_template(TEMPLATE)
            .unwrap()
            .tick_chars(r#"|/-\|/-\+"#)
            .progress_chars("=>-");
        let bar = ProgressBar::new(self.config.iterations as u64)
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
        let height = image.resolution().height();
        let width = image.resolution().width();

        let mut pixels = vec![vec![Pixel::new(); height]; width];
        let mut num_global = 0;
        let mut num_caustic = 0;

        let pb = self.init_progress_bar();
        for _ in (0..self.config.iterations).progress_with(pb) {
            let pmg = self.build_photon_map(StoragePolicy::Global, self.config.photons_global);
            let pmc = self.build_photon_map(StoragePolicy::Caustic, self.config.photons_caustic);
            num_global += self.config.photons_global;
            num_caustic += self.config.photons_caustic;

            let meshgrid = (pixels.par_iter_mut().enumerate())
                .map(|(r, p)| (r, p.par_iter_mut().enumerate()))
                .flat_map(|(r, pi)| pi.map(move |(c, p)| ((r, c), p)));
            let res = meshgrid
                .map(|(pos, pixel)| {
                    let pg = PhotonInfo::new(&pmg, pixel.radius_global(), num_global);
                    let pc = PhotonInfo::new(&pmc, pixel.radius_caustic(), num_caustic);
                    (pos, self.render_pixel(pos, pixel, pg, pc))
                })
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
        state: RtState,
        ray: Ray,
        range: DisRange,
    ) -> Contribution {
        let state = state.increment_depth();
        if state.depth() > self.config.max_depth {
            return Contribution::new();
        }

        let res = context.scene().find_intersection(&ray, range);
        if let Some((intersection, id)) = res {
            let entities = context.scene().get_entities();
            let material = entities.get_material(id.material_id()).unwrap();
            material.shade(context, state, ray, intersection)
        } else {
            Contribution::from(self.config.background_color)
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
    pub iterations: usize,
    pub max_depth: usize,
    pub max_invisible_depth: usize,
    pub background_color: Color,
    pub photons_global: usize,
    pub photons_caustic: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            iterations: 4,
            max_depth: 12,
            max_invisible_depth: 4,
            background_color: Color::BLACK,
            photons_global: 50000,
            photons_caustic: 1000000,
        }
    }
}

#[derive(Debug, Snafu, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigurationError {
    #[snafu(display("number of iterations is not positive"))]
    InvalidIterations,
    #[snafu(display("max depth is not positive"))]
    InvalidMaxDepth,
    #[snafu(display("max invisible depth is not positive"))]
    NonPositiveMaxInvisibleDepth,
    #[snafu(display("max invisible depth is larger than max depth"))]
    ExceededMaxInvisibleDepth,
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
        cont: &Contribution,
        num_emitted_global: usize,
        num_emitted_caustic: usize,
    ) -> Color {
        if let Some(global) = &mut self.global {
            global.accumulate(cont.flux_global(), cont.num_global().into());
        } else {
            self.global = Some(Observation::new(
                cont.flux_global(),
                cont.num_global().into(),
                Val(10.0),
            ));
        }
        if let Some(caustic) = &mut self.caustic {
            caustic.accumulate(cont.flux_caustic(), cont.num_caustic().into());
        } else {
            self.caustic = Some(Observation::new(
                cont.flux_caustic(),
                cont.num_caustic().into(),
                Val(10.0),
            ));
        }
        cont.light()
            + self.global.as_ref().unwrap().radiance(num_emitted_global)
            + self.caustic.as_ref().unwrap().radiance(num_emitted_caustic)
    }

    fn radius_global(&self) -> Option<Val> {
        self.global.as_ref().map(|o| o.radius)
    }

    fn radius_caustic(&self) -> Option<Val> {
        self.caustic.as_ref().map(|o| o.radius)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Observation {
    flux: Vector,
    num: usize,
    radius: Val,
}

impl Observation {
    const NUM_ATTENUATION: Val = Val(0.75);

    fn new(flux: Vector, num: usize, radius: Val) -> Self {
        Self { flux, num, radius }
    }

    fn accumulate(&mut self, flux: Vector, num: usize) {
        let total = self.num + usize::from(Val::from(num) * Self::NUM_ATTENUATION);
        let fraction = Val::from(total) / Val::from(self.num + num);
        self.flux = (self.flux + flux) * fraction;
        self.num = total;
        self.radius = self.radius * fraction.sqrt();
    }

    fn radiance(&self, num_emitted: usize) -> Color {
        let area = Val::PI * self.radius.powi(2);
        (self.flux / (area * Val::from(num_emitted))).into()
    }
}
