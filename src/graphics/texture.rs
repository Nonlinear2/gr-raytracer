use std::path::Path;

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

    pub fn as_packed_texture(self) -> PackedTexture {
        PackedTexture {
            width: self.width,
            height: self.height,
            pixels: self.pixels,
        }
    }
}

#[derive(Clone)]
pub struct PackedTexture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AllTextures {
    pub accretion_disc: Texture,
    pub sky_background: Texture,
}

pub type PackedAllTextures = Vec<u8>;

impl AllTextures {
    pub fn as_packed_texture(self) -> PackedAllTextures {
        let tex0 = self.accretion_disc.as_packed_texture();
        let tex1 = self.sky_background.as_packed_texture();
        let textures = [tex0, tex1];

        // Build header: count (u32), pad (3*u32), then 3 infos (u32x4 each)
        let mut header_u32s: Vec<u32> = Vec::new();
        header_u32s.push(2u32); // count
        header_u32s.extend_from_slice(&[0u32, 0u32, 0u32]); // _pad0

        let mut current_offset: u32 = 0;
        for i in 0..3 {
            if i < 2 {
                let tex = &textures[i];
                header_u32s.push(tex.width);
                header_u32s.push(tex.height);
                header_u32s.push(current_offset);
                header_u32s.push(0u32);
                let pixels = tex.width.saturating_mul(tex.height);
                current_offset = current_offset.saturating_add(pixels);
            } else {
                header_u32s.extend_from_slice(&[0u32, 0u32, 0u32, 0u32]);
            }
        }

        // Flatten pixel floats
        let mut pixel_floats: Vec<f32> = Vec::new();
        for i in 0..2 {
            let tex = &textures[i];
            for px in &tex.pixels {
                pixel_floats.push(px[0]);
                pixel_floats.push(px[1]);
                pixel_floats.push(px[2]);
                pixel_floats.push(px[3]);
            }
        }

        let mut texture_bytes: Vec<u8> = Vec::new();
        texture_bytes.extend_from_slice(bytemuck::cast_slice(&header_u32s));
        if !pixel_floats.is_empty() {
            texture_bytes.extend_from_slice(bytemuck::cast_slice(&pixel_floats));
        }

        texture_bytes
    }
}