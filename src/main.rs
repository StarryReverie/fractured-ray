use std::error::Error;
use std::fs::File;

use smallvec::smallvec;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::Color;
use fractured_ray::domain::entity::Scene;
use fractured_ray::domain::entity::material::{Diffuse, Emissive, Specular};
use fractured_ray::domain::entity::shape::{Plane, Polygon, Sphere};
use fractured_ray::domain::geometry::{Point, UnitVector, Val};
use fractured_ray::domain::renderer::{Configuration, CoreRenderer, Renderer};
use fractured_ray::infrastructure::image::PngWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(Val(0.0), Val(2.0), Val(14.0)),
        -UnitVector::z_direction(),
        Resolution::new(1440, (16, 9))?,
        Val(2.0),
        Val(5.0),
    )?;

    let mut scene = Scene::new();

    scene.add(
        Polygon::new([
            Point::new(Val(-1.5), Val(4.0), Val(-2.0)),
            Point::new(Val(1.5), Val(4.0), Val(-2.0)),
            Point::new(Val(1.5), Val(4.0), Val(1.0)),
            Point::new(Val(-1.5), Val(4.0), Val(1.0)),
        ])?,
        Emissive::new(Color::WHITE),
    );

    scene.add(
        Sphere::new(Point::new(Val(0.0), Val(1.0), Val(-1.0)), Val(1.0))?,
        Diffuse::new(Color::MAGENTA),
    );
    scene.add(
        Sphere::new(Point::new(Val(-3.0), Val(1.0), Val(-1.0)), Val(1.0))?,
        Diffuse::new(Color::CYAN),
    );
    scene.add(
        Sphere::new(Point::new(Val(1.0), Val(1.0), Val(-3.0)), Val(1.0))?,
        Diffuse::new(Color::YELLOW),
    );

    scene.add_mesh(
        smallvec![
            Point::new(Val(3.0), Val(0.0), Val(1.0)),
            Point::new(Val(1.0), Val(0.0), Val(1.0)),
            Point::new(Val(1.0), Val(0.0), Val(-1.0)),
            Point::new(Val(3.0), Val(0.0), Val(-1.0)),
            Point::new(Val(2.0), Val(2.0), Val(0.0)),
        ],
        vec![
            smallvec![0, 1, 2, 3],
            smallvec![0, 1, 4],
            smallvec![1, 2, 4],
            smallvec![2, 3, 4],
            smallvec![3, 1, 4],
        ],
        Diffuse::new(Color::WHITE),
    )?;

    scene.add(
        Plane::new(
            Point::new(Val(-4.0), Val(0.0), Val(0.0)),
            UnitVector::x_direction(),
        ),
        Diffuse::new(Color::GREEN),
    );
    scene.add(
        Plane::new(
            Point::new(Val(4.0), Val(0.0), Val(0.0)),
            -UnitVector::x_direction(),
        ),
        Diffuse::new(Color::RED),
    );
    scene.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(15.0)),
            -UnitVector::z_direction(),
        ),
        Diffuse::new(Color::WHITE * Val(0.8)),
    );
    scene.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(-5.0)),
            UnitVector::z_direction(),
        ),
        Diffuse::new(Color::WHITE * Val(0.8)),
    );
    scene.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(-2.0)),
            UnitVector::y_direction(),
        ),
        Specular::new(Color::WHITE * Val(0.4)),
    );
    scene.add(
        Plane::new(
            Point::new(Val(0.0), Val(4.0), Val(-0.0)),
            -UnitVector::y_direction(),
        ),
        Diffuse::new(Color::WHITE * Val(0.8)),
    );

    let renderer = CoreRenderer::new(
        camera,
        scene,
        Configuration {
            ssaa_samples: 64,
            ..Configuration::default()
        },
    )?;
    let image = renderer.render();
    PngWriter::new(File::create("output/image.png")?).write(image)?;

    Ok(())
}
