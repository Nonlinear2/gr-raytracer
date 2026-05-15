# black-hole-simulation
A simple black hole simulation in rust

geodesic equation:
$$\frac{d^2 x^\mu}{d\lambda^2} + \Gamma^\mu_{\nu\rho}\,\frac{dx^\nu}{d\lambda}\,\frac{dx^\rho}{d\lambda} = 0$$

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
-\Gamma^\mu_{\nu\rho}\,k^\nu\,k^\rho\\
\end{pmatrix}
$$