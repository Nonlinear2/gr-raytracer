use crate::geometry::point::Point3;
use crate::gpu::surface::GpuMaterial;
use crate::graphics::color::Color;
use crate::graphics::texture::TextureId;

pub trait CpuMaterial {
}

#[allow(dead_code)]
pub struct Diffuse {
    pub color: Color,
    pub emission: Color,
}

impl CpuMaterial for Diffuse {

}

#[allow(dead_code)]
pub struct Metal {
    pub color: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl CpuMaterial for Metal {

}

pub trait Material: CpuMaterial + GpuMaterial {}

pub trait Object {
}

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
    pub texture: TextureId,
}

impl Object for Sphere {

}

#[allow(dead_code)]
pub struct Disc {
    pub center: Point3,
    pub normal: crate::geometry::vector::ThreeVector,
    pub radius: f32,
    pub inner_radius: f32,
    pub material: Box<dyn Material>,
    pub texture: TextureId,
}

impl Object for Disc {

}