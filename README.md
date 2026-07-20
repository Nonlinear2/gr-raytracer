<h1 align="center">GR-Raytracer</h1>
<h4 align="center">A dual CPU / GPU general relativity ray tracer written in Rust and WGSL.</h4> 

The ray-tracer features an integrator for the geodesic equation in schwarzschild and euclidean geometries. The integrator steps light rays along null geodesics and can use euler's method or runge kutta 4. The supported objects are discs and spheres with arbitrary image textures and diffuse or metallic surfaces.

# Usage
Make sure you have Cargo and the Rust compiler installed, then run 
```
cargo run --release
```
You can modify rendering parameters in `config.rs`, and change the scene in `main.rs`.

# A few words about the project

The pipeline was CPU only at first, until I re-wrote most of the code in WGSL. Unfortunately, using WGSL has not been a pleasant experience, mainly because of the lack of a proper module system and debugging tools. I also felt that the nature of manifolds better suits an object oriented language. I thus decided to bring back the CPU code, and write a dual pipeline. Now hopefully, I can add new features in rust without necessarly touching the GPU shaders, and update them only when I am sure of the implementation.

Note that for now the pipeline only supports time independant geometries.

# Notes on general relativity and implementation details

To represent spacetime, we start by describing the corresponding 4 dimensional manifold $\mathcal{M}$.
Firstly, $\mathcal{M}$ is assumed to be the set of points in $\mathbb{R}^4$, together with the usual topology. The manifold structure of $\mathcal{M}$ is given by the maximal atlas induced by the chart $\varphi = (\mathcal{M}$, $\mathbf{Id}: \mathcal{M} \to \mathbb{R}^4)$.

However, we cant describe points on an abstract manifold, so we pick $\varphi$ to be a distinguished global chart
and write transition maps from every other chart to this one. We call this chart `ChartWorld` in the code, it corresponds to cartesian coordinates centered on the camera. Up to identification by the distinguished map, this is the closest we can get to actually writing charts from $\mathcal{M}$ to $\mathbb{R}^4$. 

The first coordinate will corespond to time, and the other three to space. Because our pipeline only supports time independant geometries (for now), we can simplify the implementation of charts by writing them only for submanifolds of $\mathcal{M}$ obtained by fixing the time coordinate. We can then apply the identity map for the time coordinate.

Finally, we define the metric: a tensor field $g: \mathcal{M} \to T^{(0, 2)}\mathcal{M}$, where $g$ is symmetric and non-degenerate.
At a point $p$, the metric is: $g(p): T_p\mathcal{M} \times T_p\mathcal{M} \to \mathbb{R}$. Now given a chart $\varphi$, the coordinate vector fields $\partial_i$ form a basis of the tangent space, and we can define: $g_{\mu\nu}(p) = g(p)(\partial_\mu, \partial_\nu)$. From this, we find that:
$g(p) = g_{\mu\nu}(p) \; dx^\mu \otimes dx^\nu$.
In our case, we define a `g` method to the manifold trait that for each chart returns a matrix with entries $g_{\mu\nu}$ in that chart, and this suffices to represent the metric tensor field.


- Photon4 objects belong to R^4, and Photon3 objects will be photons at a point in time in world.

## Manifolds

### Point chart transitions:
$$p^{(\mathrm{to})} \;=\; \varphi_{(\mathrm{to})} \circ \varphi_{(\mathrm{from})}(p^{(\mathrm{from})})$$

Example: spherical to cartesian


### Vector chart transitions:

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
