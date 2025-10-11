# Configuration File Examples

This directory contains example TOML configuration files for the fractured-ray renderer.

## Available Examples

### cornell_box.toml
A classic Cornell box scene with:
- White, red, and green walls
- A specular (mirror) sphere
- A refractive (glass) sphere
- Participating media (Henyey-Greenstein phase function)
- Area light source

### simple_scene.toml
A simple scene demonstrating various materials and shapes:
- Checkered floor pattern
- Multiple materials: diffuse, specular, refractive, glossy
- Different primitive shapes: spheres, plane, polygon, triangle, AABB
- Ceiling light

### teapot.toml
Example showing external OBJ model loading:
- Loads the teapot model from assets
- Demonstrates model import capabilities
- Uses glossy aluminum floor

## Running Examples

To render a scene from a configuration file:

```bash
# Using the config_test example
cargo run --example config_test

# Or create your own loader:
use fractured_ray::infrastructure::description::{DescriptionLoader, TomlDescriptionLoader};

let loader = TomlDescriptionLoader::new("examples/configs/cornell_box.toml");
let description = loader.load()?;

let renderer = CoreRenderer::new(
    description.camera,
    description.entity_scene,
    description.volume_scene,
    description.renderer_config,
)?;

let image = renderer.render();
```

## Configuration Format

See `docs/configuration_format.md` for complete documentation of the TOML configuration format.

## Creating Your Own Scenes

1. Start with one of the example files
2. Modify the camera position and direction
3. Add or modify materials in the `[materials.*]` sections
4. Add entities in `[[entities]]` sections
5. Optionally add volumes in `[[volumes]]` sections
6. Run your scene

## Tips

- Use reusable materials and textures to keep your config files organized
- Start with lower iteration counts (4-8) for quick previews
- Increase iterations for final high-quality renders (16-64+)
- Set `photons_caustic = 0` if you don't need caustics (faster rendering)
- Use `background_color` to set a custom background instead of black
