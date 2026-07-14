# GR-Raytracer
A GPU accelerated general relativity ray tracer written in Rust and WGSL.

# Usage
Make sure you have cargo and the Rust compiler installed, then run 
```
cargo run --release
```
You can modify rendering parameters in `config.rs`, and change the scene in `main.rs`.

Note that for now the pipeline only supports time independant geometries.

# Notes on general relativity and implementation details

## Code conventions
- The code implements the following manifolds:
- R^4 as a semi riemannian manifold, which we will call "R^4"
- 3D submanifolds of R^4 obtained by chosing a time coodinate t, which we will call "R^3_t"
- charts designate maps from coordinates to manifolds and not the opposite.
- world designates the manifold "R^3_t" together with the atlas containing a single chart: cartesian coordinates centered on the camera.

- photon objects belong to R^4, and worldphoton objects will be photons at a point in time in world.
- we always write vector_space when talking about a vector space to avoid confusion with "space" meaning the ThreeVector representing space in a FourVector

- vec3<f32> is used only if a single chart is accepted by the function.

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
