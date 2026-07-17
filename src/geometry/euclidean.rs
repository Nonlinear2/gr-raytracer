use crate::geometry::manifold::{ChartWorld, Chart, GpuManifold, HasAtlas3, PseudoRiemanian4Manifold};
use crate::geometry::photon::{Photon3, Photon4, PhotonDerivative};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::FourVector;
use glam::Mat4;

#[allow(dead_code)]
pub struct Euclidean4Manifold {
    pub subatlas_center: Point3<ChartWorld> // center of the atlas for fixed-time submanifolds
}

impl Euclidean4Manifold {
    pub fn new(center: Point3<ChartWorld>) -> Self {
        Self {
            subatlas_center: center,
        }
    }
}

impl HasAtlas3 for Euclidean4Manifold {
    fn has_chart(&self, chart: Chart) -> bool {
        chart == Chart::Cartesian
    }

    fn subatlas_center(&self) -> Point3<ChartWorld> {
        self.subatlas_center
    }

    fn preferred_chart_for_point(&self, _point: Point3<ChartWorld>) -> Chart {
        Chart::Cartesian
    }
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {

    fn is_close_to_singular(&self, _x: Point4) -> bool {
        false
    }

    fn world_photon3_to_photon4(&self, world_photon: Photon3) -> Photon4 {
        Photon4::new(
            Point4::from_space_time(0., self.point_from_world(world_photon.pos, Chart::Cartesian)),
            FourVector::from_space_time(0., self.vector_from_world(world_photon.pos, world_photon.vel, Chart::Cartesian))
        )
    }

    fn to_world_photon3(&self, photon: Photon4) -> Photon3 {
        Photon3::new(
            self.point_to_world(photon.pos.space()),
            self.vector_to_world(photon.pos.space(), photon.vel.space())
        )
    }

    fn g(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn g_inv(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn del_g(&self, _x: Point4, _i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, _pos: Point4, _mu: usize, _nu: usize, _lambda: usize) -> f32 {
        0.
    }

    fn geodesic_derivative(&self, photon: Photon4) -> PhotonDerivative {
        PhotonDerivative {
            d_pos: photon.vel,
            d_vel: FourVector::zero(photon.vel.vector_space),
        }
    }
}

impl GpuManifold for Euclidean4Manifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("euclidean.wgsl").to_string()
    }
}
