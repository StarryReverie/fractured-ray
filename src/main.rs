use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::geometry::{Point, UnitVector};
use fractured_ray::domain::renderer::Renderer;
use fractured_ray::infrastructure::image::PpmWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(0.0, 0.0, 0.0),
        -UnitVector::z_direction(),
        Resolution::new(1440, (16, 9)).unwrap(),
        2.0,
        1.0,
    )
    .unwrap();
    let renderer = Renderer::new(camera, 4);
    let image = renderer.render();
    PpmWriter::new(File::create("output/image.ppm")?).write(image)?;
    Ok(())
}
