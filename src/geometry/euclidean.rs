use crate::geometry::manifold::{Chart, HasAtlas3, PseudoRiemanian4Manifold};
use crate::geometry::photon::{Photon4, Photon3};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, TangentSpace, ThreeVector};
use crate::integration::euler;
use glam::Mat4;

#[allow(dead_code)]
pub struct Euclidean4Manifold {
    pub subatlas_center: Point3 // center of the atlas for fixed-time submanifolds expressed in Chart::CartesianWorld 
}

impl HasAtlas3 for Euclidean4Manifold {
    fn has_chart(&self, chart: Chart) -> bool {
        chart == Chart::Cartesian || chart == Chart::CartesianWorld
    }

    fn preferred_chart_for_point(&self, _point: Point3) -> Chart {
        Chart::Cartesian
    }

    fn transition_point(&self, p: Point3, to: Chart) -> Point3 {
        match (p.chart, to) {
            (Chart::Cartesian, Chart::CartesianWorld) => p.as_chart(Chart::CartesianWorld) + self.subatlas_center,
            (Chart::CartesianWorld, Chart::Cartesian) => (p - self.subatlas_center).as_chart(Chart::Cartesian),
            _ => panic!()
        }
    }

    fn transition_vector(&self, _p: Point3, v: ThreeVector, _to: Chart) -> ThreeVector {
        v
    }
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {

    fn is_singular(&self, _x: Point4) -> bool {
        false
    }

    fn world_photon3_to_photon4(&self, world_photon: Photon3) -> Photon4 {
        Photon4::new(
            Point4::from_space_time(0., world_photon.pos),
            FourVector::from_space_time(0., world_photon.vel)
        )
    }

    fn to_world_photon3(&self, photon: Photon4) -> Photon3 {
        assert!(photon.pos.chart == Chart::Cartesian);
        assert!(photon.vel.vector_space == TangentSpace::Cartesian);

        Photon3::new(
            self.transition_point(
                photon.pos.space(),
                Chart::CartesianWorld
            ),
            self.transition_vector(
                Point3::ZERO_CART, // unused
                photon.vel.space(),
                Chart::CartesianWorld
            )
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

    fn step_along_null_geodesic(&self, s: Photon4) -> Photon4 {
        euler::euler_step(
            s.pos, s.vel, s.vel.as_point4(), FourVector::zero(TangentSpace::Cartesian)
        )
    }

    fn geometry_parameters(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }
}