use std::path::Path;

use crate::geometry::manifold::Chart;
use crate::geometry::point::Point3;
use crate::graphics::color::Color;

use bytemuck::{Pod, Zeroable};

pub const GPU_OBJECT_SPHERE: u32 = 1;
pub const GPU_OBJECT_DISC: u32 = 2;

#[allow(dead_code)]
pub const GPU_MATERIAL_DIFFUSE: u32 = 0;
pub const GPU_MATERIAL_METAL: u32 = 1;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Texture {
    NONE = 0,
    ACCRETION = 1,
    BACKGROUND = 2,
}

#[derive(Clone)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

impl TextureImage {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let image = image::open(path.as_ref())
            .map_err(|err| format!("failed to load texture {:?}: {err}", path.as_ref()))?
            .flipv()
            .to_rgba8();

        let (width, height) = image.dimensions();
        let pixels = image.pixels().map(|pixel| {
            [
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
                pixel[3] as f32 / 255.0,
            ]
        }).collect();

        Ok(Self { width, height, pixels })
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Textures {
    pub accretion_disc: TextureImage,
    pub sky_background: TextureImage,
}

pub trait Material {
    fn packed_material_kind(&self) -> u32;

    fn packed_material_params(&self) -> [f32; 4];

    fn packed_emission_params(&self) -> [f32; 4];
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedMaterial {
    pub kind: u32,
    pub _pad0: [u32; 3],
    pub color: [f32; 3],
    pub params: f32,
    pub _pad1: [u32; 4],
}

#[allow(dead_code)]
pub struct Diffuse {
    pub color: Color,
    pub emission: Color,
}

impl Material for Diffuse {

    fn packed_material_kind(&self) -> u32 {
        GPU_MATERIAL_DIFFUSE
    }

    fn packed_material_params(&self) -> [f32; 4] {
        [
            self.color.r / 255.0,
            self.color.g / 255.0,
            self.color.b / 255.0,
            0.0,
        ]
    }

    fn packed_emission_params(&self) -> [f32; 4] {
        [
            self.emission.r / 255.0,
            self.emission.g / 255.0,
            self.emission.b / 255.0,
            0.0,
        ]
    }
}

#[allow(dead_code)]
pub struct Metal {
    pub color: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl Material for Metal {

    fn packed_material_kind(&self) -> u32 {
        GPU_MATERIAL_METAL
    }

    fn packed_material_params(&self) -> [f32; 4] {
        [
            self.color.r / 255.0,
            self.color.g / 255.0,
            self.color.b / 255.0,
            self.fuzz,
        ]
    }

    fn packed_emission_params(&self) -> [f32; 4] {
        [
            self.emission.r / 255.0,
            self.emission.g / 255.0,
            self.emission.b / 255.0,
            0.0,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedObject {
    pub kind: u32,
    pub texture: u32,
    pub _pad0: [u32; 2],
    pub material: PackedMaterial,
    pub _pad1: [u32; 4],
    pub data0: [f32; 4],
    pub data1: [f32; 4],
    pub emission_params: [f32; 4],
}

pub trait Object {
    fn as_packed_object(&self) -> Option<PackedObject>;
}

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
    pub texture: Texture,
}

impl Object for Sphere {
    fn as_packed_object(&self) -> Option<PackedObject> {
        assert!(self.center.chart == Chart::CartesianWorld);

        Some(PackedObject {
            kind: GPU_OBJECT_SPHERE,
            texture: self.texture as u32,
            _pad0: [0u32; 2],
            material: PackedMaterial {
                kind: self.material.packed_material_kind(),
                _pad0: [0u32; 3],
                color: [
                    self.material.packed_material_params()[0],
                    self.material.packed_material_params()[1],
                    self.material.packed_material_params()[2],
                ],
                params: self.material.packed_material_params()[3],
                _pad1: [0u32; 4],
            },
            _pad1: [0u32; 4],
            data0: [self.center.x(), self.center.y(), self.center.z(), self.radius],
            data1: [0.0; 4],
            emission_params: self.material.packed_emission_params(),
        })
    }
}

#[allow(dead_code)]
pub struct Disc {
    pub center: Point3,
    pub normal: crate::geometry::vector::ThreeVector,
    pub radius: f32,
    pub material: Box<dyn Material>,
    pub texture: Texture,
}

impl Object for Disc {
    fn as_packed_object(&self) -> Option<PackedObject> {
        assert!(self.center.chart == Chart::CartesianWorld);
        assert!(matches!(self.normal.vector_space, crate::geometry::vector::TangentSpace::CartesianWorld));

        let normal = self.normal.normalize();

        Some(PackedObject {
            kind: GPU_OBJECT_DISC,
            texture: self.texture as u32,
            _pad0: [0u32; 2],
            material: PackedMaterial {
                kind: self.material.packed_material_kind(),
                _pad0: [0u32; 3],
                color: [
                    self.material.packed_material_params()[0],
                    self.material.packed_material_params()[1],
                    self.material.packed_material_params()[2],
                ],
                params: self.material.packed_material_params()[3],
                _pad1: [0u32; 4],
            },
            _pad1: [0u32; 4],
            data0: [self.center.x(), self.center.y(), self.center.z(), self.radius],
            data1: [normal.x(), normal.y(), normal.z(), 0.0],
            emission_params: self.material.packed_emission_params(),
        })
    }
}