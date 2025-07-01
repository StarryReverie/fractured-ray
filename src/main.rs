use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::entity::Scene;
use fractured_ray::domain::entity::shape::Sphere;
use fractured_ray::domain::geometry::{Point, Vector};
use fractured_ray::domain::renderer::{Configuration, Renderer};
use fractured_ray::infrastructure::image::PpmWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(0.0, 3.0, 0.0),
        Vector::new(0.0, -2.0, -3.0).normalize()?,
        Resolution::new(1440, (16, 9))?,
        2.0,
        1.0,
    )?;

    let mut scene = Scene::new();
    scene.add(Sphere::new(Point::new(0.0, 0.0, -3.0), 1.0)?);
    scene.add(Sphere::new(Point::new(-1.0, 0.0, -4.0), 1.0)?);
    scene.add(Sphere::new(Point::new(1.0, 0.0, -4.0), 1.0)?);

    let renderer = Renderer::new(camera, scene, Configuration::default())?;
    let image = renderer.render();
    PpmWriter::new(File::create("output/image.ppm")?).write(image)?;

    Ok(())
}
