use crate::graphics::{ray::Photon, vector::{FourVector, Point3, Point4}};
use glam::{Vec4, Mat4};

pub trait Metric {
    fn g(&self, x: Point4) -> Mat4;
    fn del_g(&self, x: Point4, i: u32) -> Mat4;
    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;
    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon;

    fn dot(&self, x: Point4, v1: Vec4, v2: Vec4) -> f32 {
        v1.dot(self.g(x) * v2)
    }

    fn norm(&self, x: Point4, v1: Vec4) -> f32 {
        self.dot(x, v1, v1)
    }
}

pub struct SchwartzschildMetric {
    pub center: Point3,
    pub r_s: f32,
}

impl SchwartzschildMetric {
    pub fn new(center: Point3, r_s: f32) -> Self {
        Self {
            center: center,
            r_s: r_s,
        }
    }

    pub fn mass(&self) -> f32 { // schwartzschild radius
        return self.r_s; // r_s = 2GM/c^2.
    }

    pub fn to_spherical_coordinates(&self, pos: Point3) -> Point3 {
        let length = (self.center - pos).length();
        Point3::new(
            length,
            pos.y.atan2(pos.x),
            if length > 0. { (pos.z / length).acos() } else { 0. },
        )
    }
}

impl Metric for SchwartzschildMetric {
    fn g(&self, pos: Point4) -> Mat4 {
        let r = pos[1];
        let theta = pos[2];
        Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., 1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., r*r*theta.sin()*theta.sin()),
        }
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        let sph_pos = self.to_spherical_coordinates(pos.space());

        let r = sph_pos[1];
        let theta = sph_pos[2];
        match i {
            0 => Mat4::ZERO,
            1 => Mat4 {
                x_axis: Vec4::new(self.r_s / (r*r), 0., 0., 0.),
                y_axis: Vec4::new(0., -self.r_s / (r*r*(1. - self.r_s / r) * (1. - self.r_s / r)), 0., 0.),
                z_axis: Vec4::new(0., 0., 2.*r, 0.),
                w_axis: Vec4::new(0., 0., 0., 2.*r*theta.sin()*theta.sin()),
            },
            2 => Mat4 {
                x_axis: Vec4::ZERO,
                y_axis: Vec4::ZERO,
                z_axis: Vec4::ZERO,
                w_axis: Vec4::new(0., 0., 0., 2.*r*r*theta.cos()*theta.sin()),
            },
            3 => Mat4::ZERO,
            _ => unreachable!()
        }
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        let sph_pos = self.to_spherical_coordinates(pos.space());

        let g_inv = self.g(Vec4::from_space_time(pos.time(), sph_pos)).inverse();
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
        gamma
    }

    fn step_along_null_geodesic(&self, photon: Photon, h: f32) -> Photon {
        Photon {
            pos: {
                let mut x_new = photon.pos;

                for mu in 0..4 {
                    x_new[mu] += h * photon.vel[mu];
                }

                x_new
            },

            vel: {
                let mut k_new = photon.vel;

                for mu in 0..4 {
                    let mut acc = 0.0;

                    for alpha in 0..4 {
                        for beta in 0..4 {
                            let gamma = self.christoffel(photon.pos, mu, alpha, beta);
                            acc += gamma * photon.vel[alpha] * photon.vel[beta];
                        }
                    }

                    k_new[mu] -= h * acc;
                }

                k_new
            },
        }
    }

}