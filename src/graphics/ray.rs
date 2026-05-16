use crate::graphics::vector::{FourVector, Point4, SphPoint4, SphVec4, SphVec3};
use crate::graphics::surface::Material;
use crate::integration::solvers::positive_root;

use glam::{Vec3, Vec4, Mat4};


pub struct PhotonIntersection<'a> {
    pub g: Mat4,
    pub point: SphPoint4,
    pub normal: Vec3,
    pub material: &'a dyn Material
}


pub struct Photon {
    pub pos: SphPoint4,
    pub vel: SphVec4,
}

impl Photon {
    pub fn from_space_vel(g: Mat4, pos: SphPoint4, vel: SphVec3) -> Self {
        // g must be symmetric.

        // we need to find the fist velocity component in terms of vel to verify the normalization vel * g * vel = 0
        // we have:
        // g_00 (k^0)^2 + 2*g_i0 k^j k^0 + g_ij k^i k^j = 0

        let mut b = 0.;
        for i in 1..4 {
            b += g.col(0)[i] * vel[i];
        }
        b *= 2.;
    
        let mut c = 0.;
        for i in 1..4 {
            for j in 1..4 {
                c += g.col(i)[j] * vel[i] * vel[j];
            }
        }

        let k_0 = positive_root(g.col(0)[0], b, c);

        Self::new(pos, SphVec4::from_space_time(k_0, vel))
    }

    pub fn new(pos: SphPoint4, vel: SphVec4) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }
}