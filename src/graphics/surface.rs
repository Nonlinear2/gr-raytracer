use crate::graphics::ray::{PhotonIntersection, Photon};
use crate::graphics::vector::{FourVector, Point3, ThreeVector, CoordinateSystem, random_on_sphere};
use crate::graphics::color::Color;
use::glam::{Vec4, Mat4};

pub trait Material {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn scatter(&self, incoming: ThreeVector, hit: &PhotonIntersection) -> Option<(Color, ThreeVector)>;
}

pub struct Diffuse {
    pub albedo: Color,
    pub emission: Color,
}

impl Material for Diffuse {
    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(&self, incoming: ThreeVector, hit: &PhotonIntersection) -> Option<(Color, ThreeVector)> {
        assert!(incoming.coordinate_system == CoordinateSystem::Cartesian);
        let new_direction = 
            (hit.normal + random_on_sphere(CoordinateSystem::Cartesian) * 0.5).normalize();
        Some((self.albedo, new_direction))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl Material for Metal {
    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(&self, incoming: ThreeVector, hit: &PhotonIntersection) -> Option<(Color, ThreeVector)> {
        assert!(incoming.coordinate_system == CoordinateSystem::Cartesian);
        assert!(0.0 <= self.fuzz);
        assert!(self.fuzz <= 1.0);

        let incoming = incoming.normalize();
        let reflected = incoming - 2.0 * incoming.dot(hit.normal) * hit.normal;

        let new_direction = 
            (reflected + self.fuzz * random_on_sphere(CoordinateSystem::Cartesian)).normalize();

        if new_direction.dot(hit.normal) > 0.0 {
            Some((self.albedo, new_direction))
        } else {
            None
        }
    }
}

pub struct NoMaterial {}

impl Material for NoMaterial {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn scatter(&self, _incoming: ThreeVector, _hit: &PhotonIntersection) -> Option<(Color, ThreeVector)> {
        None
    }
}


pub trait Surface {
    fn hit(&self, ray: &Photon) -> Option<PhotonIntersection>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Photon) -> Option<PhotonIntersection> {
        let x = ray.pos.space() - self.center;
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