<h1 align="center">GR-Raytracer</h1>
<h3 align="center">A dual CPU / GPU general relativity ray tracer written in Rust and WGSL.</h3> 

![Rendered black hole](./assets/images/rendered_black_hole.png?raw=true)

This project traces light in pseudo-Riemannian geometries to render images of black holes and other curved spacetime scenes. To do so, it features an integrator for the geodesic equation using Euler or RK4 stepping. For now the supported objects in the scene are discs and spheres with arbitrary image textures and diffuse or metallic surfaces. Currently only time independent geometries are supported.

# Usage

Make sure you have Git, Cargo and the Rust compiler installed.
Then open a terminal, clone the repo using `git clone https://github.com/Nonlinear2/gr-raytracer`, navigate to the project's root directory and run 
```
cargo run --release
```
A window with the rendered image should appear on the screen.
The default parameters are a 270x480 image using the GPU pipeline, and it should render in a few seconds. Using the CPU is much slower.

You can modify rendering parameters in `config.rs`, and change the scene in `main.rs`.

# A few words about the project

The pipeline was CPU only at first, until I re-wrote most of the code in WGSL. Unfortunately, using WGSL has not been a pleasant experience, mainly because of the lack of a proper module system, abstractions, and debugging tools. The nature of manifolds also blended nicely with Rust's trait system. So I decided to bring back the CPU code and write a dual pipeline. Now hopefully, I can prototype features in Rust without necessarily touching the GPU shaders and update them occasionally once I am confident in the implementation. Time will tell if this was a good idea!

# Notes on general relativity and implementation details

You can find all the details **[here](https://nonlinear2.github.io/gr-raytracer/theory/)**!