use crate::graphics::{surface::Surface};

pub struct World {
    // pub metric: Box<dyn Metric>,
    pub objects: Vec<Box<dyn Surface>>,   
}