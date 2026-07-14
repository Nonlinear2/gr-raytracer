use crate::geometry::manifold::Chart;
use crate::geometry::point::Point3;
use crate::graphics::color::Color;
use crate::graphics::texture::TextureId;

pub trait Material {
}

#[allow(dead_code)]
pub struct Diffuse {
    pub color: Color,
    pub emission: Color,
}

impl Material for Diffuse {

}

#[allow(dead_code)]
pub struct Metal {
    pub color: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl Material for Metal {

}

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
