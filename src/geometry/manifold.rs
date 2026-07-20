use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, TangentSpace, ThreeVector};
use crate::geometry::photon::{Photon4, Photon3, PhotonDerivative};
use crate::integration;

use glam::Mat4;
use num_enum::{TryFromPrimitive};

/// which chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
pub enum Chart {
    Cartesian = 1, // cartesian with center point
    SphericalZ = 2, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z
    SphericalX = 3, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X
}

/// the background cartesian chart of the fixed time submanifolds, in which the scene is described
/// it is tracked at the type level (Point3<ChartWorld>) rather than in the Chart enum
#[derive(Clone, Copy, PartialEq)]
pub struct ChartWorld;

/// basis of the tangent space corresponding to the ChartWorld chart,
/// tracked at the type level (ThreeVector<TangentWorld>) rather than in the TangentSpace enum
#[derive(Clone, Copy, PartialEq)]
pub struct TangentWorld;

pub fn tangent_space(chart: Chart) -> TangentSpace {
    match chart {
        Chart::Cartesian => TangentSpace::Cartesian,
        Chart::SphericalZ => TangentSpace::SphericalZ,
        Chart::SphericalX => TangentSpace::SphericalX,
    }
}

/// Atlas describing submanifolds M of spacetime given by fixing the time coordinate.
/// The World chart is always part of the atlas and is not listed in Chart,
/// because "coordinates through ChartWorld" will be the way to describe points of M.
pub trait HasAtlas3 {
    fn has_chart(&self, chart: Chart) -> bool;
    fn subatlas_center(&self) -> Point3<ChartWorld>;
    fn preferred_chart_for_point(&self, point: Point3<ChartWorld>) -> Chart;

    /// the open subset of M where the chart is defined (described through the world chart as usual)
    fn chart_domain_contains(&self, chart: Chart, p: Point3<ChartWorld>) -> bool {
        debug_assert!(self.has_chart(chart));
        let rel = p - self.subatlas_center();
        match chart {
            Chart::Cartesian => true,
            Chart::SphericalZ => rel.x() * rel.x() + rel.y() * rel.y() > 0.0,
            Chart::SphericalX => rel.y() * rel.y() + rel.z() * rel.z() > 0.0,
        }
    }

    // transition maps do not depend on the metric, so they are shared between manifolds
    fn point_to_world(&self, p: Point3) -> Point3<ChartWorld> {
        debug_assert!(self.has_chart(p.chart));
        match p.chart {
        Chart::Cartesian => {
            Point3::new(p.x(), p.y(), p.z(), ChartWorld) + self.subatlas_center()
        },
        Chart::SphericalZ => {
            let x = p.r() * p.theta().sin() * p.phi().cos();
            let y = p.r() * p.theta().sin() * p.phi().sin();
            let z = p.r() * p.theta().cos();

            Point3::new(x, y, z, ChartWorld) + self.subatlas_center()
        },
        Chart::SphericalX => {
            let x = p.r() * p.theta().cos();
            let y = p.r() * p.theta().sin() * p.phi().cos();
            let z = p.r() * p.theta().sin() * p.phi().sin();

            Point3::new(x, y, z, ChartWorld) + self.subatlas_center()
        },
        }
    }

    fn point_from_world(&self, p: Point3<ChartWorld>, to: Chart) -> Point3 {
        debug_assert!(self.has_chart(to));
        debug_assert!(self.chart_domain_contains(to, p));
        let p_rel = p - self.subatlas_center();
        match to {
        Chart::Cartesian => {
            Point3::new(p_rel.x(), p_rel.y(), p_rel.z(), Chart::Cartesian)
        },
        Chart::SphericalZ => {
            let r = p_rel.distance_to_zero();
            let theta = (p_rel.z() / r).acos();
            let phi = p_rel.y().atan2(p_rel.x()).rem_euclid(2.0 * std::f32::consts::PI);

            Point3::new_spherical_z(r, theta, phi)
        },
        Chart::SphericalX => {
            let r = p_rel.distance_to_zero();
            let theta = (p_rel.x() / r).acos();
            let phi = p_rel.z().atan2(p_rel.y()).rem_euclid(2.0 * std::f32::consts::PI);

            Point3::new_spherical_x(r, theta, phi)
        },
        }
    }

    fn vector_to_world(&self, p: Point3, v: ThreeVector) -> ThreeVector<TangentWorld> {
        debug_assert!(self.has_chart(p.chart));
        match p.chart {
        Chart::Cartesian => {
            ThreeVector::new(v.x(), v.y(), v.z(), TangentWorld)
        },
        Chart::SphericalZ => {
            let r = p.r();
            let theta = p.theta();
            let phi = p.phi();
            let v_r = v.r();
            let v_theta = v.theta();
            let v_phi = v.phi();

            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            ThreeVector::new(
                sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                cos_theta * v_r - r * sin_theta * v_theta,
                TangentWorld,
            )
        },
        Chart::SphericalX => {
            let r = p.r();
            let theta = p.theta();
            let phi = p.phi();
            let v_r = v.r();
            let v_theta = v.theta();
            let v_phi = v.phi();

            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            ThreeVector::new(
                cos_theta * v_r - r * sin_theta * v_theta,
                sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                TangentWorld,
            )
        },
        }
    }

    fn vector_from_world(&self, p: Point3<ChartWorld>, v: ThreeVector<TangentWorld>, to: Chart) -> ThreeVector {
        debug_assert!(self.has_chart(to));
        debug_assert!(self.chart_domain_contains(to, p));
        match to {
        Chart::Cartesian => {
            ThreeVector::new(v.x(), v.y(), v.z(), TangentSpace::Cartesian)
        },
        Chart::SphericalZ => {
            let rel = p - self.subatlas_center();
            let x = rel.x();
            let y = rel.y();
            let z = rel.z();
            let rho = (x * x + y * y).sqrt(); // distance to the z axis
            let r = (x * x + y * y + z * z).sqrt();

            let dr_dx = x / r;
            let dr_dy = y / r;
            let dr_dz = z / r;
            let dth_dx = x * z / (r * r * rho);
            let dth_dy = y * z / (r * r * rho);
            let dth_dz = -rho / (r * r);
            let dph_dx = -y / (rho * rho);
            let dph_dy = x / (rho * rho);

            ThreeVector::new(
                dr_dx * v.x() + dr_dy * v.y() + dr_dz * v.z(),
                dth_dx * v.x() + dth_dy * v.y() + dth_dz * v.z(),
                dph_dx * v.x() + dph_dy * v.y(),
                TangentSpace::SphericalZ
            )
        },
        Chart::SphericalX => {
            let rel = p - self.subatlas_center();
            let x = rel.x();
            let y = rel.y();
            let z = rel.z();
            let rho = (y * y + z * z).sqrt(); // distance to the x axis
            let r = (x * x + y * y + z * z).sqrt();

            let dr_dx = x / r;
            let dr_dy = y / r;
            let dr_dz = z / r;
            let dth_dx = -rho / (r * r);
            let dth_dy = x * y / (r * r * rho);
            let dth_dz = x * z / (r * r * rho);
            let dph_dy = -z / (rho * rho);
            let dph_dz = y / (rho * rho);

            ThreeVector::new(
                dr_dx * v.x() + dr_dy * v.y() + dr_dz * v.z(),
                dth_dx * v.x() + dth_dy * v.y() + dth_dz * v.z(),
                dph_dy * v.y() + dph_dz * v.z(),
                TangentSpace::SphericalX,
            )
        },
        }
    }
}

pub trait PseudoRiemanian4Manifold: HasAtlas3 {

    fn is_close_to_singular(&self, x: Point4) -> bool;

    fn world_photon3_to_photon4(&self, world_photon: Photon3) -> Photon4;

    fn to_world_photon3(&self, photon: Photon4) -> Photon3;

    fn g(&self, x: Point4) -> Mat4;

    fn g_inv(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    // the methods below only depend on the metric, so they are shared between manifolds

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
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

    /// this function computes the right hand side of the geodesic equation as described in the readme.
    fn geodesic_derivative(&self, photon: Photon4) -> PhotonDerivative {
        debug_assert!(!self.is_close_to_singular(photon.pos));

        let k = photon.vel;

        let mut del_k = FourVector::zero(k.vector_space);
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

    fn step_along_null_geodesic(&self, photon: Photon4) -> Photon4 {
        integration::step(photon, |p| self.geodesic_derivative(p))
    }
}

pub trait GpuManifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)>;
    fn get_geometry_source(&self) -> String;
}

pub trait Manifold: PseudoRiemanian4Manifold + GpuManifold {}

impl<T: PseudoRiemanian4Manifold + GpuManifold> Manifold for T {}
