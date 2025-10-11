use std::error::Error;

use fractured_ray::infrastructure::description::{DescriptionLoader, TomlDescriptionLoader};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading cornell_box.toml...");
    let loader = TomlDescriptionLoader::new("examples/configs/cornell_box.toml");
    let description = loader.load()?;
    println!("✓ Successfully loaded cornell_box.toml");
    println!("  Renderer iterations: {}", description.renderer_config.iterations());
    println!("  Camera position: {:?}", description.camera.position());
    
    println!("\nLoading simple_scene.toml...");
    let loader = TomlDescriptionLoader::new("examples/configs/simple_scene.toml");
    let description = loader.load()?;
    println!("✓ Successfully loaded simple_scene.toml");
    println!("  Renderer iterations: {}", description.renderer_config.iterations());
    println!("  Camera position: {:?}", description.camera.position());
    
    println!("\nLoading teapot.toml...");
    let loader = TomlDescriptionLoader::new("examples/configs/teapot.toml");
    let description = loader.load()?;
    println!("✓ Successfully loaded teapot.toml");
    println!("  Renderer iterations: {}", description.renderer_config.iterations());
    println!("  Camera position: {:?}", description.camera.position());
    
    println!("\n✓ All configuration files loaded successfully!");
    Ok(())
}
