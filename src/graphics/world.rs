use crate::graphics::{surface::Surface};
use crate::relativity::metric::PseudoRiemanianManifold;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanianManifold>,
    pub objects: Objects,   
}