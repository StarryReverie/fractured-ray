# Fractured-Ray

## Overview

Fractured-Ray is a raytracer implemented in Rust.

This project is currently under active development.

## Features

- Path tracing: global illumination, soft shadows, etc.
- Shape primitives: triangles, polygons, spheres, meshes, etc.
- Material primitives: diffuse, specular, refractive, scattering, etc.
- Transformation: rotation & translation
- Parallel rendering

## Examples

Cornell Box

![Cornell Box](docs/images/cornell-box.png)

Diamond

![Diamond](docs/images/diamond.png)

Teapot

![Teapot](docs/images/teapot.png)

## TODOs

- [x] Algorithms
  - [x] Path Tracing
  - [x] Distribution Ray Tracing
  - [x] Ray-Object Intersection Acceleration Structure
  - [x] BSDF & Cosine-Weighted Sampling
  - [x] Light Sampling
  - [x] Multiple Importance Sampling
  - [x] Stochastic Progressive Photon Mapping
- [ ] Shapes
  - [x] Common Shape Primitives
    - [x] Planes
    - [x] Polygons
    - [x] Spheres
    - [x] Triangles
  - [x] Meshes
  - [x] Instance & Transformation
  - [ ] Volumes
  - [ ] Shading Normal
- [ ] Materials
  - [x] Common Material Primitives
    - [x] Blurry
    - [x] Diffuse
    - [x] Emissive
    - [x] Glossy
    - [x] Mixed
    - [x] Refractive
    - [x] Scattering
    - [x] Specular
- [ ] Textures
  - [ ] Colors
  - [ ] Checker Board
  - [ ] Simplex Noise
  - [ ] Image
- [ ] Infrastructure
  - [x] Progress Bar
  - [ ] CLI Options
  - [ ] Description DSL
  - [ ] External Model Import

# License

Copyright (C) 2025 Justin Chen

This project is licensed under the MIT License.
