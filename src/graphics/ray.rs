use crate::graphics::vector::{Point3};
use crate::graphics::surface::Material;
use glam::Vec3;

pub struct PhotonIntersection<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: &'a dyn Material
}


pub struct Ray {
    pub pos: Point3,
    pub vel: Vec3,
}

impl Ray {
    // pub fn at(&self, t: f32) -> Point3 {
    //     return self.point + t*self.direction;
    // }
    pub fn step(&self, step_size: f32) -> Ray {
        Ray {
            pos: self.pos + self.vel.normalize() * step_size,
            vel: self.vel,
        }
    }
}