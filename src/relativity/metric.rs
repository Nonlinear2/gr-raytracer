use crate::graphics::vector::Point3;
use glam::{Vec4, Mat4};

pub type Point4 = Vec4;

// Point4: ct, r, theta, phi

pub struct State {
    pub x: Vec4, // position
    pub k: Vec4, // tangent vector
}

pub trait Metric {
    fn g(&self, x: Point4) -> Mat4;
    fn del_g(&self, x: Point4, i: u32) -> Mat4;
    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;
    fn step_along_geodesic(&self, s: State, h: f32) -> State;

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
        let g = self.g(pos).inverse().to_cols_array_2d();
        let mut gamma = 0.;

        let d_mu_g = self.del_g(pos, mu as u32).to_cols_array_2d();
        let d_nu_g = self.del_g(pos, nu as u32).to_cols_array_2d();

        for alpha in 0..4 {
            let d_alpha_g = self.del_g(pos, alpha as u32).to_cols_array_2d();

            gamma += 0.5 * g[lambda][alpha] * (
                d_mu_g[alpha][nu]
              + d_nu_g[alpha][mu]
              - d_alpha_g[mu][nu]
            )
        }
        gamma
    }

    fn step_along_geodesic(&self, s: State, h: f32) -> State {
        State {
            x: {
                let mut x_new = s.x;

                for mu in 0..4 {
                    x_new[mu] += h * s.k[mu];
                }

                x_new
            },

            k: {
                let mut k_new = s.k;

                for mu in 0..4 {
                    let mut acc = 0.0;

                    for alpha in 0..4 {
                        for beta in 0..4 {
                            let gamma = self.christoffel(s.x, mu, alpha, beta);
                            acc += gamma * s.k[alpha] * s.k[beta];
                        }
                    }

                    k_new[mu] -= h * acc;
                }

                k_new
            },
        }
    }

}