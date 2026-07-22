# GR-Raytracer

A dual CPU / GPU general relativity ray tracer written in Rust and WGSL.

<!--
![Rendered black hole](assets/rendered_black_hole.png)
-->

The ray tracer features an integrator for the geodesic equation in Schwarzschild
and Euclidean geometries. It steps light rays along null geodesics using Euler's
method or Runge–Kutta 4. Supported objects are discs and spheres with arbitrary
image textures and diffuse or metallic surfaces.

## Usage

Make sure you have Cargo and the Rust compiler installed, then run:

```bash
cargo run --release
```

Rendering parameters live in `config.rs`; the scene is defined in `main.rs`.

## [Notes on general relativity and implementation details](theory.md)