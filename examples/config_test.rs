use std::error::Error;

use fractured_ray::domain::image::external::ImageResource;
use fractured_ray::domain::renderer::{CoreRenderer, Renderer};
use fractured_ray::infrastructure::description::{DescriptionLoader, TomlDescriptionLoader};
use fractured_ray::infrastructure::image::PngImageResource;

fn main() -> Result<(), Box<dyn Error>> {
    let loader = TomlDescriptionLoader::new("examples/configs/cornell_box.toml");
    let description = loader.load()?;

    let renderer = CoreRenderer::new(
        description.camera,
        description.entity_scene,
        description.volume_scene,
        description.renderer_config,
    )?;

    let image = renderer.render();
    PngImageResource::new("output/cornell-box-from-config.png").save(&image)?;

    println!("Rendered image saved to output/cornell-box-from-config.png");
    Ok(())
}
