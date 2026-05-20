use crate::geometry::{manifold::Chart, point::Point4, vector::FourVector};

const EULER_STEP_SIZE: f32 = 0.01;

pub fn euler_step(x: Point4, k: FourVector, del_x: Point4, del_k: FourVector) -> (Point4, FourVector) {
    assert!(x.chart == del_x.chart);
    assert!(k.vector_space == del_k.vector_space); // tangent basis

    match x.chart {
        Chart::Cartesian => (
            x + EULER_STEP_SIZE * del_x,
            k + EULER_STEP_SIZE * del_k,
        ),
        Chart::SphericalX | Chart::SphericalZ => {
            let mut new_x = Point4::new(
                x[0] + EULER_STEP_SIZE * del_x[0],
                x[1] + EULER_STEP_SIZE * del_x[1],
                x[2] + EULER_STEP_SIZE * del_x[2],
                x[3] + EULER_STEP_SIZE * del_x[3],
                x.chart,
            );
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
        Chart::CartesianWorld => {
            panic!();
        }
    }
}