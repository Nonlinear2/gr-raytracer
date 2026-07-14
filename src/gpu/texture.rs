use wgpu::{Device, Queue, Sampler, TextureView};

use crate::graphics::texture::Texture;

pub trait GpuTexture {
    fn to_wgpu_texture(&self, device: &Device) -> wgpu::Texture;
    fn write_to_queue(&self, wgpu_texture: &wgpu::Texture, queue: &Queue);
    fn get_view(&self, wgpu_texture: &wgpu::Texture) -> TextureView;
    fn get_sampler(&self, device: &Device) -> Sampler;
}

impl GpuTexture for Texture {
    fn to_wgpu_texture(&self, device: &Device) -> wgpu::Texture {
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

    fn write_to_queue(&self, wgpu_texture: &wgpu::Texture, queue: &Queue) {
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

    fn get_view(&self, wgpu_texture: &wgpu::Texture) -> TextureView {
        wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn get_sampler(&self, device: &Device) -> Sampler {
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