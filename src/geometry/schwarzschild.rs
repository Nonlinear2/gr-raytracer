use glam::{Mat4, Vec4};

use crate::geometry::manifold::{ChartWorld, Chart, GpuManifold, HasAtlas3, PseudoRiemanian4Manifold};
use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::FourVector;
use crate::integration::euler;
use crate::math::positive_root;

pub struct Schwarzschild4Manifold {
    pub subatlas_center: Point3<ChartWorld>, // center of the atlas for fixed-time submanifolds
    pub r_s: f32,
}

impl Schwarzschild4Manifold {
    pub fn new(center: Point3<ChartWorld>, r_s: f32) -> Self {
        Self {
            subatlas_center: center,
            r_s: r_s,
        }
    }
}

impl HasAtlas3 for Schwarzschild4Manifold {
    fn has_chart(&self, chart: Chart) -> bool {
        chart == Chart::SphericalX || chart == Chart::SphericalZ
    }

    fn subatlas_center(&self) -> Point3<ChartWorld> {
        self.subatlas_center
    }

    fn preferred_chart_for_point(&self, point: Point3<ChartWorld>) -> Chart {
        let rel = point - self.subatlas_center;

        let dist_to_z_axis_sq = rel.x() * rel.x() + rel.y() * rel.y();
        let dist_to_x_axis_sq = rel.y() * rel.y() + rel.z() * rel.z();

        if dist_to_z_axis_sq < dist_to_x_axis_sq {
            Chart::SphericalX
        } else {
            Chart::SphericalZ
        }
    }
}

impl PseudoRiemanian4Manifold for Schwarzschild4Manifold {
    fn is_singular(&self, x: Point4) -> bool {
        x.r() <= self.r_s
    }

    fn world_photon3_to_photon4(&self, world_photon: Photon3) -> Photon4 {
        let chart = self.preferred_chart_for_point(world_photon.pos);

        let pos = self.point_from_world(world_photon.pos, chart);

        let vel = self.vector_from_world(world_photon.pos, world_photon.vel, chart);

        let (v_r, v_th, v_ph) = (vel.r(), vel.theta(), vel.phi());

        let photon_x= Point4::from_space_time(0.0, pos);

        // compute k^0 such that <k, k> = 0 so that the photon's trajectory be lightlike.
        // we need to solve g_mu_nu k^mu k^nu = 0 for k^0 which is a quadratic equation

        let g = self.g(photon_x);
        let b = 2.0 * (g.col(0)[1] * v_r + g.col(0)[2] * v_th + g.col(0)[3] * v_ph);
        let c = 
              g.col(1)[1] * v_r * v_r
            + 2.0 * g.col(1)[2] * v_r * v_th
            + 2.0 * g.col(1)[3] * v_r * v_ph
            + g.col(2)[2] * v_th * v_th
            + 2.0 * g.col(2)[3] * v_th * v_ph
            + g.col(3)[3] * v_ph * v_ph;

        let k_0 = positive_root(g.col(0)[0], b, c);

        Photon4::new(photon_x, FourVector::from_space_time(k_0, vel))
    }

    fn to_world_photon3(&self, photon: Photon4) -> Photon3 {
        Photon3::new(
            self.point_to_world(photon.pos.space()),
            self.vector_to_world(photon.pos.space(), photon.vel.space()),
        )
    }

    fn g(&self, pos: Point4) -> Mat4 {
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

        let r = pos.r();
        let theta = pos.theta();
        assert!(r > self.r_s);

        let g = Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., -1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., -r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., -r*r*theta.sin()*theta.sin()),
        };
        
        assert!(g.determinant().is_finite());

        g
    }

    fn g_inv(&self, pos: Point4) -> Mat4 {
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));
        assert!(pos.r() > self.r_s);

        let r = pos.r();
        let theta = pos.theta();
        assert!(r > self.r_s);

        let g_inv = Mat4 {
            x_axis: Vec4::new(1. / (1. - self.r_s / r), 0., 0., 0.),
            y_axis: Vec4::new(0., self.r_s / r - 1., 0., 0.),
            z_axis: Vec4::new(0., 0., -1./(r*r), 0.),
            w_axis: Vec4::new(0., 0., 0., -1./(r*r*theta.sin()*theta.sin())),
        };

        g_inv
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

        let r = pos.r();
        let theta = pos.theta();
        match i {
            0 => Mat4::ZERO,
            1 => Mat4 {
                x_axis: Vec4::new(self.r_s / (r*r), 0., 0., 0.),
                y_axis: Vec4::new(0., self.r_s / (r*r*(1. - self.r_s / r) * (1. - self.r_s / r)), 0., 0.),
                z_axis: Vec4::new(0., 0., -2.*r, 0.),
                w_axis: Vec4::new(0., 0., 0., -2.*r*theta.sin()*theta.sin()),
            },
            2 => Mat4 {
                x_axis: Vec4::ZERO,
                y_axis: Vec4::ZERO,
                z_axis: Vec4::ZERO,
                w_axis: Vec4::new(0., 0., 0., -2.*r*r*theta.cos()*theta.sin()),
            },
            3 => Mat4::ZERO,
            _ => unreachable!()
        }
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

        let g_inv = self.g_inv(pos);
        let mut gamma = 0.;

        let d_mu_g = self.del_g(pos, mu as u32);
        let d_nu_g = self.del_g(pos, nu as u32);

        for alpha in 0..4 {
            let d_alpha_g = self.del_g(pos, alpha as u32);

            gamma += 0.5 * g_inv.col(lambda)[alpha] * (
                d_mu_g.col(alpha)[nu]
              + d_nu_g.col(alpha)[mu]
              - d_alpha_g.col(mu)[nu]
            )
        }
        assert!(gamma.is_finite());

        gamma
    }

    fn step_along_null_geodesic(&self, photon: Photon4) -> Photon4 {
        let x = photon.pos;
        let k = photon.vel;

        let mut del_k = FourVector::zero(photon.vel.vector_space);
        for mu in 0..4 {
            for alpha in 0..4 {
                for beta in 0..4 {
                    let gamma = self.christoffel(x, alpha, beta, mu);
                    del_k[mu] -= gamma * k[alpha] * k[beta];
                }
            }
        }

        euler::euler_step(x, k, k.as_point4(), del_k)
    }
}

impl GpuManifold for Schwarzschild4Manifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("R_S", self.r_s as f64),
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("schwarzschild.wgsl").to_string()
    }
}
