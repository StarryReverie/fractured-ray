use rand::prelude::*;
use rayon::prelude::*;
use snafu::prelude::*;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::Color;
use crate::domain::entity::Scene;
use crate::domain::entity::shape::DisRange;
use crate::domain::geometry::Val;
use crate::domain::image::Image;
use crate::domain::ray::{Ray, RayTrace};

#[cfg_attr(test, mockall::automock)]
pub trait Renderer: Send + Sync + 'static {
    fn render(&self) -> Image;

    fn trace(&self, ray_trace: RayTrace, range: DisRange, depth: usize) -> Ray;
}

#[derive(Debug)]
pub struct CoreRenderer {
    camera: Camera,
    scene: Scene,
    config: Configuration,
}

impl CoreRenderer {
    pub fn new(
        camera: Camera,
        scene: Scene,
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

    fn render_pixel(&self, (row, column): (usize, usize)) -> Ray {
        let mut rng = rand::rng();

        let offset = Offset::new(
            Val::from(rng.random::<f64>()),
            Val::from(rng.random::<f64>()),
        )
        .expect("offset range should be bounded to [0, 1)");

        let point = self
            .camera
            .calc_point_in_pixel(row, column, offset)
            .expect("row and column should not be out of bound");

        let direction = (point - self.camera.position())
            .normalize()
            .expect("focal length should be positive");

        let ray = self.trace(RayTrace::new(point, direction), DisRange::positive(), 1);
        ray
    }
}

impl Renderer for CoreRenderer {
    fn render(&self) -> Image {
        let mut image = Image::new(self.camera.resolution().clone());

        let height = image.resolution().height();
        let width = image.resolution().width();

        let meshgrid = (0..height)
            .into_iter()
            .flat_map(|r| (0..width).into_iter().map(move |c| (r, c)))
            .collect::<Vec<_>>();

        println!("Renderering started");

        for batch in 1..=self.config.ssaa_samples {
            let res = meshgrid
                .par_iter()
                .cloned()
                .map(|pos| (pos, self.render_pixel(pos)))
                .collect_vec_list();

            for ((row, column), ray) in res.into_iter().flatten() {
                image.record(row, column, ray.color());
            }

            println!("finished batch #{batch}");
        }

        image
    }

    fn trace(&self, ray_trace: RayTrace, range: DisRange, depth: usize) -> Ray {
        if depth > self.config.tracing_depth {
            return Ray::new(
                RayTrace::new(ray_trace.start(), -ray_trace.direction()),
                Color::BLACK,
            );
        }

        let res = self.scene.find_intersection(&ray_trace, range);
        if let Some((intersection, entity)) = res {
            entity.shade(self, ray_trace, intersection, depth)
        } else {
            Ray::new(
                RayTrace::new(ray_trace.start(), -ray_trace.direction()),
                Color::WHITE,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub ssaa_samples: usize,
    pub tracing_depth: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            ssaa_samples: 4,
            tracing_depth: 8,
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
