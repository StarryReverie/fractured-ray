use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::{Albedo, Color};
use fractured_ray::domain::entity::BvhSceneBuilder;
use fractured_ray::domain::material::primitive::{Diffuse, Emissive, Refractive, Specular};
use fractured_ray::domain::math::algebra::{UnitVector, Vector};
use fractured_ray::domain::math::geometry::{Point, Rotation, SpreadAngle, Translation};
use fractured_ray::domain::math::numeric::Val;
use fractured_ray::domain::renderer::{Configuration, CoreRenderer, Renderer};
use fractured_ray::domain::shape::instance::MeshConstructorInstance;
use fractured_ray::domain::shape::mesh::MeshConstructor;
use fractured_ray::domain::shape::primitive::{Plane, Polygon, Sphere};
use fractured_ray::infrastructure::image::PngWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(Val(0.0), Val(2.0), Val(14.0)),
        -UnitVector::z_direction(),
        Resolution::new(1440, (16, 9))?,
        Val(2.0),
        Val(5.0),
    )?;

    let mut builder = BvhSceneBuilder::new();

    builder.add(
        Plane::new(
            Point::new(Val(-4.0), Val(0.0), Val(0.0)),
            UnitVector::x_direction(),
        ),
        Diffuse::new(Albedo::GREEN),
    );
    builder.add(
        Plane::new(
            Point::new(Val(4.0), Val(0.0), Val(0.0)),
            -UnitVector::x_direction(),
        ),
        Diffuse::new(Albedo::RED),
    );
    builder.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(15.0)),
            -UnitVector::z_direction(),
        ),
        Diffuse::new(Albedo::WHITE),
    );
    builder.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(-5.0)),
            UnitVector::z_direction(),
        ),
        Diffuse::new(Albedo::WHITE),
    );
    builder.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(-2.0)),
            UnitVector::y_direction(),
        ),
        Specular::new((Albedo::WHITE * Val(0.4)).into()),
    );
    builder.add(
        Plane::new(
            Point::new(Val(0.0), Val(4.0), Val(-0.0)),
            -UnitVector::y_direction(),
        ),
        Diffuse::new(Albedo::WHITE),
    );

    builder.add(
        Polygon::new([
            Point::new(Val(-2.0), Val(3.999), Val(-2.0)),
            Point::new(Val(2.0), Val(3.999), Val(-2.0)),
            Point::new(Val(2.0), Val(3.999), Val(2.0)),
            Point::new(Val(-2.0), Val(3.999), Val(2.0)),
        ])?,
        Emissive::new(Color::WHITE, SpreadAngle::hemisphere()),
    );

    builder.add(
        Sphere::new(Point::new(Val(0.0), Val(1.0), Val(-1.0)), Val(1.0))?,
        Refractive::new(Albedo::WHITE, Val(1.5))?,
    );
    builder.add(
        Sphere::new(Point::new(Val(-3.0), Val(1.0), Val(-1.0)), Val(1.0))?,
        Diffuse::new(Albedo::CYAN),
    );
    builder.add(
        Sphere::new(Point::new(Val(1.0), Val(1.0), Val(-3.0)), Val(1.0))?,
        Diffuse::new(Albedo::YELLOW),
    );

    builder.add_constructor(
        MeshConstructorInstance::wrap(MeshConstructor::new(
            vec![
                Point::new(Val(1.0), Val(0.0), Val(1.0)),
                Point::new(Val(-1.0), Val(0.0), Val(1.0)),
                Point::new(Val(-1.0), Val(0.0), Val(-1.0)),
                Point::new(Val(1.0), Val(0.0), Val(-1.0)),
                Point::new(Val(0.0), Val(2.0), Val(0.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![0, 1, 4],
                vec![1, 2, 4],
                vec![2, 3, 4],
                vec![3, 0, 4],
            ],
        )?)
        .rotate(Rotation::new(
            UnitVector::y_direction(),
            UnitVector::z_direction(),
            Val::PI / Val(3.0),
        ))
        .translate(Translation::new(Vector::new(Val(2.0), Val(0.0), Val(0.0)))),
        Diffuse::new(Albedo::WHITE),
    );

    let scene = builder.build();

    let renderer = CoreRenderer::new(
        camera,
        scene,
        Configuration {
            iterations: 16,
            ..Configuration::default()
        },
    )?;
    let image = renderer.render();
    PngWriter::new(File::create("output/image.png")?).write(image)?;

    Ok(())
}
