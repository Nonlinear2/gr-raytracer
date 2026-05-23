use crate::geometry::manifold::Chart;
use crate::geometry::point::Point3;
use crate::geometry::vector::{TangentSpace, ThreeVector, random_on_sphere};
use crate::graphics::color::Color;

use bytemuck::{Pod, Zeroable};
use rand::rngs::StdRng;

pub const GPU_OBJECT_SPHERE: u32 = 1;
pub const GPU_MATERIAL_DIFFUSE: u32 = 1;
pub const GPU_MATERIAL_METAL: u32 = 2;

pub trait Material {
    fn emission(&self) -> Color {
        Color::BLACK
    }

    fn gpu_material_kind(&self) -> u32;

    fn gpu_material_params(&self) -> [f32; 4];

    fn gpu_emission_params(&self) -> [f32; 4];
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedGpuObject {
    pub kind: u32,
    pub material_kind: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub data0: [f32; 4],
    pub material_params: [f32; 4],
    pub emission_params: [f32; 4],
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

    fn gpu_material_kind(&self) -> u32 {
        GPU_MATERIAL_DIFFUSE
    }

    fn gpu_material_params(&self) -> [f32; 4] {
        [
            self.albedo.r / 255.0,
            self.albedo.g / 255.0,
            self.albedo.b / 255.0,
            0.0,
        ]
    }

    fn gpu_emission_params(&self) -> [f32; 4] {
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
    pub albedo: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl Material for Metal {
    fn emission(&self) -> Color {
        self.emission
    }

    fn gpu_material_kind(&self) -> u32 {
        GPU_MATERIAL_METAL
    }

    fn gpu_material_params(&self) -> [f32; 4] {
        [
            self.albedo.r / 255.0,
            self.albedo.g / 255.0,
            self.albedo.b / 255.0,
            self.fuzz,
        ]
    }

    fn gpu_emission_params(&self) -> [f32; 4] {
        [
            self.emission.r / 255.0,
            self.emission.g / 255.0,
            self.emission.b / 255.0,
            0.0,
        ]
    }
}

pub trait Surface {
    fn as_packed_gpu_object(&self) -> Option<PackedGpuObject>;
}

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Surface for Sphere {
    fn as_packed_gpu_object(&self) -> Option<PackedGpuObject> {
        assert!(self.center.chart == Chart::CartesianWorld);
        Some(PackedGpuObject {
            kind: GPU_OBJECT_SPHERE,
            material_kind: self.material.gpu_material_kind(),
            _pad0: 0,
            _pad1: 0,
            data0: [self.center.x(), self.center.y(), self.center.z(), self.radius],
            material_params: self.material.gpu_material_params(),
            emission_params: self.material.gpu_emission_params(),
        })
    }
}