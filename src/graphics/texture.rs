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

    fn solid_rgba(rgba: [f32; 4]) -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![rgba],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextureSegment {
    data: [f32; 4],
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

    #[allow(dead_code)]
    pub fn get(&self, id: TextureId) -> &Texture {
        &self.textures[id.as_index()]
    }

    pub fn as_packed_texture(&self) -> PackedTextures {
        let header_count = TextureId::COUNT as u32;
        let pixel_count = self
            .textures
            .iter()
            .map(|texture| texture.pixels.len() as u32)
            .sum::<u32>();

        let mut segments = Vec::with_capacity((header_count + pixel_count) as usize);
        let mut pixel_offset = header_count;

        for texture in &self.textures {
            segments.push(TextureSegment {
                data: [texture.width as f32, texture.height as f32, pixel_offset as f32, 0.0],
            });
            pixel_offset += texture.pixels.len() as u32;
        }

        for texture in &self.textures {
            segments.extend(texture.pixels.iter().map(|pixel| TextureSegment {data: *pixel}));
        }

        PackedTextures { segments }
    }
}