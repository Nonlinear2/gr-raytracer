use crate::config;
use crate::geometry::photon::{Photon4, PhotonDerivative};
use crate::integration::{advance, wrap_photon};

pub fn step(photon: Photon4, derivative: impl Fn(Photon4) -> PhotonDerivative) -> Photon4 {
    // k1 = derivative(photon)
    // k2 = derivative(photon + k1*h/2)
    // k3 = derivative(photon + k2*h/2)
    // k4 = derivative(photon + k3*h)
    // new_photon = photon + h * (k1 + 2*k2 + 2*k3 + k4) / 6

    const STEP: f32 = config::INTEGRATION_STEP_SIZE;

    let k1 = derivative(photon);
    let k2 = derivative(wrap_photon(advance(photon, k1, STEP / 2.0)));
    let k3 = derivative(wrap_photon(advance(photon, k2, STEP / 2.0)));
    let k4 = derivative(wrap_photon(advance(photon, k3, STEP)));

    let combined = PhotonDerivative {
        d_pos: (k1.d_pos + 2.0 * k2.d_pos + 2.0 * k3.d_pos + k4.d_pos) * (1.0 / 6.0),
        d_vel: (k1.d_vel + 2.0 * k2.d_vel + 2.0 * k3.d_vel + k4.d_vel) * (1.0 / 6.0),
    };

    wrap_photon(advance(photon, combined, STEP))
}
