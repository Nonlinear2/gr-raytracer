use crate::graphics::{surface::Surface};
use crate::geometry::manifold::PseudoRiemanian4Manifold;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,   
}