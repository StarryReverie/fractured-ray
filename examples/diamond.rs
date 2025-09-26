use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::core::{Albedo, Spectrum};
use fractured_ray::domain::material::primitive::{
    Emissive, Glossy, GlossyPredefinition, Refractive,
};
use fractured_ray::domain::math::algebra::Vector;
use fractured_ray::domain::math::geometry::{Direction, Distance, Normal, Point, SpreadAngle};
use fractured_ray::domain::math::numeric::Val;
use fractured_ray::domain::math::transformation::{Rotation, Translation};
use fractured_ray::domain::medium::primitive::Isotropic;
use fractured_ray::domain::renderer::{CoreRenderer, CoreRendererConfiguration, Renderer};
use fractured_ray::domain::scene::entity::{
    BvhEntitySceneBuilder, EntitySceneBuilder, TypedEntitySceneBuilder,
};
use fractured_ray::domain::scene::volume::{
    BvhVolumeSceneBuilder, TypedVolumeSceneBuilder, VolumeSceneBuilder,
};
use fractured_ray::domain::shape::mesh::{MeshConstructor, MeshInstanceConstructor};
use fractured_ray::domain::shape::primitive::{Aabb, Plane, Polygon};
use fractured_ray::infrastructure::image::PngWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut scene = BvhEntitySceneBuilder::new();

    let diamond = get_diamond_mesh()?;

    scene.add_constructor(
        MeshInstanceConstructor::wrap(diamond)
            .rotate(Rotation::new(
                Direction::y_direction(),
                Direction::normalize(Vector::new(Val(-2.0), Val(2.5), Val(2.0)))?,
                Val(0.0),
            ))
            .translate(Translation::new(Vector::new(Val(3.0), Val(0.0), Val(-2.0)))),
        Refractive::new(Albedo::broadcast(Val(0.9))?, Val(2.417))?,
    );

    scene.add(
        Plane::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Normal::y_direction(),
        ),
        Glossy::lookup(GlossyPredefinition::Iron, Val(0.3))?,
    );

    for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let spacing = Val(6.0);
        let (dx, dz) = (spacing * Val::from(dx), spacing * Val::from(dz));
        scene.add(
            Polygon::new([
                Point::new(Val(-4.0) + dx, Val(18.0), Val(-4.0) + dz),
                Point::new(Val(4.0) + dx, Val(18.0), Val(-4.0) + dz),
                Point::new(Val(4.0) + dx, Val(18.0), Val(4.0) + dz),
                Point::new(Val(-4.0) + dx, Val(18.0), Val(4.0) + dz),
            ])?,
            Emissive::new(Spectrum::broadcast(Val(2.0)), SpreadAngle::hemisphere()),
        );
    }

    let camera = Camera::new(
        Point::new(Val(0.0), Val(5.0), Val(80.0)),
        -Direction::z_direction(),
        Resolution::new(720, (16, 9))?,
        Distance::new(Val(2.0))?,
        Distance::new(Val(5.0))?,
    );

    let mut vol_scene = BvhVolumeSceneBuilder::new();

    vol_scene.add(
        Aabb::new(
            Point::new(Val(-100.0), Val(-100.0), Val(-100.0)),
            Point::new(Val(100.0), Val(100.0), Val(100.0)),
        ),
        Isotropic::new(
            (Albedo::WHITE * Val(0.5)).into(),
            Spectrum::broadcast(Val(1000.0)),
        )?,
    );

    let renderer = CoreRenderer::new(
        camera,
        scene.build(),
        vol_scene.build(),
        CoreRendererConfiguration::default()
            .with_iterations(256)
            .with_photons_caustic(0),
    )?;

    let image = renderer.render();
    PngWriter::new(File::create("output/diamond.png")?).write(image)?;

    Ok(())
}

fn get_diamond_mesh() -> Result<MeshConstructor, Box<dyn Error>> {
    let mesh = MeshConstructor::new(
        vec![
            // 0
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            // 1 -> 8
            Point::new(Val(0.0), Val(6.0), Val(5.0)),
            Point::new(Val(3.53553), Val(6.0), Val(3.53553)),
            Point::new(Val(5.0), Val(6.0), Val(0.0)),
            Point::new(Val(3.53553), Val(6.0), Val(-3.53553)),
            Point::new(Val(0.0), Val(6.0), Val(-5.0)),
            Point::new(Val(-3.53553), Val(6.0), Val(-3.53553)),
            Point::new(Val(-5.0), Val(6.0), Val(0.0)),
            Point::new(Val(-3.53553), Val(6.0), Val(3.53553)),
            // 9 -> 16
            Point::new(Val(1.33939), Val(8.0), Val(3.23358)),
            Point::new(Val(3.23358), Val(8.0), Val(1.33939)),
            Point::new(Val(3.23358), Val(8.0), Val(-1.33939)),
            Point::new(Val(1.33939), Val(8.0), Val(-3.23358)),
            Point::new(Val(-1.33939), Val(8.0), Val(-3.23358)),
            Point::new(Val(-3.23358), Val(8.0), Val(-1.33939)),
            Point::new(Val(-3.23358), Val(8.0), Val(1.33939)),
            Point::new(Val(-1.33939), Val(8.0), Val(3.23358)),
        ],
        vec![
            // Layer 1
            vec![2, 1, 0],
            vec![3, 2, 0],
            vec![4, 3, 0],
            vec![5, 4, 0],
            vec![6, 5, 0],
            vec![7, 6, 0],
            vec![8, 7, 0],
            vec![1, 8, 0],
            // Layer 2
            vec![1, 2, 9],
            vec![2, 3, 10],
            vec![3, 4, 11],
            vec![4, 5, 12],
            vec![5, 6, 13],
            vec![6, 7, 14],
            vec![7, 8, 15],
            vec![8, 1, 16],
            // Layer 3
            vec![1, 9, 16],
            vec![2, 10, 9],
            vec![3, 11, 10],
            vec![4, 12, 11],
            vec![5, 13, 12],
            vec![6, 14, 13],
            vec![7, 15, 14],
            vec![8, 16, 15],
            // Top
            vec![9, 10, 11, 12, 13, 14, 15, 16],
        ],
    )?;

    Ok(mesh)
}
