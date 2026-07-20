use glam::{Mat4, Vec4};

use crate::config;
use crate::geometry::manifold::{ChartWorld, Chart, GpuManifold, HasAtlas3, PseudoRiemanian4Manifold};
use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::FourVector;
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
    fn is_close_to_singular(&self, x: Point4) -> bool {
        x.r() <= self.r_s + config::INTEGRATION_STEP_SIZE
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
        debug_assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));
        // the entries of g are the same regardless of if the spherical coordinates are centered on the X or Z axis,
        // because the schwarzschild metric is spherically symmetric. Therefore we dont need to condition on the chart.
        let r = pos.r();
        let theta = pos.theta();
        debug_assert!(r > self.r_s);

        let g = Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., -1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., -r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., -r*r*theta.sin()*theta.sin()),
        };
        
        debug_assert!(g.determinant().is_finite());

        g
    }

    fn g_inv(&self, pos: Point4) -> Mat4 {
        debug_assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));
        debug_assert!(pos.r() > self.r_s);

        let r = pos.r();
        let theta = pos.theta();
        debug_assert!(r > self.r_s);

        let g_inv = Mat4 {
            x_axis: Vec4::new(1. / (1. - self.r_s / r), 0., 0., 0.),
            y_axis: Vec4::new(0., self.r_s / r - 1., 0., 0.),
            z_axis: Vec4::new(0., 0., -1./(r*r), 0.),
            w_axis: Vec4::new(0., 0., 0., -1./(r*r*theta.sin()*theta.sin())),
        };

        g_inv
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        debug_assert!(matches!(pos.chart, Chart::SphericalX | Chart::SphericalZ));

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
