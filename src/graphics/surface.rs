use crate::graphics::ray::{Ray, PhotonIntersection};
use crate::graphics::vector::{Color, Point3};
use glam::Vec3;

pub struct Material {
    pub color: Color,
    pub emission: Color,
}

impl Material {
    fn scatter(){

    }
}

pub trait Surface {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub color: Color,
    pub material: Material,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection> {
        if (ray.pos - self.center).length() <= self.radius {
            return Some(PhotonIntersection {
                point: ray.pos,
                normal: (ray.pos - self.center).normalize(),
                material: &self.material,
            });
        } else {
            return None;
        }
    }
}