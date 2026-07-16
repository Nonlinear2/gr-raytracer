use bytemuck::{Pod, Zeroable};

use crate::geometry::{manifold::{ChartWorld, TangentWorld}, photon::Photon3, point::Point3, vector::ThreeVector};

// gpu tags of the CartesianWorld chart and tangent space (see main.wgsl)
pub const CHART_CARTESIAN_WORLD: u32 = 0;
pub const TANGENT_CARTESIAN_WORLD: u32 = 0;

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
            pos_chart: CHART_CARTESIAN_WORLD,
            vel: [photon.vel.x(), photon.vel.y(), photon.vel.z()],
            vel_space: TANGENT_CARTESIAN_WORLD,
        }
    }
}

impl From<PackedPhoton3> for Photon3 {
    fn from(photon: PackedPhoton3) -> Self {
        assert!(photon.pos_chart == CHART_CARTESIAN_WORLD);
        assert!(photon.vel_space == TANGENT_CARTESIAN_WORLD);

        Photon3::new(
            Point3::new(
                photon.pos[0],
                photon.pos[1],
                photon.pos[2],
                ChartWorld,
            ),
            ThreeVector::new(
                photon.vel[0],
                photon.vel[1],
                photon.vel[2],
                TangentWorld,
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
