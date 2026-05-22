use crate::geometry::manifold::tangent_space;
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{ThreeVector, FourVector};
use crate::geometry::surface::Material;

#[derive(Clone, Copy)]
pub enum StopReason {
    MaxStepsReached,
    BackgroundReached,
    ObjectHit,
    HorizonHit,
}

pub struct WorldPhoton3State<'a> {
    pub photon3: Photon3,
    #[allow(dead_code)]
    pub normal: Option<ThreeVector>,
    pub material: Option<&'a dyn Material>
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