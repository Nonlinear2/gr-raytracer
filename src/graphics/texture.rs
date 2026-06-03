use std::path::Path;

use bytemuck::{Pod, Zeroable};

#[repr(u32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureId {
    NONE = 0,
    ACCRETION = 1,
    BACKGROUND = 2,
}

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

impl Texture {
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextureSegment {
    data: [u32; 4],
}

pub struct PackedTextures {
    segments: Vec<TextureSegment>,
}

impl PackedTextures {
    pub fn as_slice(&self) -> &[u8] {
        bytemuck::cast_slice(&self.segments)
    }
}

#[derive(Clone)]
pub struct Textures {
    pub accretion: Texture,
    pub background: Texture,
}

impl Textures {
    pub fn as_packed_texture(&self) -> PackedTextures {
        let header_count = 3u32;
        let acc_pixels = self.accretion.pixels.len() as u32;
        let bg_pixels = self.background.pixels.len() as u32;

        let mut segments = Vec::with_capacity((header_count + acc_pixels + bg_pixels) as usize);

        segments.push(TextureSegment {
            data: [1, 1, 0, 0],
        });
        segments.push(TextureSegment {
            data: [self.accretion.width, self.accretion.height, header_count, 0],
        });
        segments.push(TextureSegment {
            data: [
                self.background.width,
                self.background.height,
                header_count + acc_pixels,
                0,
            ],
        });

        segments.extend(self.accretion.pixels.iter().map(|pixel| TextureSegment {
            data: [
                pixel[0].to_bits(),
                pixel[1].to_bits(),
                pixel[2].to_bits(),
                pixel[3].to_bits(),
            ],
        }));

        segments.extend(self.background.pixels.iter().map(|pixel| TextureSegment {
            data: [
                pixel[0].to_bits(),
                pixel[1].to_bits(),
                pixel[2].to_bits(),
                pixel[3].to_bits(),
            ],
        }));

        PackedTextures { segments }
    }
}