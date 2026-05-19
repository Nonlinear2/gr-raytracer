use crate::graphics::ray::{Photon, WorldPhoton};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{TangentSpace, ThreeVector, FourVector};
use crate::integration::euler;

use crate::integration::solvers::positive_root;
use glam::{Vec4, Mat4};

const SPH_EPS: f32 = 1e-3;

/// which global chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
/// Important points: 
/// These charts will designate the maps from coordinates to "manifold" and not the opposite. They are technically inverse charts
#[derive(Clone, Copy, PartialEq)]
pub enum Chart {
    CartesianWorld,
    Cartesian, // cartesian with center point
    SphericalZ, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z 
    SphericalX, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X 
}

// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted).
pub trait HasAtlas3 {
    fn has_chart(&self, chart: Chart) -> bool; // should always have CartesianWorld
    fn preferred_chart_for_point(&self, point: Point3) -> Chart;
    fn transition_point(&self, from: Chart, to: Chart, p: Point3) -> Point3;
    fn transition_vector(&self, from: Chart, to: Chart, p: Point3, v: ThreeVector) -> ThreeVector;
}

#[allow(dead_code)]
pub struct EuclideanAtlas3 {
    pub center: Point3,
}

pub struct SchwarzschildAtlas3 {
    pub center: Point3,
}

impl HasAtlas3 for EuclideanAtlas3 {
    fn has_chart(&self, chart: Chart) -> bool {
        chart == Chart::Cartesian || chart == Chart::CartesianWorld
    }

    fn preferred_chart_for_point(&self, _point: Point3) -> Chart {
        Chart::Cartesian
    }

    fn transition_point(&self, from: Chart, to: Chart, pos: Point3) -> Point3 {
        match (from, to) {
            (Chart::Cartesian, Chart::CartesianWorld) => pos - self.center,
            (Chart::CartesianWorld, Chart::Cartesian) => pos + self.center,
            _ => panic!()
        }
    }

    fn transition_vector(&self, _from: Chart, _to: Chart, _p: Point3, v: ThreeVector) -> ThreeVector {
        v
    }
}

impl HasAtlas3 for SchwarzschildAtlas3 {
    fn has_chart(&self, chart: Chart) -> bool {
        chart == Chart::SphericalX || chart == Chart::SphericalZ || chart == Chart::CartesianWorld
    }

    fn preferred_chart_for_point(&self, _point: Point3) -> Chart {
        todo!()
    }

    fn transition_point(&self, from: Chart, to: Chart, p: Point3) -> Point3 {
        match (from, to) {
        (Chart::CartesianWorld, Chart::SphericalZ) => {
            (p - self.center).to_spherical()
        },
        (Chart::CartesianWorld, Chart::SphericalX) => {
            todo!()
        },

        (Chart::SphericalZ, Chart::CartesianWorld) => {
            p.to_cartesian() + self.center
        },
        (Chart::SphericalX, Chart::CartesianWorld) => {
            todo!()
        },

        (Chart::SphericalX, Chart::SphericalZ) => {
            todo!()
        },
        (Chart::SphericalZ, Chart::SphericalX) => {
            todo!()
        },
        _ => panic!()
        }
    }

    fn transition_vector(&self, from: Chart, to: Chart, p: Point3, v: ThreeVector) -> ThreeVector {
        assert!(p.chart == from); // p must already be in the initial chart (see readme for explanations)

        match (from, to) {
        (Chart::CartesianWorld, Chart::SphericalZ) => {
            // TODO: add guards against trying to convert to singular points on SphericalZ 
            //     // if the point is close to the z axis, spherical coordinates become singular, and v_phi becomes unphysical.
            //     // we set it to 0.0 arbitrairly.
            // let (v_r, v_th, v_ph) = if rho <= SPH_EPS {
            //     let pole_sign = if z >= 0.0 { 1.0 } else { -1.0 };
            //     let tangential = (vel.x() * vel.x() + vel.y() * vel.y()).sqrt();
            //     (
            //         pole_sign * vel.z(),
            //         tangential / r,
            //         0.0,
            //     )
            // } else {
            let rel = p - self.center;
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

        (Chart::CartesianWorld, Chart::SphericalX) => {
            todo!()
        },

        (Chart::SphericalZ, Chart::CartesianWorld) => {
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

            ThreeVector::new_cartesian(
                sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                cos_theta * v_r - r * sin_theta * v_theta,
            )
        },

        (Chart::SphericalX, Chart::CartesianWorld) => {
            todo!()
        },

        (Chart::SphericalX, Chart::SphericalZ) => {
            todo!()
        },
        (Chart::SphericalZ, Chart::SphericalX) => {
            todo!()
        },
        _ => panic!()
        }
    }
}

pub trait PseudoRiemanian4Manifold {

    fn is_singular(&self, x: Point4) -> bool;

    fn world_to_photon(&self, world_photon: WorldPhoton) -> Photon;

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton;

    fn g(&self, x: Point4) -> Mat4;

    fn g_inv(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon) -> Photon;
}

pub struct Euclidean4Manifold {
    pub sub_atlas: EuclideanAtlas3 // atlas for fixed-time submanifolds
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {

    fn is_singular(&self, _x: Point4) -> bool {
        false
    }

    fn world_to_photon(&self, world_photon: WorldPhoton) -> Photon {
        Photon::new(
            Point4::from_space_time(0., world_photon.pos),
            FourVector::from_space_time(0., world_photon.vel)
        )
    }

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton {
        assert!(photon.pos.chart == Chart::Cartesian);
        assert!(photon.vel.vector_space == TangentSpace::Cartesian);

        WorldPhoton::new(
            self.sub_atlas.transition_point(
                Chart::Cartesian,
                Chart::CartesianWorld,
                photon.pos.space()
            ),
            self.sub_atlas.transition_vector(
                Chart::Cartesian,
                Chart::CartesianWorld,
                Point3::ZERO_CART, // unused
                photon.vel.space()
            )
        )
    }

    fn g(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn g_inv(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn del_g(&self, _x: Point4, _i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, _pos: Point4, _mu: usize, _nu: usize, _lambda: usize) -> f32 {
        0.
    }

    fn step_along_null_geodesic(&self, s: Photon) -> Photon {
        let (x_new, k_new) = euler::euler_step(
            s.pos, s.vel, s.vel.as_point4(), FourVector::ZERO_CART
        );
        Photon::new(x_new, k_new)
    }
}

pub struct Schwarzschild4Manifold {
    pub sub_atlas: SchwarzschildAtlas3, // atlas for fixed-time submanifolds
    pub r_s: f32,
}

impl Schwarzschild4Manifold {
    // center is a Point in world space
    pub fn new(center: Point3, r_s: f32) -> Self {
        assert!(center.chart == Chart::Cartesian);
        Self {
            sub_atlas: SchwarzschildAtlas3 { center: center },
            r_s: r_s,
        }
    }
}

impl PseudoRiemanian4Manifold for Schwarzschild4Manifold {
    fn is_singular(&self, x: Point4) -> bool {
        x.r() <= self.r_s
    }

    /// x is a point in world
    /// vel is a vector in the tangent space of world
    fn world_to_photon(&self, x: Point3, vel: ThreeVector) -> Photon {
        assert!(x.chart == Chart::Cartesian);
        assert!(vel.vector_space == TangentSpace::Cartesian);
        assert!((x - self.center).distance_to_zero() > SPH_EPS);

        let rel = x - self.center;
        let x = rel.x();
        let y = rel.y();
        let z = rel.z();
        let rho = (x * x + y * y).sqrt(); // distance to the z axis

        let pos = if rho <= SPH_EPS {
            // keep pos tangent vectors aligned with velocity
            rel.to_spherical_on_z_axis(vel.y().atan2(vel.x()).rem_euclid(std::f32::consts::TAU))
        } else {
            rel.to_spherical()
        };

        let r = pos.r();

        // if the photon is close to the z axis, spherical coordinates become singular, and v_phi becomes unphysical.
        // we set it to 0.0 arbitrairly.
        let (v_r, v_th, v_ph) = if rho <= SPH_EPS {
            let pole_sign = if z >= 0.0 { 1.0 } else { -1.0 };
            let tangential = (vel.x() * vel.x() + vel.y() * vel.y()).sqrt();
            (
                pole_sign * vel.z(),
                tangential / r,
                0.0,
            )
        } else {
            let dr_dx = x / r;
            let dr_dy = y / r;
            let dr_dz = z / r;
            let dth_dx = x * z / (r * r * rho);
            let dth_dy = y * z / (r * r * rho);
            let dth_dz = -rho / (r * r);
            let dph_dx = -y / (rho * rho);
            let dph_dy = x / (rho * rho);
            (
                dr_dx * vel.x() + dr_dy * vel.y() + dr_dz * vel.z(),
                dth_dx * vel.x() + dth_dy * vel.y() + dth_dz * vel.z(),
                dph_dx * vel.x() + dph_dy * vel.y(),
            )
        };

        let vel_sph = ThreeVector::new(v_r, v_th, v_ph, TangentSpace::Spherical);

        let photon_x= Point4::from_space_time(0.0, pos);

        // compute k^0 such that <k, k> = 0 so that the photon's trajectory be lightlike.
        // we need to solve g_mu_nu k^mu k^nu = 0 for k^0 which is a quadratic equation

        let g = self.g(photon_x);
        let b = 2.0 * (g.col(0)[1] * v_r + g.col(0)[2] * v_th + g.col(0)[3] * v_ph);
        let c = 
              g.col(1)[1] * v_r * v_r
            + 2.0 * g.col(1)[2] * v_r * v_th
            + 2.0 * g.col(1)[3] * v_r * v_ph
            + g.col(2)[2] * v_th * v_th
            + 2.0 * g.col(2)[3] * v_th * v_ph
            + g.col(3)[3] * v_ph * v_ph;

        let k_0 = positive_root(g.col(0)[0], b, c);

        Photon::new(photon_x, FourVector::from_space_time(k_0, vel_sph))
    }        

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton {
        WorldPhoton::new(
            self.photon_to_world_pos(photon),
            self.photon_to_world_vel(photon),
        )
    }

    fn g(&self, pos: Point4) -> Mat4 {
        assert!(pos.chart == Chart::Spherical);

        let r = pos.r();
        let theta = pos.theta();
        assert!(r > self.r_s);

        let g = Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., -1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., -r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., -r*r*theta.sin()*theta.sin()),
        };
        
        if !g.determinant().is_finite() {
            eprintln!("[g_sph] Degenerate metric: r={}, theta={}, det={}", r, theta, g.determinant());
        }
        
        g
    }

    fn g_inv(&self, pos: Point4) -> Mat4 {
        assert!(pos.chart == Chart::Spherical);
        assert!(pos.r() > self.r_s);
        assert!(pos.theta() >= SPH_EPS);
        assert!(pos.theta() <= std::f32::consts::PI - SPH_EPS);

        let r = pos.r();
        let theta = pos.theta();
        assert!(r > self.r_s);

        let g_inv = Mat4 {
            x_axis: Vec4::new(1. / (1. - self.r_s / r), 0., 0., 0.),
            y_axis: Vec4::new(0., self.r_s / r - 1., 0., 0.),
            z_axis: Vec4::new(0., 0., -1./(r*r), 0.),
            w_axis: Vec4::new(0., 0., 0., -1./(r*r*theta.sin()*theta.sin())),
        };

        g_inv
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        assert!(pos.chart == Chart::Spherical);

        let r = pos.r();
        let theta = pos.theta();
        match i {
            0 => Mat4::ZERO,
            1 => Mat4 {
                x_axis: Vec4::new(self.r_s / (r*r), 0., 0., 0.),
                y_axis: Vec4::new(0., self.r_s / (r*r*(1. - self.r_s / r) * (1. - self.r_s / r)), 0., 0.),
                z_axis: Vec4::new(0., 0., -2.*r, 0.),
                w_axis: Vec4::new(0., 0., 0., -2.*r*theta.sin()*theta.sin()),
            },
            2 => Mat4 {
                x_axis: Vec4::ZERO,
                y_axis: Vec4::ZERO,
                z_axis: Vec4::ZERO,
                w_axis: Vec4::new(0., 0., 0., -2.*r*r*theta.cos()*theta.sin()),
            },
            3 => Mat4::ZERO,
            _ => unreachable!()
        }
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        assert!(pos.chart == Chart::Spherical);

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
        if !gamma.is_finite() {
            eprintln!("[christoffel] NaN detected: mu={}, nu={}, lambda={}, pos=({},{},{}), gamma={}", mu, nu, lambda, pos.r(), pos.theta(), pos.phi(), gamma);
            eprintln!("[christoffel] g_inv determinant={:?}", g_inv.determinant());
        }
        gamma
    }

    fn step_along_null_geodesic(&self, photon: Photon) -> Photon {
        let mut x = photon.pos;

        // avoid coordinate chart singularity for theta = 0 or theta = pi      
        if x.theta() < SPH_EPS || x.theta() > std::f32::consts::PI - SPH_EPS {
            let theta_adj = if x.theta() <= SPH_EPS { SPH_EPS } else { std::f32::consts::PI - SPH_EPS };
            // eprintln!("[christoffel] perturbing theta from {} to {} to avoid pole", theta, theta_adj);
            x = Point4::new_spherical(x.t(), x.r(), theta_adj, x.phi());
        }

        let k = photon.vel;

        let mut del_k = FourVector::ZERO_SPH;
        for mu in 0..4 {
            for alpha in 0..4 {
                for beta in 0..4 {
                    let gamma = self.christoffel(x, alpha, beta, mu);
                    del_k[mu] -= gamma * k[alpha] * k[beta];
                }
            }
        }

        let (new_x, new_k) = euler::euler_step(x, k, k.as_point4(), del_k);

        Photon {
            pos: new_x,
            vel: new_k,
        }
    }
}