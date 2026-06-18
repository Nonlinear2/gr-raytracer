use crate::config;
use crate::geometry::manifold::{Chart, tangent_space};
use crate::geometry::point::{Point3};
use crate::geometry::vector::{TangentSpace, ThreeVector};

use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy)]
pub struct Photon3 {
    pub pos: Point3,
    pub vel: ThreeVector,
}

// photon in a fixed-time r^4 submanifold: R^3_t.
impl Photon3 {
    pub fn new(pos: Point3, vel: ThreeVector) -> Self {
        assert!(tangent_space(pos.chart) ==  vel.vector_space);

        Self {
            pos: pos,
            vel: vel,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedPhoton3 {
    pub pos: [f32; 3],
    pub pos_chart: u32,
    pub vel: [f32; 3],
    pub vel_space: u32,
}


impl From<Photon3> for PackedPhoton3 {
    fn from(photon: Photon3) -> Self {
        Self {
            pos: [photon.pos.x(), photon.pos.y(), photon.pos.z()],
            pos_chart: photon.pos.chart as u32,
            vel: [photon.vel.x(), photon.vel.y(), photon.vel.z()],
            vel_space: photon.vel.vector_space as u32,
        }
    }
}

impl From<PackedPhoton3> for Photon3 {
    fn from(photon: PackedPhoton3) -> Self {
        Photon3::new(
            crate::geometry::point::Point3::new(
                photon.pos[0],
                photon.pos[1],
                photon.pos[2],
                Chart::try_from(photon.pos_chart).unwrap(),
            ),
            ThreeVector::new(
                photon.vel[0],
                photon.vel[1],
                photon.vel[2],
                TangentSpace::try_from(photon.vel_space).unwrap(),
            ),
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedRayResult {
    pub photon: PackedPhoton3,
    pub stop_reason: u32,
    pub padding: [u32; 7],
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