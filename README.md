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
- bracket operators to access Point3, Point4 components are "unsafe", meaning there are no asserts to check r >= 0, theta in [0, pi]...
- world means 3d space with no time component and cartesian coordinates centered on the camera.
- photon objects will belong to the manifold, not to world.
- we always write vector_space when talking about a vector space to avoid confusion with "space" meaning the ThreeVector representing space in a FourVector