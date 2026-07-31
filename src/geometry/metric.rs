use glam::Mat4;

use crate::geometry::chart::IsChart;
use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::point::Point4;
use crate::geometry::vector::FourVector;
use crate::math::positive_root;

/// the metric of a pseudo-riemannian manifold, read in the spacetime chart induced by C.
/// a geometry implements this once per chart of its atlas.
pub trait Metric<C: IsChart> {

    fn g(&self, x: Point4<C>) -> Mat4;

    fn g_inv(&self, x: Point4<C>) -> Mat4;

    fn del_g(&self, x: Point4<C>, i: u32) -> Mat4;

    // the methods below only depend on the metric, so they are shared between manifolds

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
