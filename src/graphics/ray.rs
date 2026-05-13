use crate::graphics::vector::{Point3, Vec3};

pub struct RayIntersection {
    pub point: Point3,
    pub normal: Vec3,
    pub t: f32,
}


pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Point3 {
        return self.origin + t*self.direction;
    }
}