use crate::graphics::{surface::Surface};
use crate::relativity::metric::PseudoRiemanianManifold;

pub struct World {
    pub metric: Box<dyn PseudoRiemanianManifold>,
    pub objects: Vec<Box<dyn Surface>>,   
}