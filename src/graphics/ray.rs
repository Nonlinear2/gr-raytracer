use crate::graphics::vector::{FourVector, Point3, Point4, ThreeVector};
use crate::graphics::surface::Material;
use crate::integration::solvers::positive_root;

use glam::{Vec3, Vec4, Mat4};


pub enum StopReason {
    MaxStepsReached,
    ObjectHit,
    HorizonHit,
}

pub struct PhotonIntersection<'a> {
    pub point: Point4,
    pub normal: ThreeVector,
    pub material: &'a dyn Material
}

pub struct Photon {
    pub pos: Point4,
    pub vel: FourVector,
}

impl Photon {
    pub fn new(pos: Point4, vel: FourVector) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }
}