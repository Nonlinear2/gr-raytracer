use std::path::Path;

use wgpu::{Device, Queue, Sampler, TextureView};

#[repr(u32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureId {
    NONE = 0,
    ACCRETION = 1,
    BACKGROUND = 2,
}

impl TextureId {
    pub const COUNT: usize = 3;

    pub const fn as_index(self) -> usize {
        self as usize
    }
}

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

impl Texture {
    /// brightness is on a scale of 0 to 1
    pub fn from_file(path: impl AsRef<Path>, brightness: f32) -> Result<Self, String> {
        let image = image::open(path.as_ref())
            .map_err(|err| format!("failed to load texture {:?}: {err}", path.as_ref()))?
            .flipv()
            .to_rgba8();

        let (width, height) = image.dimensions();
        let pixels = image.pixels().map(|pixel| {
            [
                pixel[0] as f32 * brightness / 255.0,
                pixel[1] as f32 * brightness / 255.0,
                pixel[2] as f32 * brightness / 255.0,
                pixel[3] as f32 / 255.0,
            ]
        }).collect();

        Ok(Self { width, height, pixels})
    }

    fn solid_rgba(rgba: [f32; 4]) -> Self {
        Self { width: 1, height: 1, pixels: vec![rgba] }
    }

    pub fn to_wgpu_texture(&self, device: &Device) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    pub fn write_to_queue(&self, wgpu_texture: &wgpu::Texture, queue: &Queue) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            // Vec<[f32;4]> -> &[u8]
            bytemuck::cast_slice(&self.pixels),

            wgpu::TexelCopyBufferLayout {
                offset: 0,

                // 4 floats * 4 bytes per float
                bytes_per_row: Some(16 * self.width),

                rows_per_image: Some(self.height),
            },

            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_view(&self, wgpu_texture: &wgpu::Texture) -> TextureView {
        wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_sampler(&self, device: &Device) -> Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,

            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,

            ..Default::default()
        })
    }
}

#[derive(Clone)]
pub struct Textures {
    textures: [Texture; TextureId::COUNT],
}

impl Textures {
    pub fn new(accretion: Texture, background: Texture) -> Self {
        Self {
            textures: [
                Texture::solid_rgba([0.0, 0.0, 0.0, 0.0]),
                accretion,
                background,
            ],
        }
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        &self.textures[id.as_index()]
    }
}