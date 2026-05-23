use num_enum::TryFromPrimitive;

use crate::geometry::manifold::{Chart, tangent_space};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, TangentSpace, ThreeVector};

use bytemuck::{Pod, Zeroable};

#[repr(u32)]
#[derive(Clone, Copy, TryFromPrimitive)]
pub enum StopReason {
    MaxStepsReached,
    BackgroundReached,
    ObjectHit,
    HorizonHit,
}

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

// photon in spacetime with manifold coordinate system
#[derive(Clone, Copy)]
pub struct Photon4 {
    pub pos: Point4,
    pub vel: FourVector,
}

impl Photon4 {
    pub fn new(pos: Point4, vel: FourVector) -> Self {
        assert!(tangent_space(pos.chart) ==  vel.vector_space);

        Self {
            pos: pos,
            vel: vel,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedPhoton4 {
    pub pos: [f32; 4],
    pub pos_chart: u32,
    pub pos_padding: [u32; 3],
    pub vel: [f32; 4],
    pub vel_space: u32,
    pub vel_padding: [u32; 3],
}


impl From<Photon4> for PackedPhoton4 {
    fn from(photon: Photon4) -> Self {
        Self {
            pos: [photon.pos.t(), photon.pos.r(), photon.pos.theta(), photon.pos.phi()],
            pos_chart: photon.pos.chart as u32,
            pos_padding: [0; 3],
            vel: [photon.vel.t(), photon.vel.r(), photon.vel.theta(), photon.vel.phi()],
            vel_space: photon.vel.vector_space as u32,
            vel_padding: [0; 3],
        }
    }
}

impl From<PackedPhoton4> for Photon4 {
    fn from(photon: PackedPhoton4) -> Self {
        Photon4::new(
            crate::geometry::point::Point4::new(
                photon.pos[0],
                photon.pos[1],
                photon.pos[2],
                photon.pos[3],
                Chart::try_from(photon.pos_chart).unwrap(),
            ),
            FourVector::new(
                photon.vel[0],
                photon.vel[1],
                photon.vel[2],
                photon.vel[3],
                TangentSpace::try_from(photon.vel_space).unwrap(),
            ),
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedRayResult {
    pub photon: PackedPhoton4,
    pub stop_reason: u32,
    pub padding: [u32; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedColorResult {
    pub color: [f32; 4],
}