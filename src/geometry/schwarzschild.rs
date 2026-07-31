use glam::{Mat4, Vec4};

use crate::config;
use crate::geometry::chart::{Cartesian, IsSphericalChart, SphericalX, SphericalZ, Transition};
use crate::geometry::manifold::{GpuManifold, PseudoRiemanian4Manifold};
use crate::geometry::metric::Metric;
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::ThreeVector;

/// the cartesian chart and the two spherical charts around the center
pub struct SphericalAtlas {
    pub center: Point3<Cartesian>,
}

impl SphericalAtlas {
    pub fn new(center: Point3<Cartesian>) -> Self {
        Self { center }
    }
}

impl Transition<Cartesian, SphericalZ> for SphericalAtlas {
    fn intersects(&self, p: Point3<Cartesian>) -> bool {
        let rel = p - self.center;
        rel.x() * rel.x() + rel.y() * rel.y() > 0.0
    }

    fn transition_point(&self, p: Point3<Cartesian>) -> Point3<SphericalZ> {
        debug_assert!(Transition::<Cartesian, SphericalZ>::intersects(self, p));
        let rel = p - self.center;

        let r = rel.distance_to_zero();
        let theta = (rel.z() / r).acos();
        let phi = rel.y().atan2(rel.x()).rem_euclid(std::f32::consts::TAU);

        Point3::new_spherical(r, theta, phi, SphericalZ)
    }

    fn transition_vector(&self, p: Point3<Cartesian>, v: ThreeVector<Cartesian>) -> ThreeVector<SphericalZ> {
        debug_assert!(Transition::<Cartesian, SphericalZ>::intersects(self, p));
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

        ThreeVector::new_spherical(
            dr_dx * v.x() + dr_dy * v.y() + dr_dz * v.z(),
            dth_dx * v.x() + dth_dy * v.y() + dth_dz * v.z(),
            dph_dx * v.x() + dph_dy * v.y(),
            SphericalZ,
        )
    }
}

impl Transition<SphericalZ, Cartesian> for SphericalAtlas {
    fn intersects(&self, _p: Point3<SphericalZ>) -> bool {
        true
    }

    fn transition_point(&self, p: Point3<SphericalZ>) -> Point3<Cartesian> {
        let x = p.r() * p.theta().sin() * p.phi().cos();
        let y = p.r() * p.theta().sin() * p.phi().sin();
        let z = p.r() * p.theta().cos();

        Point3::new(x, y, z, self.center.chart) + self.center
    }

    fn transition_vector(&self, p: Point3<SphericalZ>, v: ThreeVector<SphericalZ>) -> ThreeVector<Cartesian> {
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
            self.center.chart,
        )
    }
}

impl Transition<Cartesian, SphericalX> for SphericalAtlas {
    fn intersects(&self, p: Point3<Cartesian>) -> bool {
        let rel = p - self.center;
        rel.y() * rel.y() + rel.z() * rel.z() > 0.0
    }

    fn transition_point(&self, p: Point3<Cartesian>) -> Point3<SphericalX> {
        debug_assert!(Transition::<Cartesian, SphericalX>::intersects(self, p));
        let rel = p - self.center;

        let r = rel.distance_to_zero();
        let theta = (rel.x() / r).acos();
        let phi = rel.z().atan2(rel.y()).rem_euclid(std::f32::consts::TAU);

        Point3::new_spherical(r, theta, phi, SphericalX)
    }

    fn transition_vector(&self, p: Point3<Cartesian>, v: ThreeVector<Cartesian>) -> ThreeVector<SphericalX> {
        debug_assert!(Transition::<Cartesian, SphericalX>::intersects(self, p));
        let rel = p - self.center;
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

        ThreeVector::new_spherical(
            dr_dx * v.x() + dr_dy * v.y() + dr_dz * v.z(),
            dth_dx * v.x() + dth_dy * v.y() + dth_dz * v.z(),
            dph_dy * v.y() + dph_dz * v.z(),
            SphericalX,
        )
    }
}

impl Transition<SphericalX, Cartesian> for SphericalAtlas {
    fn intersects(&self, _p: Point3<SphericalX>) -> bool {
        true
    }

    fn transition_point(&self, p: Point3<SphericalX>) -> Point3<Cartesian> {
        let x = p.r() * p.theta().cos();
        let y = p.r() * p.theta().sin() * p.phi().cos();
        let z = p.r() * p.theta().sin() * p.phi().sin();

        Point3::new(x, y, z, self.center.chart) + self.center
    }

    fn transition_vector(&self, p: Point3<SphericalX>, v: ThreeVector<SphericalX>) -> ThreeVector<Cartesian> {
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
            self.center.chart,
        )
    }
}

// the two spherical charts only overlap through the cartesian chart they are built on

impl Transition<SphericalZ, SphericalX> for SphericalAtlas {
    fn intersects(&self, p: Point3<SphericalZ>) -> bool {
        let cart: Point3<Cartesian> = self.transition_point(p);
        Transition::<Cartesian, SphericalX>::intersects(self, cart)
    }

    fn transition_point(&self, p: Point3<SphericalZ>) -> Point3<SphericalX> {
        let cart: Point3<Cartesian> = self.transition_point(p);
        Transition::<Cartesian, SphericalX>::transition_point(self, cart)
    }

    fn transition_vector(&self, p: Point3<SphericalZ>, v: ThreeVector<SphericalZ>) -> ThreeVector<SphericalX> {
        let cart: Point3<Cartesian> = self.transition_point(p);
        let cart_v: ThreeVector<Cartesian> = self.transition_vector(p, v);
        Transition::<Cartesian, SphericalX>::transition_vector(self, cart, cart_v)
    }
}

impl Transition<SphericalX, SphericalZ> for SphericalAtlas {
    fn intersects(&self, p: Point3<SphericalX>) -> bool {
        let cart: Point3<Cartesian> = self.transition_point(p);
        Transition::<Cartesian, SphericalZ>::intersects(self, cart)
    }

    fn transition_point(&self, p: Point3<SphericalX>) -> Point3<SphericalZ> {
        let cart: Point3<Cartesian> = self.transition_point(p);
        Transition::<Cartesian, SphericalZ>::transition_point(self, cart)
    }

    fn transition_vector(&self, p: Point3<SphericalX>, v: ThreeVector<SphericalX>) -> ThreeVector<SphericalZ> {
        let cart: Point3<Cartesian> = self.transition_point(p);
        let cart_v: ThreeVector<Cartesian> = self.transition_vector(p, v);
        Transition::<Cartesian, SphericalZ>::transition_vector(self, cart, cart_v)
    }
}


pub struct Schwarzschild4Manifold {
    pub atlas: SphericalAtlas,
    pub r_s: f32,
}

impl Schwarzschild4Manifold {
    pub fn new(center: Point3<Cartesian>, r_s: f32) -> Self {
        Self {
            atlas: SphericalAtlas::new(center),
            r_s: r_s,
        }
    }
}

// the entries of g are the same regardless of if the spherical coordinates are centered on the X or Z axis,
// because the schwarzschild metric is spherically symmetric. Therefore we are generic over the chart.

impl<C: IsSphericalChart> Metric<C> for Schwarzschild4Manifold {

    fn g(&self, pos: Point4<C>) -> Mat4 {
        let r = pos.r();
        let theta = pos.theta();
        debug_assert!(r > self.r_s);

        let g = Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., -1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., -r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., -r*r*theta.sin()*theta.sin()),
        };

        debug_assert!(g.determinant().is_finite());

        g
    }

    fn g_inv(&self, pos: Point4<C>) -> Mat4 {
        let r = pos.r();
        let theta = pos.theta();
        debug_assert!(r > self.r_s);

        let g_inv = Mat4 {
            x_axis: Vec4::new(1. / (1. - self.r_s / r), 0., 0., 0.),
            y_axis: Vec4::new(0., self.r_s / r - 1., 0., 0.),
            z_axis: Vec4::new(0., 0., -1./(r*r), 0.),
            w_axis: Vec4::new(0., 0., 0., -1./(r*r*theta.sin()*theta.sin())),
        };

        g_inv
    }

    fn del_g(&self, pos: Point4<C>, i: u32) -> Mat4 {
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
}

impl<C: IsSphericalChart> PseudoRiemanian4Manifold<C> for Schwarzschild4Manifold {
    fn is_close_to_singular(&self, x: Point4<C>) -> bool {
        x.r() <= self.r_s + config::INTEGRATION_STEP_SIZE
    }
}

impl GpuManifold for Schwarzschild4Manifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("R_S", self.r_s as f64),
            ("SUBATLAS_CENTER_X", self.atlas.center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.atlas.center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.atlas.center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("schwarzschild.wgsl").to_string()
    }
}
