use crate::geometry::manifold::{tangent_space, ChartWorld, TangentWorld};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, ThreeVector};


// photon in spacetime with manifold coordinate system
#[derive(Clone, Copy)]
pub struct Photon4 {
    pub pos: Point4,
    pub vel: FourVector,
}

impl Photon4 {
    pub fn new(pos: Point4, vel: FourVector) -> Self {
        debug_assert!(tangent_space(pos.chart) ==  vel.vector_space);

        Self {
            pos: pos,
            vel: vel,
        }
    }
}


// photon in a fixed-time r^4 submanifold: R^3_t, expressed in the world chart.
#[derive(Clone, Copy)]
pub struct Photon3 {
    pub pos: Point3<ChartWorld>,
    pub vel: ThreeVector<TangentWorld>,
}

impl Photon3 {
    pub fn new(pos: Point3<ChartWorld>, vel: ThreeVector<TangentWorld>) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }
}
