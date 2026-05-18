use crate::graphics::point::{Chart, Point3, Point4};
use crate::graphics::vector::{TangentSpace, ThreeVector, FourVector};
use crate::graphics::surface::Material;

pub enum StopReason {
    MaxStepsReached,
    BackgroundReached,
    ObjectHit,
    HorizonHit,
}

pub struct WorldPhotonState<'a> {
    pub world_photon: WorldPhoton,
    #[allow(dead_code)]
    pub normal: Option<ThreeVector>,
    pub material: Option<&'a dyn Material>
}


#[derive(Clone, Copy)]
pub struct WorldPhoton {
    pub pos: Point3,
    pub vel: ThreeVector,
}

// unphysical photon in world coordinates
impl WorldPhoton {
    pub fn new(pos: Point3, vel: ThreeVector) -> Self {
        assert!(pos.chart == Chart::Cartesian);
        assert!(vel.vector_space == TangentSpace::Cartesian);

        Self {
            pos: pos,
            vel: vel,
        }
    }
}

// photon in spacetime with manifold coordinate system
#[derive(Clone, Copy)]
pub struct Photon {
    pub pos: Point4,
    pub vel: FourVector,
}

impl Photon {
    pub fn new(pos: Point4, vel: FourVector) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }
}