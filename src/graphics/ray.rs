use crate::graphics::vector::{FourVector, Point4};
use crate::graphics::surface::Material;
use crate::integration::solvers::positive_root;

use glam::{Vec3, Vec4, Mat4};


pub struct PhotonIntersection<'a> {
    pub g: Mat4,
    pub point: Point4,
    pub normal: Vec3,
    pub material: &'a dyn Material
}


pub struct Photon {
    pub pos: Point4,
    pub vel: Vec4,
}

impl Photon {
    pub fn from_space_vel(g: Mat4, pos: Point4, vel: Vec3) -> Self {
        // g must be symmetric.

        // we need to find the fist velocity component in terms of vel to verify the normalization vel * g * vel = 0
        // we have:
        // g_00 (k^0)^2 + 2*g_0i k^0 k^i + g_ij k^i k^j = 0

        let b = 2.0 * (
            g.col(0)[1] * vel.x +
            g.col(0)[2] * vel.y +
            g.col(0)[3] * vel.z
        );
    
        let vx = vel.x; let vy = vel.y; let vz = vel.z;
        let mut c = 0.;
        c += g.col(1)[1] * vx * vx;
        c += 2.0 * g.col(1)[2] * vx * vy;
        c += 2.0 * g.col(1)[3] * vx * vz;
        c += g.col(2)[2] * vy * vy;
        c += 2.0 * g.col(2)[3] * vy * vz;
        c += g.col(3)[3] * vz * vz;

        let k_0 = positive_root(g.col(0)[0], b, c);

        Self::new(pos, Vec4::from_space_time(k_0, vel))
    }

    pub fn new(pos: Point4, vel: Vec4) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }

    }
}