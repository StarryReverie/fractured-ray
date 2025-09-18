use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::{Albedo, Spectrum};
use fractured_ray::domain::material::primitive::{Diffuse, Emissive, Refractive, Specular};
use fractured_ray::domain::math::geometry::{Direction, Point, SpreadAngle};
use fractured_ray::domain::math::numeric::Val;
use fractured_ray::domain::medium::primitive::{HenyeyGreenstein, Vacuum};
use fractured_ray::domain::renderer::{CoreRenderer, CoreRendererConfiguration, Renderer};
use fractured_ray::domain::scene::entity::{
    BvhEntitySceneBuilder, EntitySceneBuilder, TypedEntitySceneBuilder,
};
use fractured_ray::domain::scene::volume::{
    BvhVolumeSceneBuilder, TypedVolumeSceneBuilder, VolumeSceneBuilder,
};
use fractured_ray::domain::shape::mesh::MeshConstructor;
use fractured_ray::domain::shape::primitive::{Aabb, Polygon, Sphere};
use fractured_ray::infrastructure::image::PngWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let camera = Camera::new(
        Point::new(Val(278.0), Val(273.0), Val(-800.0)),
        Direction::z_direction(),
        Resolution::new(800, (1, 1))?,
        Val(0.025),
        Val(0.035),
    )?;

    let mut builder = BvhEntitySceneBuilder::new();

    // Light
    builder.add(
        Polygon::new([
            Point::new(Val(343.0), Val(548.799), Val(227.0)),
            Point::new(Val(343.0), Val(548.799), Val(332.0)),
            Point::new(Val(213.0), Val(548.799), Val(332.0)),
            Point::new(Val(213.0), Val(548.799), Val(227.0)),
        ])?,
        Emissive::new(
            Spectrum::new(Val(0.9), Val(0.85), Val(0.4)) * Val(10.0),
            SpreadAngle::hemisphere(),
        ),
    );

    // Floor
    builder.add(
        Polygon::new([
            Point::new(Val(552.8), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(0.0), Val(559.2)),
            Point::new(Val(549.6), Val(0.0), Val(559.2)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );

    // Ceiling
    builder.add(
        Polygon::new([
            Point::new(Val(556.0), Val(548.8), Val(0.0)),
            Point::new(Val(556.0), Val(548.8), Val(559.2)),
            Point::new(Val(0.0), Val(548.8), Val(559.2)),
            Point::new(Val(0.0), Val(548.8), Val(0.0)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );

    // Left Wall
    builder.add(
        Polygon::new([
            Point::new(Val(549.6), Val(0.0), Val(0.0)),
            Point::new(Val(549.6), Val(0.0), Val(559.2)),
            Point::new(Val(556.0), Val(548.8), Val(559.2)),
            Point::new(Val(556.0), Val(548.8), Val(0.0)),
        ])?,
        Diffuse::new(Albedo::RED),
    );

    // Right Wall
    builder.add(
        Polygon::new([
            Point::new(Val(0.0), Val(0.0), Val(559.2)),
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(0.0), Val(548.8), Val(0.0)),
            Point::new(Val(0.0), Val(548.8), Val(559.2)),
        ])?,
        Diffuse::new(Albedo::GREEN),
    );

    // Back Wall
    builder.add(
        Polygon::new([
            Point::new(Val(549.6), Val(0.0), Val(559.2)),
            Point::new(Val(0.0), Val(0.0), Val(559.2)),
            Point::new(Val(0.0), Val(548.8), Val(559.2)),
            Point::new(Val(556.0), Val(548.8), Val(559.2)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );

    // Short Block
    builder.add_constructor(
        MeshConstructor::new(
            vec![
                Point::new(Val(130.0), Val(165.0), Val(65.0)),
                Point::new(Val(82.0), Val(165.0), Val(225.0)),
                Point::new(Val(240.0), Val(165.0), Val(272.0)),
                Point::new(Val(290.0), Val(165.0), Val(114.0)),
                Point::new(Val(290.0), Val(0.0), Val(114.0)),
                Point::new(Val(240.0), Val(0.0), Val(272.0)),
                Point::new(Val(130.0), Val(0.0), Val(65.0)),
                Point::new(Val(82.0), Val(0.0), Val(225.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![3, 0, 6, 4],
                vec![3, 4, 5, 2],
                vec![2, 5, 7, 1],
                vec![1, 7, 6, 0],
                vec![4, 6, 7, 5],
            ],
        )?,
        Diffuse::new(Albedo::WHITE),
    );

    // Tall Block
    builder.add_constructor(
        MeshConstructor::new(
            vec![
                Point::new(Val(423.0), Val(330.0), Val(247.0)),
                Point::new(Val(265.0), Val(330.0), Val(296.0)),
                Point::new(Val(314.0), Val(330.0), Val(456.0)),
                Point::new(Val(472.0), Val(330.0), Val(406.0)),
                Point::new(Val(423.0), Val(0.0), Val(247.0)),
                Point::new(Val(472.0), Val(0.0), Val(406.0)),
                Point::new(Val(314.0), Val(0.0), Val(456.0)),
                Point::new(Val(265.0), Val(0.0), Val(296.0)),
            ],
            vec![
                vec![0, 1, 2, 3],
                vec![4, 0, 3, 5],
                vec![5, 3, 2, 6],
                vec![6, 2, 1, 7],
                vec![7, 1, 0, 4],
                vec![4, 5, 6, 7],
            ],
        )?,
        Diffuse::new(Albedo::WHITE),
    );

    // Specular Ball
    builder.add(
        Sphere::new(Point::new(Val(400.0), Val(90.0), Val(180.0)), Val(90.0))?,
        Specular::new((Albedo::WHITE * Val(0.8)).into()),
    );

    // Refractive Ball
    builder.add(
        Sphere::new(Point::new(Val(185.0), Val(240.0), Val(169.5)), Val(75.0))?,
        Refractive::new((Albedo::WHITE * Val(0.8)).into(), Val(1.5))?,
    );

    let scene = builder.build();

    let mut vol_builder = BvhVolumeSceneBuilder::new();

    vol_builder.add(
        Aabb::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Point::new(Val(600.0), Val(600.0), Val(800.0)),
        ),
        HenyeyGreenstein::new(
            (Albedo::WHITE * Val(1.0)).into(),
            Spectrum::broadcast(Val(300.0)),
            Val(0.3),
        )?,
    );

    vol_builder.add(
        Sphere::new(Point::new(Val(185.0), Val(240.0), Val(169.5)), Val(75.0))?,
        Vacuum::new(),
    );

    let volume_scene = vol_builder.build();

    let renderer = CoreRenderer::new(
        camera,
        scene,
        volume_scene,
        CoreRendererConfiguration::default().with_iterations(16),
    )?;
    let image = renderer.render();
    PngWriter::new(File::create("output/cornell-box.png")?).write(image)?;

    Ok(())
}
