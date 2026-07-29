# Notes on general relativity and implementation details (work in progress)

This document compiles notes on differential geometry, general relativity and how they are implemented in this project.
Writing these notes helps me anchor what i've learned, and can help me recall it in the future. Be aware that there might be errors or misconceptions, I am still far from being at ease with these subjects.

## Manifolds

To represent spacetime, we start by describing the corresponding 4-dimensional
manifold $\mathcal{M}$. For the geometries we render, we make the modeling assumption that $\mathcal{M}$ admits a global chart 
$(\mathcal{M}, \varphi : \mathcal{M} \to U)$,
where $U$ is an open subset of $\mathbb{R}^4$ and $\varphi$ is a homeomorphism.
The manifold structure of $\mathcal{M}$ is then given by the maximal atlas induced by this chart. The
first coordinate is associated to time, and the other three to space.

Because our pipeline only supports time-independent geometries (for now), we can
simplify the manifold implementation by only describing three-dimensional
submanifolds $\mathcal{M}_t$ of $\mathcal{M}$ obtained by fixing the time
coordinate to a value $t$.

### Chart transitions

We can't describe points on an abstract manifold, so we pick $\varphi$ to be a
distinguished global chart and write transition maps from every other chart to
this one. We call this chart `ChartWorld` in the code. It corresponds to
Cartesian coordinates centered on the camera.

Let $p$ be a point in $\mathcal{M}_t$. If we call
$P^{(\mathrm{world})}, P^{(\mathrm{from})} \in \mathbb{R}^3$ the images of $p$ by
the charts $\varphi_{\mathrm{world}}, \varphi_{\mathrm{from}}$, then

$$
P^{(\mathrm{world})} \;=\; \varphi_{\mathrm{world}} \circ \varphi_{\mathrm{from}}^{-1}(P^{(\mathrm{from})})
$$

In the code, the map $\varphi_{\mathrm{world}} \circ \varphi_{\mathrm{from}}^{-1}$
is defined as `point_to_world`. Up to identification by the distinguished map,
this is the closest we can get to actually writing charts from $\mathcal{M}_t$ to
$\mathbb{R}^3$.

Similarly, given the transition maps, we can also write tangent vector
transitions. Let $p$ be a point in $\mathcal{M}_t$. We are looking for the
components of $v^{(\mathrm{to})}$ with respect to the basis
$\partial^{(\mathrm{to})}$ in terms of $v^{(\mathrm{from})}$ in the basis
$\partial^{(\mathrm{from})}$.

To do this, we can write the coordinate vector fields $\partial^{(\mathrm{from})}_p$
in terms of $\partial^{(\mathrm{to})}_p$. Let $f \in C^{\infty}(\mathcal{M})$. We have

$$
\begin{aligned}
\left(\partial^{(\mathrm{from})}_p(f)\right)_\mu
&= \frac{\partial}{\partial x^\mu}(f \circ \varphi_{\mathrm{from}}^{-1}) \big|_{\varphi_{\mathrm{from}}(p)}\\
&= \frac{\partial}{\partial x^\mu}(\underbrace{f \circ \varphi_{\mathrm{to}}^{-1}}_{\mathbb{R}^3 \to \mathbb{R}} \circ \underbrace{\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}}_{\mathbb{R}^3 \to \mathbb{R}^3})\big|_{\varphi_{\mathrm{from}}(p)}
\end{aligned}
$$

and

$$
D\left[(f \circ \varphi_{\mathrm{to}}^{-1}) \circ (\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1})\right] \big|_{\varphi_{\mathrm{from}}(p)}
= D\left[f \circ \varphi_{\mathrm{to}}^{-1}\right] \big|_{\varphi_{\mathrm{to}}(p)} \cdot D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}
$$

As

$$
\partial^{(\mathrm{from})}_p(f) = D\left[f \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}
\qquad\text{and}\qquad
\partial^{(\mathrm{to})}_p(f) = D\left[f \circ \varphi_{\mathrm{to}}^{-1}\right] \big|_{\varphi_{\mathrm{to}}(p)}
$$

we have

$$
\partial^{(\mathrm{from})}_p(f)
= \partial^{(\mathrm{to})}_p(f) \cdot D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right] \big|_{\varphi_{\mathrm{from}}(p)}
$$

Writing $J := D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}$
for the Jacobian, and noting the left-hand side is the scalar $v_p(f)$ (the same
number in either chart):

$$
\begin{aligned}
v_p(f)
&= (v^{(\mathrm{from})})^\mu\, (\partial^{(\mathrm{from})}_p(f))_\mu\\
&= (v^{(\mathrm{from})})^\mu \left(\partial^{(\mathrm{to})}_p(f) \cdot J\right)_\mu\\
&= (v^{(\mathrm{from})})^\mu\, (\partial^{(\mathrm{to})}_p(f))_\nu\, J^\nu{}_\mu\\
&= \left(J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu\right)\, (\partial^{(\mathrm{to})}_p(f))_\nu
\end{aligned}
$$

This holds for every $f \in C^{\infty}(\mathcal{M})$, so $v_p$ equals
$\left(J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu\right)\, (\partial^{(\mathrm{to})}_p)_\nu$
as a derivation. Since $\partial^{(\mathrm{to})}_p$ constitutes a basis, components
of $v_p$ are unique and we identify:

$$
\boxed{\,(v^{(\mathrm{to})})^\nu = J^\nu{}_\mu\, (v^{(\mathrm{from})})^\mu, \qquad J = D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}\,}
$$

or, as a matrix equation on column vectors,

$$
v^{(\mathrm{to})} = D\left[\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right]\big|_{\varphi_{\mathrm{from}}(p)}\; v^{(\mathrm{from})}.
$$

### Pseudo-Riemannian structure

Finally, we define the metric: a tensor field
$g: \mathcal{M} \to T^{(0, 2)}\mathcal{M}$, where $g$ is symmetric and
non-degenerate. At a point $p$, the metric is
$g(p): T_p\mathcal{M} \times T_p\mathcal{M} \to \mathbb{R}$. Given a chart
$\varphi$, the coordinate vector fields $\partial_i$ form a basis of the tangent
space, and we can define $g_{\mu\nu}(p) = g(p)(\partial_\mu, \partial_\nu)$. From
this we find that $g(p) = g_{\mu\nu}(p)\, dx^\mu \otimes dx^\nu$.

In our case, we define a `g` method on the manifold trait that, for each chart,
returns a matrix with entries $g_{\mu\nu}$ in that chart, and this suffices to
represent the metric tensor field.

## The geodesic equation

Do describe the trajectory of light in spacetime, we are looking to define what a straight line is on a manifold.

### Afine connections
Let's consider the case where the manifold is $\mathcal{M} = R^n$. We would like to define what a straight curve of constant speed is. A reasonable definition for that is: "a curve whose velocity vector never changes". More formally:

Let $\gamma: (0, 1) \to \mathcal{M}$ be a smooth curve. $\gamma$ is said to be straight if $\dot{\gamma}(t)$ is constant for all $t$, or equivalently, if $\ddot\gamma(t) = 0$.

We would like to generalize this definition to any smooth manifold $\mathcal{M}$. We know how to define $\dot\gamma(t)$:
$$
\begin{aligned}
\dot\gamma(t)f 
&= \frac{d}{ds} (f \circ \gamma)|_t\\
&= \left[\frac{d}{ds}(x^i \circ \gamma)|_t \; (\partial_{\gamma(t)})_i \right](f)
\end{aligned}
$$

Now, we need to make sense of $\ddot \gamma$. The issue we have is that if we try the usual definition:
$$
\ddot \gamma(t) = \lim_{h \to 0} \frac{1}{h}(\underbrace{\dot\gamma(t + h)}_{\in \, T_{\gamma(t + h)}\mathcal{M}} - \underbrace{\dot\gamma(t)}_{\in \, T_{\gamma(t)}\mathcal{M}})
$$
We run into the problem of adding vectors from _different_ vector spaces.
This motivates the concept of a connection.

For any smooth curve $\gamma: (0, 1) \to \mathcal{M}$ from $p$ to $q$, we would like to define a map $P_\gamma: T_p\mathcal{M} \to T_q\mathcal{M}$ that transports a vector from $T_p\mathcal{M}$ to $T_q\mathcal{M}$ "without changing its orientation", whatever that means.
But writing the correct definition is not so easy, and it is best to define parallel transport infinitesimally, we can define the map $P_\gamma$ from that. This is called an afine connection. It "connects" neighbouring tangent spaces together. Here is the formal definition:

Let $X, Y, Z \in \Gamma^\infty(T\mathcal{M})$ be vector fields on $\mathcal{M}$, and $f \in C^\infty(\mathcal{M})$ be a scalar function on $\mathcal{M}$. We say that $\nabla: \Gamma^\infty(T\mathcal{M}) \times \Gamma^\infty(T\mathcal{M}) \to \Gamma^\infty(T\mathcal{M})$ is an afine connection if
$$
\nabla \text{ is bilinear: } \nabla_{aY + bZ} X = a\nabla_Y X + a\nabla_Z X \text{ and } \nabla_{Y} (aX + bZ) = a\nabla_Y X + a\nabla_Y Z
$$
$$
\nabla \text{ is } C^\infty(\mathcal{M}) \text{ linear: } \nabla_{fY} X = f\nabla_{Y} X
$$

$$
\nabla \text{ follows the following Leibnitz rule: } \nabla_Y fX = Y(f) X + f\nabla_Y X 
$$

There can exist multiple connections, and in our case we are interested in the Levi-Civita connection.
[...]

### Parallel transport

Now that we have defined connections, for any smooth curve $\gamma: (0, 1) \to \mathcal{M}$ from $p$ to $q$, we can define the map $P_\gamma: T_p\mathcal{M} \to T_q\mathcal{M}$ we taked about earlier. First of all, we define
$$
P_\gamma(t): T_p\mathcal{M} \to T_{\gamma(t)}\mathcal{M}
$$
as the only function that verifies at each instant
$$
\nabla_{\dot\gamma}\dot \gamma = 0
$$
We can write this as a differential equation by expanding $\nabla$ with christoffel symbols.

### Christoffel symbols
In a given chart, Christoffel symbols are defined as follows
$$
(\nabla_{\partial_i} \partial_j)_P = \Gamma^{k}_{ij}(P) \partial_k (P)
$$
and thus:

$$
\begin{aligned}
\nabla_{\dot\gamma} \dot\gamma
&= \nabla_{\dot\gamma} \left[\frac{d\gamma^i}{ds} \; (\partial_{\gamma})_i \right]\\
&= \frac{d\gamma^i}{ds} \nabla_{\dot\gamma} (\partial_{\gamma})_i + \frac{d^2\gamma^i}{ds^2}(\partial_{\gamma})_i\\
&= \frac{d\gamma^i}{ds} \nabla_{\left[\frac{d\gamma^j}{ds} \; (\partial_{\gamma})_j \right]} (\partial_{\gamma})_i + \frac{d^2\gamma^i}{ds^2}(\partial_{\gamma})_i\\
&= \frac{d\gamma^i}{ds}\frac{d\gamma^j}{ds} \nabla_{(\partial_{\gamma})_j} (\partial_{\gamma})_i + \frac{d^2\gamma^i}{ds^2}(\partial_{\gamma})_i\\
&= \frac{d\gamma^i}{ds}\frac{d\gamma^j}{ds} \Gamma_{ji}^k (\partial_{\gamma})_k + \frac{d^2\gamma^i}{ds^2}(\partial_{\gamma})_i\\
&= \left[\frac{d\gamma^i}{ds}\frac{d\gamma^j}{ds} \Gamma_{ji}^k + \frac{d^2\gamma^k}{ds^2}\right](\partial_{\gamma})_k\\
\end{aligned}
$$
Equating with $0$ and using the linear independance of the $(\partial_\gamma)_k$, we find that for all $k$,
$$
\frac{d^2 \gamma^k}{ds^2} + \Gamma^k_{ji}\,\frac{d\gamma^j}{ds}\,\frac{d\gamma^i}{ds} = 0
$$

and this differential equation can be turned into a first-order system:

$$
X(s) =
\begin{pmatrix}
\gamma^\mu\\
\dot\gamma^\mu
\end{pmatrix}
\qquad
X'(s) =
\begin{pmatrix}
\dfrac{d\gamma^\mu}{ds}\\[6pt]
\dfrac{d\dot\gamma^\mu}{ds}
\end{pmatrix}
=
\begin{pmatrix}
\dot\gamma^\mu\\
-\Gamma^\mu_{\alpha\beta}\,\dot\gamma^\alpha\,\dot\gamma^\beta
\end{pmatrix}
$$

## Black hole physics

### Innermost stable circular orbit

$$
r_{\mathrm{ms}} = 3 R_s
$$

## My notes and todos:
- To define cartesian, polar coordinates as we do in physics, we define an abstract chart $\phi$, and call that one cartesian. Then we define $\phi_pol$ to verify $\phi \circ \phi_pol^-1 = (r \cos(\theta), r \sin(\theta))$. To define $\phi$ explicitely, we can use the canonical isomorphism $\mathbb{R} \cong T\mathbb{R}$. (but we could use another map, i think).

- The notation $df/dx$ ... + worked out example

- how turning maps into tensors works

- motivate the definition of a connexion

- derive the christoffel version geodesic equation

- acceleration in polar coordinates

- derivation of the schwarzschild metric

topological manifold
+ smooth structure (a maximal smooth atlas)
+ metric -> levi civita christoffels <-> connection -> parallel transport and geodesics
or + christoffels <-> connection -> geodesics

- The same manifold can have two atlases $A, A'$ which are incompatible, but (M, A) can still be diffeomorphic to $(M, A')$.
This is because compatible means $Id: (M, A) \to (M, A')$ is a diffeomorphism, whereas there can still exist a diffeomorphism $\Psi$ between $(M, A)$ and $(M, A')$.