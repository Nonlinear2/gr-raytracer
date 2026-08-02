use glam::{Mat4, Vec4};

use crate::geometry::chart::Cartesian;
use crate::geometry::manifold::{GpuManifold, PseudoRiemanian4Manifold};
use crate::geometry::metric::Metric;
use crate::geometry::photon::{Photon4, PhotonDerivative};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::FourVector;

pub struct EuclideanAtlas {
    pub center: Point3<Cartesian>,
}

impl EuclideanAtlas {
    pub fn new(center: Point3<Cartesian>) -> Self {
        Self { center }
    }
}

// no transitions as EuclideanAtlas has a single chart

#[allow(dead_code)]
pub struct Euclidean4Manifold {
    pub atlas: EuclideanAtlas,
}

#[allow(dead_code)]
impl Euclidean4Manifold {
    pub fn new(center: Point3<Cartesian>) -> Self {
        Self {
            atlas: EuclideanAtlas::new(center),
        }
    }
}

impl Metric<Cartesian> for Euclidean4Manifold {

    fn g(&self, _x: Point4<Cartesian>) -> Mat4 {
        Mat4 {
            x_axis: Vec4::new(1., 0., 0., 0.),
            y_axis: Vec4::new(0., -1., 0., 0.),
            z_axis: Vec4::new(0., 0., -1., 0.),
            w_axis: Vec4::new(0., 0., 0., -1.),
        }
    }

    fn g_inv(&self, _x: Point4<Cartesian>) -> Mat4 {
        self.g(_x)
    }

    fn del_g(&self, _x: Point4<Cartesian>, _i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, _pos: Point4<Cartesian>, _mu: usize, _nu: usize, _lambda: usize) -> f32 {
        0.
    }
}

impl PseudoRiemanian4Manifold<Cartesian> for Euclidean4Manifold {

    fn is_close_to_singular(&self, _x: Point4<Cartesian>) -> bool {
        false
    }

    fn geodesic_derivative(&self, photon: Photon4<Cartesian>) -> PhotonDerivative<Cartesian> {
        PhotonDerivative {
            d_pos: photon.vel,
            d_vel: FourVector::zero(photon.vel.chart),
        }
    }
}

impl GpuManifold for Euclidean4Manifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("SUBATLAS_CENTER_X", self.atlas.center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.atlas.center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.atlas.center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("euclidean.wgsl").to_string()
    }
}
