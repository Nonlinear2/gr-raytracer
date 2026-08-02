use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use rand::rngs::StdRng;

use crate::geometry::point::Point3;
use crate::geometry::vector::ThreeVector;
use crate::graphics::color::Color;
use crate::math::sphere_uv;
use crate::scene::texture::TextureId;

const EPS: f32 = 10e-6;

pub struct HitData {
    pub hit_point: Point3<ChartWorld>,
    pub incoming_dir: ThreeVector<TangentWorld>,
    pub normal: ThreeVector<TangentWorld>,
}

pub trait CpuMaterial {
    /// texture_rgb is the sampled texture color if the object has one
    fn albedo(&self, texture_rgb: Option<Color>) -> Color;
    fn emission(&self, texture_rgb: Option<Color>) -> Color;
    fn scatter(&self, hit_data: &HitData, rng: &mut StdRng) -> Option<ThreeVector<TangentWorld>>;
}

#[allow(dead_code)]
pub struct Diffuse {
    pub color: Color,
    pub emission: Color,
}

impl CpuMaterial for Diffuse {
    fn albedo(&self, texture_rgb: Option<Color>) -> Color {
        match texture_rgb {
            Some(texture_rgb) => self.color * texture_rgb,
            None => self.color,
        }
    }

    fn emission(&self, texture_rgb: Option<Color>) -> Color {
        match texture_rgb {
            Some(_) => self.emission * self.albedo(texture_rgb),
            None => self.emission,
        }
    }

    fn scatter(&self, hit_data: &HitData, _rng: &mut StdRng) -> Option<ThreeVector<TangentWorld>> {
        Some(hit_data.normal.normalize())
    }
}

#[allow(dead_code)]
pub struct Metal {
    pub color: Color,
    pub emission: Color,
    pub fuzz: f32,
}

impl CpuMaterial for Metal {
    fn albedo(&self, _texture_rgb: Option<Color>) -> Color {
        self.color
    }

    fn emission(&self, _texture_rgb: Option<Color>) -> Color {
        self.emission
    }

    fn scatter(&self, hit_data: &HitData, rng: &mut StdRng) -> Option<ThreeVector<TangentWorld>> {
        let rand_dir = ThreeVector::random_unit(rng, hit_data.normal.vector_space);
        let fuzz = self.fuzz.clamp(0.0, 0.99);

        let reflected = hit_data.incoming_dir - 2.0 * hit_data.incoming_dir.dot(hit_data.normal) * hit_data.normal;
        let scattered = (reflected + fuzz * rand_dir).normalize();

        if scattered.dot(hit_data.normal) <= 0.0 {
            return None;
        }
        Some(scattered)
    }
}

pub trait Material: CpuMaterial + GpuMaterial {}

impl<T: CpuMaterial + GpuMaterial> Material for T {}

pub trait CpuObject {
    fn hit(&self, prev_pos: Point3<ChartWorld>, new_pos: Point3<ChartWorld>) -> Option<HitData>;
    fn uv(&self, hit_point: Point3<ChartWorld>) -> (f32, f32);
    fn texture(&self) -> TextureId;
    fn material(&self) -> &dyn Material;
}

#[allow(dead_code)]
pub struct Sphere {
    pub center: Point3<ChartWorld>,
    pub radius: f32,
    pub material: Box<dyn Material>,
    pub texture: TextureId,
}

impl CpuObject for Sphere {
    fn hit(&self, prev_pos: Point3<ChartWorld>, new_pos: Point3<ChartWorld>) -> Option<HitData> {
        let offset = (new_pos - self.center).as_threevector();
        if offset.length() > self.radius {
            return None;
        }

        let normal = offset.normalize();

        Some(HitData {
            hit_point: self.center + (normal * (self.radius * (1.0 + EPS))).as_point3(), // avoid precision errors
            incoming_dir: (new_pos - prev_pos).as_threevector().normalize(),
            normal,
        })
    }

    fn uv(&self, hit_point: Point3<ChartWorld>) -> (f32, f32) {
        sphere_uv((hit_point - self.center).as_threevector().normalize())
    }

    fn texture(&self) -> TextureId {
        self.texture
    }

    fn material(&self) -> &dyn Material {
        self.material.as_ref()
    }
}

#[allow(dead_code)]
pub struct Disc {
    pub center: Point3<ChartWorld>,
    pub normal: ThreeVector<TangentWorld>,
    pub radius: f32,
    pub inner_radius: f32,
    pub material: Box<dyn Material>,
    pub texture: TextureId,
}

impl CpuObject for Disc {
    fn hit(&self, prev_pos: Point3<ChartWorld>, new_pos: Point3<ChartWorld>) -> Option<HitData> {
        let normal = self.normal.normalize();
        let segment = (new_pos - prev_pos).as_threevector();
        let segment_dot_normal = segment.dot(normal);

        if segment_dot_normal.abs() < EPS { // movement parallel to disc, no intersection.
            return None;
        }

        // we are searching for t such that
        // (prev_pos + t*segment - center) . normal = 0
        // (t*segment) . normal = (center - prev_pos) . normal
        // t = ((center - prev_pos) . normal) / (segment . normal)
        let t = (self.center - prev_pos).as_threevector().dot(normal) / segment_dot_normal;

        if t < -EPS || t > 1.0 + EPS { // check if intersection with disc plane is outside of prev_pos and new_pos
            return None;
        }

        let hit_point = prev_pos + (segment * t).as_point3();
        let distance_from_center = (hit_point - self.center).as_threevector().length();
        if distance_from_center < self.inner_radius || distance_from_center > self.radius {
            return None;
        }

        let directed_normal = if segment_dot_normal > 0.0 { -normal } else { normal };

        Some(HitData {
            hit_point: hit_point + (directed_normal * EPS).as_point3(),
            incoming_dir: segment.normalize(),
            normal: directed_normal,
        })
    }

    fn uv(&self, hit_point: Point3<ChartWorld>) -> (f32, f32) {
        let normal = self.normal.normalize().as_vec3();
        let reference_axis = if normal.y.abs() > 0.5 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) };
        let tangent = reference_axis.cross(normal).normalize();
        let bitangent = normal.cross(tangent);

        let local = (hit_point - self.center).as_threevector().as_vec3();
        let radial_uv = (local.length() - self.inner_radius) / (self.radius - self.inner_radius).max(EPS);
        let u = local.dot(bitangent).atan2(local.dot(tangent)) / std::f32::consts::TAU + 0.5;
        (u, radial_uv)
    }

    fn texture(&self) -> TextureId {
        self.texture
    }

    fn material(&self) -> &dyn Material {
        self.material.as_ref()
    }
}

pub trait Object: CpuObject + GpuObject {}

impl<T: CpuObject + GpuObject> Object for T {}

pub const GPU_OBJECT_SPHERE: u32 = 1;
pub const GPU_OBJECT_DISC: u32 = 2;

#[allow(dead_code)]
pub const GPU_MATERIAL_DIFFUSE: u32 = 0;
#[allow(dead_code)]
pub const GPU_MATERIAL_METAL: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedMaterial {
    pub kind: u32,
    pub _pad0: [u32; 3],
    pub color: [f32; 3],
    pub params: f32,
    pub _pad1: [u32; 4],
}

pub trait GpuMaterial {
    fn packed_material_kind(&self) -> u32;
    fn packed_material_params(&self) -> [f32; 4];
    fn packed_emission_params(&self) -> [f32; 4];
}


impl GpuMaterial for Diffuse {

    fn packed_material_kind(&self) -> u32 {
        GPU_MATERIAL_DIFFUSE
    }

    fn packed_material_params(&self) -> [f32; 4] {
        [
            self.color.r,
            self.color.g,
            self.color.b,
            0.0,
        ]
    }

    fn packed_emission_params(&self) -> [f32; 4] {
        [
            self.emission.r,
            self.emission.g,
            self.emission.b,
            0.0,
        ]
    }
}

impl GpuMaterial for Metal {

    fn packed_material_kind(&self) -> u32 {
        GPU_MATERIAL_METAL
    }

    fn packed_material_params(&self) -> [f32; 4] {
        [
            self.color.r,
            self.color.g,
            self.color.b,
            self.fuzz,
        ]
    }

    fn packed_emission_params(&self) -> [f32; 4] {
        [
            self.emission.r,
            self.emission.g,
            self.emission.b,
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

pub trait GpuObject {
    fn as_packed_object(&self) -> Option<PackedObject>;
}

impl GpuObject for Sphere {
    fn as_packed_object(&self) -> Option<PackedObject> {
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

impl GpuObject for Disc {
    fn as_packed_object(&self) -> Option<PackedObject> {
        debug_assert!(self.radius.is_finite() && self.radius >= 0.0);
        debug_assert!(self.inner_radius.is_finite() && self.inner_radius >= 0.0);
        debug_assert!(self.inner_radius <= self.radius);

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
            data1: [normal.x(), normal.y(), normal.z(), self.inner_radius],
            emission_params: self.material.packed_emission_params(),
        })
    }
}
