use crate::graphics::ray::{PhotonIntersection, Ray};
use crate::graphics::vector::{self, Color, Point3};
use glam::Vec3;

pub trait Material {
    fn color(&self) -> Color;
    fn emission(&self) -> Color;
    fn scatter(&self, ray: &Ray, hit: &PhotonIntersection) -> Option<Ray>;
}

pub struct Diffuse {
    pub color: Color,
    pub emission: Color,
}

impl Material for Diffuse {
    fn color(&self) -> Color {
        self.color
    }

    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(&self, _ray: &Ray, hit: &PhotonIntersection) -> Option<Ray> {
        Some(Ray {
            pos: hit.point + 0.001 * hit.normal,
            vel: (hit.normal + vector::random_on_sphere() / 2.0).normalize(),
        })
    }
}

pub struct Metal {
    pub color: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl Material for Metal {
    fn color(&self) -> Color {
        self.color
    }

    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(&self, ray: &Ray, hit: &PhotonIntersection) -> Option<Ray> {
        let incoming = ray.vel.normalize();
        let reflected = incoming - 2.0 * incoming.dot(hit.normal) * hit.normal;
        let fuzz = self.fuzz.clamp(0.0, 1.0);

        let scattered = Ray {
            pos: hit.point + 0.001 * hit.normal,
            vel: (reflected + fuzz * vector::random_on_sphere()).normalize(),
        };

        if scattered.vel.dot(hit.normal) > 0.0 {
            Some(scattered)
        } else {
            None
        }
    }
}

pub trait Surface {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Ray) -> Option<PhotonIntersection> {
        let x = ray.pos - self.center;
        if x.length() <= self.radius {
            return Some(PhotonIntersection {
                point: self.center + self.radius * x.normalize(),
                normal: x.normalize(),
                material: self.material.as_ref(),
            });
        } else {
            return None;
        }
    }
}