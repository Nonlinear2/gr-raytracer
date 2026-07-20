<h1 align="center">GR-Raytracer</h1>
<h4 align="center">A dual CPU / GPU general relativity ray tracer written in Rust and WGSL.</h4> 

The ray-tracer features an integrator for the geodesic equation in schwarzschild and euclidean geometries. The integrator steps light rays along null geodesics and can use euler's method or runge kutta 4. The supported objects are discs and spheres with arbitrary image textures and diffuse or metallic surfaces.

# Usage
Make sure you have Cargo and the Rust compiler installed, then run 
```
cargo run --release
```
You can modify rendering parameters in `config.rs`, and change the scene in `main.rs`.

# The project architecture

The pipeline was CPU only at first, until I re-wrote most of the code in WGSL. Unfortunately, using WGSL has not been a pleasant experience, mainly because of the lack of a proper module system and debugging tools. I also felt that the nature of manifolds better suits an object oriented language. I thus decided to bring back the CPU code, and write a dual pipeline. Now hopefully, I can add new features in rust without necessarly touching the GPU shaders, and update them only when I am sure of the implementation.

# Notes on general relativity and implementation details

To represent spacetime, we start by describing the corresponding 4 dimensional manifold. We start by giving $\mathbb{R}^4$ the maximal atlas induced by the identity chart. Here, the first coordinate will corespond to time, and the other three to space. For our purposes, we will not need to implement charts of $\mathbb{R}^4$ directly, but rather charts of submanifolds of $\mathbb{R}^4$ obtained by fixing the time coordinate. This is because our pipeline only needs to support time independant geometries (for now). 

trait implements $\mathbb{R}^4$ as a semi riemannian manifold.
- 3D submanifolds of $\mathbb{R}^4$ obtained by chosing a time coodinate $t$, which we will call $\mathbb{R}^3_t$

- charts designate maps from coordinates to manifolds and not the opposite.
- World designates the manifold $\mathbb{R}^3_t$ together with the atlas containing a single chart: cartesian coordinates centered on the camera.

- Photon4 objects belong to R^4, and worldphoton objects will be photons at a point in time in world.
- we always write vector_space when talking about a vector space to avoid confusion with "space" meaning the ThreeVector representing space in a FourVector

Note that for now the pipeline only supports time independant geometries.

## Manifolds

### vector chart transitions:
$$v^{(\mathrm{to})} \;=\; \left. D\!\left(\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right)\right|_{p}\; v^{(\mathrm{from})}$$

## The geodesic equation

geodesic equation:
$$\frac{d^2 x^\mu}{d\lambda^2} + \Gamma^\mu_{\alpha\beta}\,\frac{dx^\alpha}{d\lambda}\,\frac{dx^\beta}{d\lambda} = 0$$

turned into a system of order 1:

$$X(\lambda) = 
\begin{pmatrix}
x^\mu\\
k^\mu\\
\end{pmatrix}
$$

$$
X'(\lambda) =
\begin{pmatrix}
\frac{x^\mu}{d\lambda}\\
\frac{k^\mu}{d\lambda}\\
\end{pmatrix}
=
\begin{pmatrix}
k^\mu\\
-\Gamma^\mu_{\alpha\beta}\,k^\nu\,k^\beta\\
\end{pmatrix}
$$

## Black hole physics

### Innermost Stable Circular Orbit:
$$r_{ms} = 3R_s$$
