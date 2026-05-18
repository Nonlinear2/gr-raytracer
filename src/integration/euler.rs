use crate::graphics::vector::{CoordinateSystem, FourVector};

const EULER_STEP_SIZE: f32 = 0.01;

pub fn euler_step(x: FourVector, k: FourVector, del_x: FourVector, del_k: FourVector) -> (FourVector, FourVector) {
    assert!(x.coordinate_system == k.coordinate_system);
    assert!(x.coordinate_system == del_x.coordinate_system);
    assert!(x.coordinate_system == del_k.coordinate_system);

    match x.coordinate_system {
        CoordinateSystem::Cartesian => (
            x + EULER_STEP_SIZE * del_x,
            k + EULER_STEP_SIZE * del_k,
        ),
        CoordinateSystem::Spherical => {
            let mut new_x = x + EULER_STEP_SIZE * del_x;
            let mut new_k = k + EULER_STEP_SIZE * del_k;

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

            (new_x, new_k)
        }
    }
}