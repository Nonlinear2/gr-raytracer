use crate::{config, geometry::{manifold::Chart, photon::Photon4, point::Point4, vector::FourVector}};

pub fn euler_step(x: Point4, k: FourVector, del_x: Point4, del_k: FourVector) -> Photon4 {
    assert!(x.chart == del_x.chart);
    assert!(k.vector_space == del_k.vector_space); // tangent basis
    const STEP: f32 = config::INTEGRATION_STEP_SIZE;

    match x.chart {
        Chart::Cartesian => Photon4::new(
            x + STEP * del_x,
            k + STEP * del_k,
        ),
        Chart::SphericalX | Chart::SphericalZ => {
            let mut new_x = Point4::new(
                x[0] + STEP * del_x[0],
                x[1] + STEP * del_x[1],
                x[2] + STEP * del_x[2],
                x[3] + STEP * del_x[3],
                x.chart,
            );
            let mut new_k = k + STEP * del_k;

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
        Chart::CartesianWorld => {
            panic!();
        }
    }
}