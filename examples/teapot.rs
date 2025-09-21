use std::error::Error;
use std::fs::File;

use fractured_ray::domain::camera::{Camera, Resolution};
use fractured_ray::domain::color::{Albedo, Spectrum};
use fractured_ray::domain::material::primitive::{Diffuse, Emissive, Glossy, GlossyPredefinition};
use fractured_ray::domain::math::geometry::{Direction, Distance, Point, SpreadAngle};
use fractured_ray::domain::math::numeric::Val;
use fractured_ray::domain::renderer::{CoreRenderer, CoreRendererConfiguration, Renderer};
use fractured_ray::domain::scene::entity::{
    BvhEntitySceneBuilder, EntitySceneBuilder, TypedEntitySceneBuilder,
};
use fractured_ray::domain::scene::volume::{BvhVolumeSceneBuilder, VolumeSceneBuilder};
use fractured_ray::domain::shape::primitive::Polygon;
use fractured_ray::infrastructure::image::PngWriter;
use fractured_ray::infrastructure::model::{
    EntityModelLoader, EntityModelLoaderConfiguration, EntityObjModelLoader,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut scene = BvhEntitySceneBuilder::new();

    load_box(scene.as_mut())?;
    load_teapot(scene.as_mut())?;

    let camera = Camera::new(
        Point::new(Val(0.0), Val(5.0), Val(19.7)),
        -Direction::z_direction(),
        Resolution::new(800, (1, 1))?,
        Distance::new(Val(0.025))?,
        Distance::new(Val(0.035))?,
    );

    let renderer = CoreRenderer::new(
        camera,
        scene.build(),
        BvhVolumeSceneBuilder::new().build(),
        CoreRendererConfiguration::default()
            .with_iterations(24)
            .with_photons_caustic(0)
            .with_background_color(Spectrum::broadcast(Val(0.01))),
    )?;

    let image = renderer.render();
    PngWriter::new(File::create("output/teapot.png")?).write(image)?;

    Ok(())
}

fn load_box(scene: &mut dyn EntitySceneBuilder) -> Result<(), Box<dyn Error>> {
    scene.add(
        Polygon::new([
            Point::new(Val(1.2), Val(9.9999), Val(-0.9)),
            Point::new(Val(1.2), Val(9.9999), Val(0.9)),
            Point::new(Val(-1.2), Val(9.9999), Val(0.9)),
            Point::new(Val(-1.2), Val(9.9999), Val(-0.9)),
        ])?,
        Emissive::new(
            Spectrum::new(Val(0.9), Val(0.85), Val(0.8)) * Val(10.0),
            SpreadAngle::hemisphere(),
        ),
    );
    scene.add(
        Polygon::new([
            Point::new(Val(5.0), Val(0.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(0.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(0.0), Val(5.0)),
            Point::new(Val(5.0), Val(0.0), Val(5.0)),
        ])?,
        Glossy::lookup(GlossyPredefinition::Copper, Val(0.3))?,
    );
    scene.add(
        Polygon::new([
            Point::new(Val(5.0), Val(10.0), Val(-5.0)),
            Point::new(Val(5.0), Val(10.0), Val(5.0)),
            Point::new(Val(-5.0), Val(10.0), Val(5.0)),
            Point::new(Val(-5.0), Val(10.0), Val(-5.0)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );
    scene.add(
        Polygon::new([
            Point::new(Val(-5.0), Val(0.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(10.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(10.0), Val(5.0)),
            Point::new(Val(-5.0), Val(0.0), Val(5.0)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );
    scene.add(
        Polygon::new([
            Point::new(Val(5.0), Val(0.0), Val(-5.0)),
            Point::new(Val(5.0), Val(0.0), Val(5.0)),
            Point::new(Val(5.0), Val(10.0), Val(5.0)),
            Point::new(Val(5.0), Val(10.0), Val(-5.0)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );
    scene.add(
        Polygon::new([
            Point::new(Val(5.0), Val(0.0), Val(-5.0)),
            Point::new(Val(5.0), Val(10.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(10.0), Val(-5.0)),
            Point::new(Val(-5.0), Val(0.0), Val(-5.0)),
        ])?,
        Diffuse::new(Albedo::WHITE),
    );
    Ok(())
}

fn load_teapot(scene: &mut dyn EntitySceneBuilder) -> Result<(), Box<dyn Error>> {
    let loader = EntityObjModelLoader::parse("assets/models/teapot/teapot.obj")?;
    let config = EntityModelLoaderConfiguration::default();
    loader.load(scene, config)?;
    Ok(())
}
