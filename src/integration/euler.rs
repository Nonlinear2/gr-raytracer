use crate::config;
use crate::geometry::photon::{Photon4, PhotonDerivative};
use crate::integration::{advance, wrap_photon};

pub fn step(photon: Photon4, derivative: impl Fn(Photon4) -> PhotonDerivative) -> Photon4 {
    wrap_photon(advance(photon, derivative(photon), config::INTEGRATION_STEP_SIZE))
}