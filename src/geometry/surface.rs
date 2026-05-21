use crate::geometry::manifold::Chart;
use crate::geometry::photon::{WorldPhoton3State, Photon3};
use crate::geometry::point::Point3;
use crate::geometry::vector::{TangentSpace, ThreeVector, random_on_sphere};
use crate::graphics::color::Color;

use rand::rngs::StdRng;

pub trait Material {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn scatter(&self, hit: &WorldPhoton3State, rng: &mut StdRng) -> Option<(Color, ThreeVector)>;
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

    fn scatter(&self, hit: &WorldPhoton3State, rng: &mut StdRng) -> Option<(Color, ThreeVector)> {
        let new_direction = 
            (hit.normal.unwrap() + random_on_sphere(TangentSpace::CartesianWorld, rng) * 0.5).normalize();
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

    fn scatter(&self, hit: &WorldPhoton3State, rng: &mut StdRng) -> Option<(Color, ThreeVector)> {
        assert!(0.0 <= self.fuzz);
        assert!(self.fuzz <= 1.0);

        let incoming = hit.photon3.vel.normalize();
        let reflected = incoming - 2.0 * incoming.dot(hit.normal.unwrap()) * hit.normal.unwrap();
        let noise = self.fuzz * random_on_sphere(TangentSpace::CartesianWorld, rng);
    
        let new_direction = (reflected + noise).normalize();

        if new_direction.dot(hit.normal.unwrap()) > 0.0 {
            Some((self.albedo, new_direction))
        } else {
            None
        }
    }
}

pub trait Surface {
    fn hit(&self, ray: &Photon3) -> bool;
    fn get_hit_data(&self, ray: &Photon3) -> WorldPhoton3State<'_>;
}

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Photon3) -> bool {
        match ray.pos.chart {
            Chart::CartesianWorld => {
                (ray.pos - self.center).distance_to_zero() <= self.radius    
            },
            Chart::Cartesian => {
                (ray.pos.as_chart(Chart::CartesianWorld) + .center - self.center).distance_to_zero() <= self.radius      
            }
            Chart::SphericalZ => {
                todo!()
            }
            Chart::SphericalX => {
                todo!()
            }
        }
    }

    fn get_hit_data(&self, world_ray: &Photon3) -> WorldPhoton3State<'_> {
        assert!(world_ray.pos.chart == Chart::CartesianWorld);
        assert!(world_ray.vel.vector_space == TangentSpace::CartesianWorld);
        assert!(self.hit(world_ray));

        let x = world_ray.pos - self.center;
        let dist = x.distance_to_zero();

        let scale_factor = 1.000001 * self.radius / dist;
        let new_pos = self.center + x * scale_factor;
        let normal = x.as_threevector().normalize();

        return WorldPhoton3State {
            photon3: Photon3 { pos: new_pos, vel: world_ray.vel },
            normal: Some(normal),
            material: Some(self.material.as_ref()),
        };
    }
}