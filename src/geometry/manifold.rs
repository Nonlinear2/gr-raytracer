use crate::graphics::ray::{Photon, WorldPhoton};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{TangentSpace, ThreeVector, FourVector};
use crate::integration::euler;

use crate::integration::solvers::positive_root;
use glam::{Vec4, Mat4};

const SPH_EPS: f32 = 1e-3;
const EPS: f32 = 1e-5;

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
    #[allow(dead_code)]
    fn has_chart(&self, chart: Chart) -> bool; // should always have CartesianWorld
    fn preferred_chart_for_point(&self, point: Point3) -> Chart;
    fn transition_point(&self, from: Chart, to: Chart, p: Point3) -> Point3;
    fn transition_vector(&self, from: Chart, to: Chart, p: Point3, v: ThreeVector) -> ThreeVector;
}

#[allow(dead_code)]
pub struct EuclideanAtlas3 {
    pub center: Point3, // expressed in Chart::CartesianWorld
}

pub struct SchwarzschildAtlas3 {
    pub center: Point3, // expressed in Chart::CartesianWorld
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

    fn preferred_chart_for_point(&self, point: Point3) -> Chart {
        assert!(point.chart == Chart::CartesianWorld);

        let rel = point - self.center;

        let dist_to_z_axis_sq = rel.x() * rel.x() + rel.y() * rel.y();
        let dist_to_x_axis_sq = rel.y() * rel.y() + rel.z() * rel.z();

        if dist_to_z_axis_sq < dist_to_x_axis_sq {
            Chart::SphericalX
        } else {
            Chart::SphericalZ
        }
    }

    fn transition_point(&self, from: Chart, to: Chart, p: Point3) -> Point3 {
        match (from, to) {
        (Chart::CartesianWorld, Chart::SphericalZ) => {
            let p_rel = p - self.center;

            let r = p_rel.distance_to_zero();
            let theta = (p_rel.z() / r).acos();
            let phi = p_rel.y().atan2(p_rel.x()).rem_euclid(2.0 * std::f32::consts::PI);

            Point3::new_spherical_z(r, theta, phi)
        }

        (Chart::CartesianWorld, Chart::SphericalX) => {
            let p_rel = p - self.center;

            let r = p_rel.distance_to_zero();
            let theta = (p_rel.x() / r).acos();
            let phi = p_rel.z().atan2(p_rel.y()).rem_euclid(2.0 * std::f32::consts::PI);

            Point3::new_spherical_x(r, theta, phi)
        },

        (Chart::SphericalZ, Chart::CartesianWorld) => {

            let x = p.r() * p.theta().sin() * p.phi().cos();
            let y = p.r() * p.theta().sin() * p.phi().sin();
            let z = p.r() * p.theta().cos();

            Point3::new_cartesian(x, y, z) + self.center
        },
        (Chart::SphericalX, Chart::CartesianWorld) => {
            let x = p.r() * p.theta().cos();
            let y = p.r() * p.theta().sin() * p.phi().cos();
            let z = p.r() * p.theta().sin() * p.phi().sin();

            Point3::new_cartesian(x, y, z) + self.center
        },

        (Chart::SphericalX, Chart::SphericalZ) => {
            let p_world = self.transition_point(Chart::SphericalX, Chart::CartesianWorld, p);
            self.transition_point(Chart::CartesianWorld, Chart::SphericalZ, p_world)
        },
        (Chart::SphericalZ, Chart::SphericalX) => {
            let p_world = self.transition_point(Chart::SphericalZ, Chart::CartesianWorld, p);
            self.transition_point(Chart::CartesianWorld, Chart::SphericalX, p_world)
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

#[allow(dead_code)]
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
            s.pos, s.vel, s.vel.as_point4(), FourVector::zero(TangentSpace::Cartesian)
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
    fn world_to_photon(&self, world_photon: WorldPhoton) -> Photon {
        assert!(world_photon.pos.chart == Chart::CartesianWorld);
        assert!(world_photon.vel.vector_space == TangentSpace::Cartesian);
        assert!((world_photon.pos - self.sub_atlas.center).distance_to_zero() > EPS);

        let chart = self.sub_atlas.preferred_chart_for_point(world_photon.pos);

        let pos = self.sub_atlas.transition_point(
            Chart::CartesianWorld,
            chart,
            world_photon.pos
        );

        let vel = self.sub_atlas.transition_vector(
            Chart::CartesianWorld,
            chart,
            world_photon.pos,
            world_photon.vel
        );

        let (v_r, v_th, v_ph) = (vel.r(), vel.theta(), vel.phi());

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

        Photon::new(photon_x, FourVector::from_space_time(k_0, vel))
    }        

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton {
        WorldPhoton::new(
            self.sub_atlas.transition_point(
                photon.pos.chart,
                Chart::CartesianWorld,
                photon.pos.space()
            ),
            self.sub_atlas.transition_vector(
                photon.pos.chart,
                Chart::CartesianWorld,
                photon.pos.space(),
                photon.vel.space()
            ),
        )
    }

    fn g(&self, pos: Point4) -> Mat4 {
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

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
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));
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
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

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
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

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

        // check if we need to switch charts
        let photon = if self.sub_atlas.preferred_chart_for_point(photon.pos.space()) != photon.pos.chart {
            // change photon chart
            let world_photon = self.photon_to_world(photon);
            self.world_to_photon(world_photon)
        } else {
            photon
        };

        let x = photon.pos;
        let k = photon.vel;

        let mut del_k = FourVector::zero(photon.vel.vector_space);
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