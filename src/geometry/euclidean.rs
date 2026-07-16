use crate::geometry::manifold::{tangent_space, ChartWorld, Chart, GpuManifold, HasAtlas3, PseudoRiemanian4Manifold, TangentWorld};
use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, TangentSpace, ThreeVector};
use crate::integration::euler;
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

    fn point_to_world(&self, p: Point3) -> Point3<ChartWorld> {
        assert!(p.chart == Chart::Cartesian);
        Point3::new(p[0], p[1], p[2], ChartWorld) + self.subatlas_center
    }

    fn point_from_world(&self, p: Point3<ChartWorld>, to: Chart) -> Point3 {
        assert!(to == Chart::Cartesian);
        let rel = p - self.subatlas_center;
        Point3::new(rel.x(), rel.y(), rel.z(), Chart::Cartesian)
    }

    fn vector_to_world(&self, _p: Point3, v: ThreeVector) -> ThreeVector<TangentWorld> {
        assert!(v.vector_space == TangentSpace::Cartesian);
        ThreeVector::new(v[0], v[1], v[2], TangentWorld)
    }

    fn vector_from_world(&self, _p: Point3<ChartWorld>, v: ThreeVector<TangentWorld>, to: Chart) -> ThreeVector {
        ThreeVector::new(v.x(), v.y(), v.z(), tangent_space(to))
    }
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {

    fn is_singular(&self, _x: Point4) -> bool {
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

    fn step_along_null_geodesic(&self, s: Photon4) -> Photon4 {
        euler::euler_step(
            s.pos, s.vel, s.vel.as_point4(), FourVector::zero(TangentSpace::Cartesian)
        )
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
