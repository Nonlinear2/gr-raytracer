use crate::geometry::chart::IsChart;
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, ThreeVector};


// photon in spacetime, expressed in the spacetime chart induced by C
#[derive(Clone, Copy)]
pub struct Photon4<C> {
    pub pos: Point4<C>,
    pub vel: FourVector<C>,
}

// derivative of a photon's coordinates with respect to the affine parameter
#[derive(Clone, Copy)]
pub struct PhotonDerivative<C> {
    pub d_pos: FourVector<C>,
    pub d_vel: FourVector<C>,
}

impl<C: IsChart> Photon4<C> {
    pub fn new(pos: Point4<C>, vel: FourVector<C>) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }

    pub fn to_photon3(self) -> Photon3<C> {
        Photon3::new(self.pos.space(), self.vel.space())
    }
}


// photon in a fixed-time submanifold M_t, expressed in the chart C.
#[derive(Clone, Copy)]
pub struct Photon3<C> {
    pub pos: Point3<C>,
    pub vel: ThreeVector<C>,
}

impl<C: IsChart> Photon3<C> {
    pub fn new(pos: Point3<C>, vel: ThreeVector<C>) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }
    }
}
