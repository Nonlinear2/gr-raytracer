use crate::graphics::vector;

pub struct Ray {
    pub origin: vector::Point3,
    pub direction: vector::Vec3,
}

impl Ray {
    fn at(self, t: f32) -> vector::Point3 {
        return self.origin + t*self.direction;
    }
}