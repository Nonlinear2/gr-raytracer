use std::path::Path;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureId {
    NONE = 0,
    ACCRETION = 1,
    SKY = 2,
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
}

#[derive(Clone)]
pub struct Textures {
    textures: [Texture; TextureId::COUNT],
}

impl Textures {
    pub fn new(accretion: Texture, sky: Texture) -> Self {
        Self {
            textures: [
                Texture::solid_rgba([0.0, 0.0, 0.0, 0.0]),
                accretion,
                sky,
            ],
        }
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        &self.textures[id.as_index()]
    }
}