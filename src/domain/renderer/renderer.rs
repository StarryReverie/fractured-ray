use crate::domain::camera::Camera;
use crate::domain::color::Color;
use crate::domain::image::Image;

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
                let color = Color::new(
                    row as f32 / image.resolution().height() as f32,
                    column as f32 / image.resolution().width() as f32,
                    0.0,
                );
                image.record(row, column, color);
            }
        }
        image
    }
}
