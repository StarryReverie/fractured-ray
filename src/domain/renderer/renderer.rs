use std::ops::Bound;

use crate::domain::camera::{Camera, Offset};
use crate::domain::color::Color;
use crate::domain::entity::shape::{DisRange, Shape, Sphere};
use crate::domain::geometry::Point;
use crate::domain::image::Image;
use crate::domain::ray::{Ray, RayTrace};

#[derive(Debug)]
pub struct Renderer {
    camera: Camera,
}

impl Renderer {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }

    pub fn render(&self) -> Image {
        let mut image = Image::new(self.camera.resolution().clone());
        for row in 0..image.resolution().height() {
            for column in 0..image.resolution().width() {
                let point = self
                    .camera
                    .calc_point_in_pixel(row, column, Offset::center())
                    .expect("row and column should not be out of bound");
                let direction = (point - self.camera.position())
                    .normalize()
                    .expect("focal length should be positive");
                let ray = self.trace(
                    RayTrace::new(point, direction),
                    (Bound::Excluded(0.0), Bound::Unbounded),
                );
                image.record(row, column, ray.color());
            }
        }
        image
    }

    fn trace(&self, ray: RayTrace, range: DisRange) -> Ray {
        let sphere = Sphere::new(Point::new(0.0, 0.0, -3.0), 1.0).unwrap();
        if let Some(intersection) = sphere.hit(&ray, range) {
            let normal = intersection.normal();
            Ray::new(
                RayTrace::new(ray.start(), -ray.direction()),
                Color::new(
                    normal.x() / 2.0 + 0.5,
                    normal.y() / 2.0 + 0.5,
                    normal.z() / 2.0 + 0.5,
                ),
            )
        } else {
            Ray::new(RayTrace::new(ray.start(), -ray.direction()), Color::BLACK)
        }
    }
}
