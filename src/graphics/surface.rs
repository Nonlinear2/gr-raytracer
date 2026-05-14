use crate::graphics::ray::{Ray, PhotonIntersection};
use crate::graphics::vector::{Vec3};

pub trait Surface {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection> {
        if (ray.pos - self.center).length() <= self.radius {
            return Some(PhotonIntersection {
                point: ray.pos,
                normal: (ray.pos - self.center).normalize(),
            });
        } else {
            return None;
        }
    }
}