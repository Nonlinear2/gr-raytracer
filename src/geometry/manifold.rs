use crate::geometry::chart::IsChart;
use crate::geometry::metric::Metric;
use crate::geometry::photon::{Photon4, PhotonDerivative};
use crate::geometry::point::Point4;
use crate::geometry::vector::FourVector;
use crate::integration;

pub trait PseudoRiemanian4Manifold<C: IsChart>: Metric<C> {

    fn is_close_to_singular(&self, x: Point4<C>) -> bool;

    /// this function computes the right hand side of the geodesic equation as described in the readme.
    fn geodesic_derivative(&self, photon: Photon4<C>) -> PhotonDerivative<C> {
        debug_assert!(!self.is_close_to_singular(photon.pos));

        let k = photon.vel;

        let mut del_k = FourVector::zero(k.chart);
        for mu in 0..4 {
            for alpha in 0..4 {
                for beta in 0..4 {
                    let gamma = self.christoffel(photon.pos, alpha, beta, mu);
                    del_k[mu] -= gamma * k[alpha] * k[beta];
                }
            }
        }

        PhotonDerivative { d_pos: k, d_vel: del_k }
    }

    fn step_along_null_geodesic(&self, photon: Photon4<C>) -> Photon4<C> {
        integration::step(photon, |p| self.geodesic_derivative(p))
    }
}

pub trait GpuManifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)>;
    fn get_geometry_source(&self) -> String;
}

pub trait Manifold<C: IsChart>: PseudoRiemanian4Manifold<C> + GpuManifold {}

impl<C: IsChart, T: PseudoRiemanian4Manifold<C> + GpuManifold> Manifold<C> for T {}
