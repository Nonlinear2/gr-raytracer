use crate::graphics::{surface::Surface};

pub struct World {
    pub objects: Vec<Box<dyn Surface>>,    
}