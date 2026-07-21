<h1 align="center">GR-Raytracer</h1>
<h4 align="center">A dual CPU / GPU general relativity ray tracer written in Rust and WGSL.</h4> 

![Alt text](./assets/images/rendered_black_hole.png?raw=true)

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

## Manifolds

To represent spacetime, we start by describing the corresponding 4 dimensional manifold $\mathcal{M}$.
Firstly, $\mathcal{M}$ is assumed to be a topological space homeomorphic to $\mathbb{R}^4$ with the usual topology. The manifold structure of $\mathcal{M}$ is given by the maximal atlas induced by the chart ($\mathcal{M}$, $\varphi: \mathcal{M} \to \mathbb{R}^4$), where $\varphi$ is a homeomorphism from $\mathcal{M}$ to $\mathbb{R}^4$.
The first coordinate is associated to time, and the other three to space.
Because our pipeline only supports time independant geometries (for now), we can simplify the manifold implementation by only describing three dimensional submanifolds $\mathcal{M}_t$ of $\mathcal{M}$ obtained by fixing the time coordinate to a value $t$.

### Chart transitions:
Note that we can't describe points on an abstract manifold, so we pick $\varphi$ to be a distinguished global chart
and write transition maps from every other chart to this one. We call this chart `ChartWorld` in the code, it corresponds to cartesian coordinates centered on the camera:

Let $p$ be a point in $\mathcal{M}_t$.
If we call $P^{(\mathrm{world})}, P^{(\mathrm{from})} \in \mathbb{R}^3$ the images of $p$ by the charts $\varphi_{\mathrm{world}}, \varphi_{\mathrm{from}}$, then

$$P^{(\mathrm{world})} \;=\; \varphi_{\mathrm{world}} \circ \varphi_{\mathrm{from}}^{-1}(P^{(\mathrm{from})})$$

In the code, the map $\varphi_{\mathrm{world}} \circ \varphi_{\mathrm{from}}^{-1}$ is defined as `point_to_world`.

Up to identification by the distinguished map, this is the closest we can get to actually writing charts from $\mathcal{M}$ to $\mathbb{R}^4$.


Similarly, given the transition maps, we can also write tangent vector transitions:

Let $p$ be a point in $\mathcal{M}_t$.
We are looking for the components of $v^{(\mathrm{to})}$ with respect to the basis $\partial^{(\mathrm{to})}$ in terms of $v^{(\mathrm{from})}$ in the basis $\partial^{(\mathrm{from})}$.

To do this, we can write the coordinate vector fields $\partial^{(\mathrm{from})}_p$ in terms of $\partial^{(\mathrm{to})}_p$. Let $f \in C^{\infty}(\mathcal{M})$.
We have
$$\begin{align*}
\left(\partial^{(\mathrm{from})}_p(f)\right)_\mu
&= \frac{\partial}{\partial x^\mu}(f \circ \varphi_{\mathrm{from}}^{-1}) \big|_{\varphi_{\mathrm{from}}(p)}\\
&= \frac{\partial}{\partial x^\mu}(\underbrace{f \circ \varphi_{\mathrm{to}}^{-1}}_{\mathbb{R}^3 \to \mathbb{R}} \circ \underbrace{\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}}_{\mathbb{R}^3 \to \mathbb{R}^3})\big|_{\varphi_{\mathrm{from}}(p)}\\
\end{align*}$$
and 
$$D\left[(f \circ \varphi_{\mathrm{to}}^{-1}) \circ (\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1})\right] \big|_{\varphi_{\mathrm{from}}(p)}
= D\left[f \circ \varphi_{\mathrm{to}}^{-1}\right] \big|_{\varphi_{\mathrm{to}}(p)} \cdot D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}$$

As
$$\partial^{(\mathrm{from})}_p(f) = D\left[f \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}$$
and
$$\partial^{(\mathrm{to})}_p(f) = D\left[f \circ \varphi_{\mathrm{to}}^{-1}\right] \big|_{\varphi_{\mathrm{to}}(p)}$$
we have:
$$\partial^{(\mathrm{from})}_p(f)
= \partial^{(\mathrm{to})}_p(f) \cdot D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}$$


Writing $J := D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}$ for the Jacobian, and noting the
left hand side is the scalar $v_p(f)$ (the same number in either chart):
$$\begin{align*}
v_p(f)
&= (v^{(\mathrm{from})})^\mu\, (\partial^{(\mathrm{from})}_p(f))_\mu\\
&= (v^{(\mathrm{from})})^\mu \left(\partial^{(\mathrm{to})}_p(f) \cdot J\right)_\mu\\
&= (v^{(\mathrm{from})})^\mu\, (\partial^{(\mathrm{to})}_p(f))_\nu\, J^\nu{}_\mu\\
&= \left(J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu\right)\, (\partial^{(\mathrm{to})}_p(f))_\nu\\
\end{align*}$$

This holds for every $f \in C^{\infty}(\mathcal{M})$, so $v_p$ equals $\left(J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu\right)\, (\partial^{(\mathrm{to})}_p)_\nu$ as a
derivation. Since $\partial^{(\mathrm{to})}_p$ consitutes a basis, components of $v_p$ are unique and we identify:

$$\boxed{\,(v^{(\mathrm{to})})^\nu = J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu, \qquad J = D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}\,}$$

or, as a matrix equation on column vectors,

$$v^{(\mathrm{to})} = D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}\; v^{(\mathrm{from})}.$$


### Pseudo-Riemannian structure

Finally, we define the metric: a tensor field $g: \mathcal{M} \to T^{(0, 2)}\mathcal{M}$, where $g$ is symmetric and non-degenerate.
At a point $p$, the metric is: $g(p): T_p\mathcal{M} \times T_p\mathcal{M} \to \mathbb{R}$. Now given a chart $\varphi$, the coordinate vector fields $\partial_i$ form a basis of the tangent space, and we can define: $g_{\mu\nu}(p) = g(p)(\partial_\mu, \partial_\nu)$. From this, we find that:
$g(p) = g_{\mu\nu}(p) \; dx^\mu \otimes dx^\nu$.
In our case, we define a `g` method to the manifold trait that for each chart returns a matrix with entries $g_{\mu\nu}$ in that chart, and this suffices to represent the metric tensor field.

## The geodesic equation
We know that the trajectory of a photon in spacetime follows a null geodesic. 
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