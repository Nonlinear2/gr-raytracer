use crate::geometry::manifold::{Chart, PseudoRiemanian4Manifold};
use crate::geometry::point::Point3;
#[allow(dead_code)]
pub struct Euclidean4Manifold {
    pub subatlas_center: Point3 // center of the atlas for fixed-time submanifolds expressed in Chart::CartesianWorld 
}

impl Euclidean4Manifold {
    // center is a Point in world space
    pub fn new(center: Point3) -> Self {
        assert!(center.chart == Chart::CartesianWorld);
        Self {
            subatlas_center: center,
        }
    }
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {
    fn get_subatlas_center(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("../geometry/euclidean.wgsl").to_string()
    }
}