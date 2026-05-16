use crate::graphics::vector::{FourVector, Point4};
use crate::graphics::surface::Material;
use crate::integration::solvers::positive_root;

use glam::{Vec3, Vec4, Mat4};


pub struct PhotonIntersection<'a> {
    pub point: Point4,
    pub normal: Vec3,
    pub material: &'a dyn Material
}


pub struct Photon {
    pub pos: Point4,
    pub vel: Vec4,
}

impl Photon {
    pub fn new(g: Mat4, pos: Vec4, vel: Vec3) -> Self {
        // g must be symmetric.

        // we need to find the fist velocity component in terms of vel to verify the normalization vel * g * vel = 0
        // we have:
        // g_00 (k^0)^2 + 2*g_i0 k^0 k^j + g_ij k^i k^j = 0

        let mut b = 0.;
        for i in 1..4 {
            b += g[0][i] * vel[i];
        }
        b *= 2.*vel[0];
    
        let mut c = 0.;
        for i in 1..4 {
            for j in 1..4 {
                b += g[i][j] * vel[i] * vel[j];
            }
        }

        let k_0 = positive_root(g[0][0], b, c);

        Self {
            pos: pos,
            vel: Vec4::from_space_time(k_0, vel),
        }

    }
    pub fn step(&self, step_size: f32) -> Photon {
        Photon {
            pos: self.pos + self.vel.normalize() * step_size,
            vel: self.vel,
        }
    }
}