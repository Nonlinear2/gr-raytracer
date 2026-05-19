use crate::graphics::ray::{WorldPhotonState, WorldPhoton};
use crate::graphics::point::Point3;
use crate::graphics::vector::{TangentSpace, ThreeVector, random_on_sphere};
use crate::graphics::color::Color;

pub trait Material {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn scatter(&self, hit: &WorldPhotonState) -> Option<(Color, ThreeVector)>;
}

#[allow(dead_code)]
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
            (hit.normal.unwrap() + random_on_sphere(TangentSpace::Cartesian) * 0.5).normalize();
        Some((self.albedo, new_direction))
    }
}

#[allow(dead_code)]
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
        println!("[Metal::scatter] incoming: {}", incoming.as_vec3());
        
        let reflected = incoming - 2.0 * incoming.dot(hit.normal.unwrap()) * hit.normal.unwrap();
        println!("[Metal::scatter] reflected: {}", reflected.as_vec3());
        
        let noise = self.fuzz * random_on_sphere(TangentSpace::Cartesian);
        println!("[Metal::scatter] noise: {}, fuzz: {}", noise.as_vec3(), self.fuzz);
        
        let pre_norm = reflected + noise;
        println!("[Metal::scatter] pre_norm: {}", pre_norm.as_vec3());
        
        let new_direction = pre_norm.normalize();
        println!("[Metal::scatter] new_direction: {}", new_direction.as_vec3());

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

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &WorldPhoton) -> Option<WorldPhotonState<'_>> {
        let x = ray.pos - self.center;
        let dist = x.distance_to_zero();
        
        if dist <= self.radius {
            eprintln!("[Sphere::hit] ray.pos: {:?}, center: {:?}, x: {:?}, dist: {}", ray.pos.as_vec3(), self.center.as_vec3(), x.as_vec3(), dist);

            let scale_factor = 1.000001 * self.radius / dist;
            eprintln!("[Sphere::hit] scale_factor: {}", scale_factor);
            
            let new_pos = self.center + x * scale_factor;
            eprintln!("[Sphere::hit] new_pos: {:?}", new_pos.as_vec3());
            
            let normal = x.as_threevector().normalize();
            eprintln!("[Sphere::hit] normal: {:?}", normal.as_vec3());
            
            return Some(WorldPhotonState {
                world_photon: WorldPhoton { pos: new_pos, vel: ray.vel },
                normal: Some(normal),
                material: Some(self.material.as_ref()),
            });
        } else {
            return None;
        }
    }
}