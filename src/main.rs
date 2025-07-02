use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::Color;
use fractured_ray::domain::entity::Scene;
use fractured_ray::domain::entity::material::Diffuse;
use fractured_ray::domain::entity::shape::{Plane, Sphere};
use fractured_ray::domain::geometry::{Point, UnitVector};
use fractured_ray::domain::renderer::{Configuration, CoreRenderer, Renderer};
use fractured_ray::infrastructure::image::PpmWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(0.0, 1.0, 2.0),
        -UnitVector::z_direction(),
        Resolution::new(1440, (16, 9))?,
        2.0,
        1.0,
    )?;

    let mut scene = Scene::new();
    scene.add(
        Sphere::new(Point::new(0.0, 1.0, -1.0), 1.0)?,
        Diffuse::new(Color::MAGENTA),
    );
    scene.add(
        Sphere::new(Point::new(-1.0, 1.0, -2.0), 1.0)?,
        Diffuse::new(Color::CYAN),
    );
    scene.add(
        Sphere::new(Point::new(1.0, 1.0, -2.0), 1.0)?,
        Diffuse::new(Color::YELLOW),
    );
    scene.add(
        Plane::new(Point::new(0.0, 0.0, -2.0), -UnitVector::y_direction()),
        Diffuse::new(Color::new(0.1, 0.1, 0.1)),
    );

    let renderer = CoreRenderer::new(
        camera,
        scene,
        Configuration {
            ssaa_samples: 16,
            ..Configuration::default()
        },
    )?;
    let image = renderer.render();
    PpmWriter::new(File::create("output/image.ppm")?).write(image)?;

    Ok(())
}
