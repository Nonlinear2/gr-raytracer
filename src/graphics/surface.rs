use crate::graphics::ray::{Ray, RayIntersection};
use crate::graphics::vector::{Vec3};

pub trait Surface {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayIntersection>;
}


pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayIntersection> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(&oc);
        let c = oc.length_squared() - self.radius*self.radius;

        let delta = h*h - a*c;
        if delta < 0. {
            return None;
        }

        let mut root = (h - delta.sqrt()) / a;
        if root <= t_min || t_max <= root {
            root = (h + delta.sqrt()) / a;
            if root <= t_min || t_max <= root {
                return None;
            }
        }

        return Some(RayIntersection {
            point: ray.at(root),
            normal: (ray.at(root) - self.center) / self.radius,
            t: root
        });
    }
}