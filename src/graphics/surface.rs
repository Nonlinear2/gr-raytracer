use crate::graphics::ray::{WorldPhotonState, WorldPhoton};
use crate::graphics::vector::{Point3, ThreeVector, CoordinateSystem, random_on_sphere};
use crate::graphics::color::Color;

pub trait Material {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn scatter(&self, hit: &WorldPhotonState) -> Option<(Color, ThreeVector)>;
}

pub struct Diffuse {
    pub albedo: Color,
    pub emission: Color,
}

impl Material for Diffuse {
    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(&self, hit: &WorldPhotonState) -> Option<(Color, ThreeVector)> {
        let new_direction = 
            (hit.normal.unwrap() + random_on_sphere(CoordinateSystem::Cartesian) * 0.5).normalize();
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

    fn scatter(&self, hit: &WorldPhotonState) -> Option<(Color, ThreeVector)> {
        assert!(0.0 <= self.fuzz);
        assert!(self.fuzz <= 1.0);

        let incoming = hit.world_photon.vel.normalize();
        let reflected = incoming - 2.0 * incoming.dot(hit.normal.unwrap()) * hit.normal.unwrap();

        let new_direction = 
            (reflected + self.fuzz * random_on_sphere(CoordinateSystem::Cartesian)).normalize();

        if new_direction.dot(hit.normal.unwrap()) > 0.0 {
            Some((self.albedo, new_direction))
        } else {
            None
        }
    }
}

pub trait Surface {
    fn hit(&self, ray: &WorldPhoton) -> Option<WorldPhotonState<'_>>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &WorldPhoton) -> Option<WorldPhotonState<'_>> {
        let x = ray.pos - self.center;
        if x.length() <= self.radius {
            return Some(WorldPhotonState {
                world_photon: WorldPhoton { pos: self.center + x.normalize() * (1.000001 * self.radius), vel: ray.vel },
                normal: Some(x.normalize()),
                material: Some(self.material.as_ref()),
            });
        } else {
            return None;
        }
    }
}