# Configuration File Format

This document describes the TOML configuration file format for the fractured-ray renderer.

## Overview

The configuration file allows you to define complete scenes without writing Rust code. It supports:
- Renderer configuration
- Camera setup
- Reusable materials, mediums, and textures
- Scene entities (shapes + materials)
- Volume definitions (shapes + mediums)
- External model loading with material overrides

## Sections

### Renderer Configuration

```toml
[renderer]
iterations = 16                    # Number of rendering iterations
spp_per_iteration = 4             # Samples per pixel per iteration
max_depth = 12                    # Maximum ray bounce depth
max_invisible_depth = 4           # Maximum depth for invisible bounces
photons_global = 200000           # Number of global photons
photons_caustic = 1000000         # Number of caustic photons
initial_num_nearest = 100         # Initial number of nearest photons to search
background_color = [0.1, 0.1, 0.1]  # Optional background color [R, G, B]
```

### Camera Configuration

```toml
[camera]
position = [0.0, 5.0, 20.0]       # Camera position [x, y, z]
direction = [0.0, 0.0, -1.0]      # Camera direction (normalized) [x, y, z]
aperture = 0.05                   # Aperture size
focal_length = 0.1                # Focal length

# Resolution can be specified in two ways:
[camera.resolution]
width = 800                       # Width with aspect ratio
aspect = [16, 9]                  # Aspect ratio [width, height]

# Or with exact dimensions:
# [camera.resolution]
# width = 800
# height = 600
```

### Textures

Define reusable textures:

```toml
[textures.texture_name]
type = "constant"
color = [1.0, 1.0, 1.0]          # RGB color

[textures.checker]
type = "checkerboard"
color1 = [1.0, 1.0, 1.0]
color2 = [0.0, 0.0, 0.0]
scale = 2.0                       # Scale factor

[textures.image_tex]
type = "image_map"
path = "path/to/image.png"        # Path to image file

[textures.noise_tex]
type = "noise"
scale = 1.0                       # Noise scale

[textures.vis_normal]
type = "vis_normal"               # Visualize normals

[textures.vis_uv]
type = "vis_uv"                   # Visualize UV coordinates
```

### Materials

Define reusable materials:

```toml
# Diffuse material
[materials.mat_name]
type = "diffuse"
albedo = { type = "constant", color = [1.0, 1.0, 1.0] }  # Inline texture
# Or reference a texture:
# albedo = { texture = "texture_name" }

# Emissive material
[materials.light]
type = "emissive"
radiance = { type = "constant", color = [10.0, 10.0, 10.0] }
beam_angle = "hemisphere"         # or a number in radians

# Specular material (mirror)
[materials.mirror]
type = "specular"
albedo = { type = "constant", color = [0.9, 0.9, 0.9] }

# Refractive material (glass)
[materials.glass]
type = "refractive"
albedo = { type = "constant", color = [1.0, 1.0, 1.0] }
ior = 1.5                         # Index of refraction

# Glossy material (metals)
[materials.metal]
type = "glossy"
predefinition = "iron"            # "iron", "copper", "gold", "aluminum", "silver"
roughness = 0.3                   # 0.0 to 1.0

# Blurry material
[materials.blurry]
type = "blurry"
albedo = { type = "constant", color = [1.0, 1.0, 1.0] }
refractive_index = 1.5           # Optional, defaults to 1.5
roughness = 0.3                   # 0.0 to 1.0

# Mixed material (limited support - uses first material only)
[materials.mixed]
type = "mixed"
materials = [{ material = "mat1" }, { material = "mat2" }]
weights = [0.5, 0.5]
```

### Mediums

Define reusable mediums for volumes:

```toml
[mediums.vacuum]
type = "vacuum"

[mediums.fog]
type = "isotropic"
albedo = [1.0, 1.0, 1.0]          # RGB albedo
mean_free_path = [100.0, 100.0, 100.0]  # Mean free path for each channel

[mediums.smoke]
type = "henyey_greenstein"
albedo = [0.8, 0.8, 0.8]
mean_free_path = [50.0, 50.0, 50.0]
g = 0.3                           # Anisotropy parameter (-1.0 to 1.0)
```

### Entities (Scene Objects)

Define scene entities (shape + material combinations):

```toml
# Sphere
[[entities]]
shape = { type = "sphere", center = [0.0, 1.0, 0.0], radius = 1.0 }
material = { material = "mat_name" }  # Reference material
# Or inline material:
# material = { type = "diffuse", albedo = { type = "constant", color = [1.0, 0.0, 0.0] } }

# Plane
[[entities]]
shape = { type = "plane", point = [0.0, 0.0, 0.0], normal = [0.0, 1.0, 0.0] }
material = { material = "mat_name" }

# Polygon
[[entities]]
shape = { type = "polygon", points = [
    [1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, -1.0]
]}
material = { material = "mat_name" }

# Triangle
[[entities]]
shape = { type = "triangle", vertices = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0]
]}
material = { material = "mat_name" }

# AABB (Axis-Aligned Bounding Box)
[[entities]]
shape = { type = "aabb", min = [0.0, 0.0, 0.0], max = [1.0, 1.0, 1.0] }
material = { material = "mat_name" }
```

### Volumes

Define participating media volumes:

```toml
[[volumes]]
shape = { type = "sphere", center = [0.0, 0.0, 0.0], radius = 5.0 }
medium = { medium = "fog" }  # Reference medium
# Or inline:
# medium = { type = "isotropic", albedo = [1.0, 1.0, 1.0], mean_free_path = [100.0, 100.0, 100.0] }
```

### External Models

Load OBJ models with optional transformations and material overrides:

```toml
[[models]]
path = "path/to/model.obj"

# Optional transformation
[models.transformation]
translation = [1.0, 2.0, 3.0]     # Optional translation vector
[models.transformation.rotation]
from = [0.0, 1.0, 0.0]            # Rotation from direction
to = [1.0, 0.0, 0.0]              # Rotation to direction
roll = 0.0                         # Roll angle in radians

# Optional material overrides (by material name in OBJ file)
[models.material_overrides]
"MaterialName" = { material = "mat_name" }
```

## Complete Example

See `examples/configs/cornell_box.toml` for a complete working example.

## Notes

- All colors are specified as RGB values in the range [0.0, 1.0+]
- Angles are in radians unless otherwise specified
- Paths are relative to the working directory
- The renderer uses a right-handed coordinate system with Y-up
- Mixed materials have limited support and only use the first material in the list
