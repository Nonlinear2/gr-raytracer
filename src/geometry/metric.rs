use glam::{Mat3, Mat4, Vec3};

use crate::geometry::chart::IsChart;
use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::point::Point4;
use crate::geometry::vector::{FourVector, ThreeVector};
use crate::math::positive_root;

/// the metric of a pseudo-riemannian manifold, read in the spacetime chart induced by C.
/// a geometry implements this once per chart of its atlas.
pub trait Metric<C: IsChart> {

    fn g(&self, p: Point4<C>) -> Mat4;

    fn g_inv(&self, p: Point4<C>) -> Mat4;

    fn del_g(&self, p: Point4<C>, i: u32) -> Mat4;

    // the methods below only depend on the metric, so they are shared between manifolds

    fn spatial_g(&self, p: Point4<C>) -> Mat3 {
        let g = self.g(p);
        // we negate because the metric signature is +--- and we want a positive definite matrix
        Mat3::from_cols(
            Vec3::new(-g.col(1)[1], -g.col(1)[2], -g.col(1)[3]),
            Vec3::new(-g.col(2)[1], -g.col(2)[2], -g.col(2)[3]),
            Vec3::new(-g.col(3)[1], -g.col(3)[2], -g.col(3)[3]),
        )
    }

    fn spatial_dot(&self, p: Point4<C>, u: ThreeVector<C>, v: ThreeVector<C>) -> f32 {
        let mut out = 0f32;
        let g = self.spatial_g(p);
        for alpha in 0..3 {
            for beta in 0..3 {
                out += u[alpha] * v[beta] * g.col(alpha)[beta];
            }
        }
        out
    }

    fn spatial_norm(&self, p: Point4<C>, u: ThreeVector<C>) -> f32 {
        self.spatial_dot(p, u, u).sqrt()
    }

    fn spatial_normalize(&self, p: Point4<C>, u: ThreeVector<C>) -> ThreeVector<C> {
        u * (1.0 / self.spatial_norm(p, u))
    }

    /// uses the Gram-Schmidt procedure on the three tangent space basis vectors: \partial_1, \partial_2, \partial_3
    /// using the metric for dot products. Only \partial_1 keeps its direction.
    fn orthonormal_frame(&self, p: Point4<C>) -> (ThreeVector<C>, ThreeVector<C>, ThreeVector<C>) {

        let projection = 
            |x, e| self.spatial_dot(p, e, x) * e;

        let del_1 = ThreeVector::new(1.0, 0.0, 0.0, p.chart);
        let del_2 = ThreeVector::new(0.0, 1.0, 0.0, p.chart);
        let del_3 = ThreeVector::new(0.0, 0.0, 1.0, p.chart);

        let out_1 = self.spatial_normalize(p, del_1);
        let out_2 = self.spatial_normalize(p, del_2 - projection(del_2, out_1));
        let out_3 = self.spatial_normalize(p, del_3 - projection(del_3, out_1) - projection(del_3, out_2));

        (out_1, out_2, out_3)
        // R = [[q.dot_product(cols[j]) if j >= i else 0 for j in range(n)] for i, q in enumerate(Q)]
    }

    fn christoffel(&self, pos: Point4<C>, mu: usize, nu: usize, lambda: usize) -> f32 {
        let g_inv = self.g_inv(pos);
        let mut gamma = 0.;

        let d_mu_g = self.del_g(pos, mu as u32);
        let d_nu_g = self.del_g(pos, nu as u32);

        for alpha in 0..4 {
            let d_alpha_g = self.del_g(pos, alpha as u32);

            gamma += 0.5 * g_inv.col(lambda)[alpha] * (
                d_mu_g.col(alpha)[nu]
              + d_nu_g.col(alpha)[mu]
              - d_alpha_g.col(mu)[nu]
            )
        }
        debug_assert!(gamma.is_finite());

        gamma
    }

    /// lifts a photon of the fixed-time submanifold M_t to spacetime by computing k^0
    /// such that <k, k> = 0, so that the photon's trajectory be lightlike.
    /// we need to solve g_mu_nu k^mu k^nu = 0 for k^0 which is a quadratic equation.
    fn to_photon4(&self, photon: Photon3<C>) -> Photon4<C> {
        let pos = Point4::from_space_time(0.0, photon.pos);
        let vel = photon.vel;
        let g = self.g(pos);

        let mut b = 0.0;
        let mut c = 0.0;
        for i in 1..4 {
            b += 2.0 * g.col(0)[i] * vel[i - 1];
            for j in 1..4 {
                c += g.col(i)[j] * vel[i - 1] * vel[j - 1];
            }
        }

        let k_0 = positive_root(g.col(0)[0], b, c);

        Photon4::new(pos, FourVector::from_space_time(k_0, vel))
    }
}
