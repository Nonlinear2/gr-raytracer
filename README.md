# black-hole-simulation
A simple black hole simulation in rust

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

### Code conventions
- The code studies the following manifolds:
R^4 as a semi riemannian manifold, which we will call "R^4"
3D submanifolds of R^4 obtained by chosing a time coodinate t, which we will call "R^3_t"
- charts will designate the maps from coordinates to manifolds and not the opposite.
- world will designate the manifold "R^3_t" together with the atlas containing a single chart: cartesian coordinates centered on the camera.
- bracket operators to access Point3, Point4 components are "unsafe", meaning there are no asserts to check r >= 0, theta in [0, pi]...
- photon objects will belong to R^4, and worldphoton objects will be photons at a point in time in world.
- we always write vector_space when talking about a vector space to avoid confusion with "space" meaning the ThreeVector representing space in a FourVector

### vector chart transitions:
$$v^{(\mathrm{to})} \;=\; \left. D\!\left(\varphi_{\mathrm{to}} \circ \varphi_{\mathrm{from}}^{-1}\right)\right|_{p}\; v^{(\mathrm{from})}$$
