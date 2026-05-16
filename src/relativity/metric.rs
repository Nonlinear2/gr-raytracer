use crate::graphics::{ray::Photon, vector::{Point3, Point4}};
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
    pub mass: f32,
}

impl SchwartzschildMetric {
    fn r_s(&self) -> f32 { // schwartzschild radius
        return self.mass; // r_s = 2GM/c^2.
    }
}

impl Metric for SchwartzschildMetric {
    fn g(&self, pos: Point4) -> Mat4 {
        let r = pos[1];
        let theta = pos[2];
        Mat4 {
            x_axis: Vec4::new(1. - self.r_s() / r, 0., 0., 0.),
            y_axis: Vec4::new(0., 1./(1. - self.r_s() / r), 0., 0.),
            z_axis: Vec4::new(0., 0., r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., r*r*theta.sin()*theta.sin()),
        }
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        let r = pos[1];
        let theta = pos[2];
        match i {
            0 => Mat4::ZERO,
            1 => Mat4 {
                x_axis: Vec4::new(self.r_s() / (r*r), 0., 0., 0.),
                y_axis: Vec4::new(0., -self.r_s() / (r*r*(1. - self.r_s() / r) * (1. - self.r_s() / r)), 0., 0.),
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
        let g_inv = self.g(pos).inverse();
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

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon {
        Photon {
            pos: {
                let mut x_new = s.pos;

                for mu in 0..4 {
                    x_new[mu] += h * s.vel[mu];
                }

                x_new
            },

            vel: {
                let mut k_new = s.vel;

                for mu in 0..4 {
                    let mut acc = 0.0;

                    for alpha in 0..4 {
                        for beta in 0..4 {
                            let gamma = self.christoffel(s.pos, mu, alpha, beta);
                            acc += gamma * s.vel[alpha] * s.vel[beta];
                        }
                    }

                    k_new[mu] -= h * acc;
                }

                k_new
            },
        }
    }

}