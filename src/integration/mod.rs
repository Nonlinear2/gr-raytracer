pub mod euler;
pub mod rk4;

use crate::config;
use crate::constants::IntegrationMethod;
use crate::geometry::manifold::Chart;
use crate::geometry::photon::{Photon4, PhotonDerivative};
use crate::geometry::point::Point4;

pub fn step(photon: Photon4, derivative: impl Fn(Photon4) -> PhotonDerivative) -> Photon4 {
    match config::INTEGRATION_METHOD {
        IntegrationMethod::EULER => euler::step(photon, derivative),
        IntegrationMethod::RK4 => rk4::step(photon, derivative),
    }
}

pub fn advance(photon: Photon4, derivative: PhotonDerivative, h: f32) -> Photon4 {
    let new_pos = Point4::new(
        photon.pos[0] + h * derivative.d_pos[0],
        photon.pos[1] + h * derivative.d_pos[1],
        photon.pos[2] + h * derivative.d_pos[2],
        photon.pos[3] + h * derivative.d_pos[3],
        photon.pos.chart,
    );

    Photon4::new(new_pos, photon.vel + h * derivative.d_vel)
}

pub fn wrap_photon(photon: Photon4) -> Photon4 {
    match photon.pos.chart {
        Chart::Cartesian => photon,
        Chart::SphericalX | Chart::SphericalZ => {
            let mut new_x = photon.pos;
            let mut new_k = photon.vel;

            let mut theta = new_x[2];
            let mut phi   = new_x[3];

            let mut k_theta = new_k[2];

            if new_x[1] < 0.0 {
                panic!("r negative after numerical integration step");
            }

            if theta < 0.0 {
                theta = -theta;
                k_theta = -k_theta;
                phi += std::f32::consts::PI;
            }

            if theta > std::f32::consts::PI {
                theta = std::f32::consts::TAU - theta;
                k_theta = -k_theta;
                phi += std::f32::consts::PI;
            }

            new_x[2] = theta.clamp(0.0, std::f32::consts::PI);
            new_x[3] = phi.rem_euclid(std::f32::consts::TAU);

            new_k[2] = k_theta;

            Photon4::new(new_x, new_k)
        }
    }
}
