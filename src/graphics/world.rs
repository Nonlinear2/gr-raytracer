use crate::graphics::{surface::Surface};
use crate::relativity::metric::Metric;

pub struct World {
    pub metric: Box<dyn Metric>,
    pub objects: Vec<Box<dyn Surface>>,   
}