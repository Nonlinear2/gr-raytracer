pub mod cpu;
pub mod gpu;

use bytemuck::{Pod, Zeroable};

use crate::config;
use crate::geometry::photon::Photon3;
use crate::graphics::color::Color;

pub trait GeodesicIntegrator {
    fn run(&self, rays: Vec<Photon3>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>);
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedTraceResult {
    pub positions: [PackedTracePoint; config::MAX_INTEGRATION_STEPS as usize],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedTracePoint {
    pub pos: [f32; 3],
    pub fill_flag: f32,
}
