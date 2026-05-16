use crate::graphics::ray::{PhotonIntersection, Photon};
use crate::graphics::vector::{self, Color, Point3, FourVector};
use::glam::{Vec4, Mat4};

pub trait Material {
    fn emission(&self) -> Color {
        Color::ZERO
    }

    fn scatter(
        &self,
        ray_in: &Photon,
        hit: &PhotonIntersection,
        attenuation: &mut Color,
        scattered: &mut Photon,
    ) -> bool;
}

pub struct Diffuse {
    pub albedo: Color,
    pub emission: Color,
}

impl Material for Diffuse {
    fn emission(&self) -> Color {
        self.emission
    }

    fn scatter(
        &self,
        _ray: &Photon,
        hit: &PhotonIntersection,
        attenuation: &mut Color,
        scattered: &mut Photon,
    ) -> bool {
        *attenuation = self.albedo;
        *scattered = Photon::new(
            hit.g,
            Vec4::from_space_time(hit.point.time(), hit.point.space() + 0.001 * hit.normal),
            (hit.normal + vector::random_on_sphere() / 2.0).normalize(),
        );

        true
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

    fn scatter(
        &self,
        ray: &Photon,
        hit: &PhotonIntersection,
        attenuation: &mut Color,
        scattered: &mut Photon,
    ) -> bool {
        let incoming = ray.vel.space().normalize();
        let reflected = incoming - 2.0 * incoming.dot(hit.normal) * hit.normal;
        let fuzz = self.fuzz.clamp(0.0, 1.0);

        *attenuation = self.albedo;
        *scattered = Photon::new(
            hit.g,
            Vec4::from_space_time(hit.point.time(), hit.point.space() + 0.001 * hit.normal),
            (reflected + fuzz * vector::random_on_sphere()).normalize(),
        );

        scattered.vel.space().dot(hit.normal) > 0.0
    }
}

pub struct BlackHole {}

impl Material for BlackHole {
    fn emission(&self) -> Color {
        Color::ZERO
    }

    fn scatter(
        &self,
        _ray: &Photon,
        _hit: &PhotonIntersection,
        _attenuation: &mut Color,
        _scattered: &mut Photon,
    ) -> bool {
        false
    }
}

pub trait Surface {
    fn hit(&self, ray: &Photon, g: Mat4) -> Option<PhotonIntersection>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn hit(&self, ray: &Photon, g: Mat4) -> Option<PhotonIntersection> {
        let x = ray.pos.space() - self.center;
        if x.length() <= self.radius {
            return Some(PhotonIntersection {
                g: g,
                point: Vec4::from_space_time(ray.pos.time(), self.center + self.radius * x.normalize()),
                normal: x.normalize(),
                material: self.material.as_ref(),
            });
        } else {
            return None;
        }
    }
}