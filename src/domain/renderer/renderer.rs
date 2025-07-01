use rand::prelude::*;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::Color;
use crate::domain::entity::Scene;
use crate::domain::entity::shape::{DisRange, RayIntersection};
use crate::domain::image::Image;
use crate::domain::ray::{Ray, RayTrace};

#[derive(Debug)]
pub struct Renderer {
    camera: Camera,
    scene: Scene,
    ssaa_factor: usize,
}

impl Renderer {
    pub fn new(camera: Camera, scene: Scene, ssaa_factor: usize) -> Self {
        Self {
            camera,
            scene,
            ssaa_factor,
        }
    }

    pub fn render(&self) -> Image {
        let mut rng = rand::rng();
        let mut image = Image::new(self.camera.resolution().clone());

        for row in 0..image.resolution().height() {
            for column in 0..image.resolution().width() {
                for _ in 0..self.ssaa_factor {
                    let offset =
                        Offset::new(rng.random_range(0.0..1.0), rng.random_range(0.0..1.0))
                            .expect("offset range should be bounded to [0, 1)");
                    let point = self
                        .camera
                        .calc_point_in_pixel(row, column, offset)
                        .expect("row and column should not be out of bound");

                    let direction = (point - self.camera.position())
                        .normalize()
                        .expect("focal length should be positive");

                    let ray = self.trace(RayTrace::new(point, direction), DisRange::positive());
                    image.record(row, column, ray.color());
                }
            }
        }

        image
    }

    pub fn trace(&self, ray_trace: RayTrace, range: DisRange) -> Ray {
        let intersection = self.scene.find_intersection(&ray_trace, range);
        if let Some(intersection) = intersection {
            self.calc_color(ray_trace, intersection)
        } else {
            Ray::new(
                RayTrace::new(ray_trace.start(), -ray_trace.direction()),
                Color::BLACK,
            )
        }
    }

    fn calc_color(&self, ray_trace: RayTrace, intersection: RayIntersection) -> Ray {
        let normal = intersection.normal();
        Ray::new(
            RayTrace::new(ray_trace.start(), -ray_trace.direction()),
            Color::new(
                normal.x() / 2.0 + 0.5,
                normal.y() / 2.0 + 0.5,
                normal.z() / 2.0 + 0.5,
            ),
        )
    }
}
