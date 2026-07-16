pub mod surface;
pub mod texture;

use crate::geometry::manifold::Manifold;
use crate::scene::surface::Object;
use crate::scene::texture::Textures;

pub type Objects = Vec<Box<dyn Object>>;

pub struct World {
    pub scene_size: f32,
    pub manifold: Box<dyn Manifold>,
    pub objects: Objects,
    pub textures: Textures,
}
