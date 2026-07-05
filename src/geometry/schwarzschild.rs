use crate::geometry::manifold::{Chart, PseudoRiemanian4Manifold};
use crate::geometry::point::Point3;

pub struct Schwarzschild4Manifold {
    pub subatlas_center: Point3, // center of the atlas for fixed-time submanifolds expressed in Chart::CartesianWorld 
    pub r_s: f32,
}

impl Schwarzschild4Manifold {
    // center is a Point in world space
    pub fn new(center: Point3, r_s: f32) -> Self {
        assert!(center.chart == Chart::CartesianWorld);
        Self {
            subatlas_center: center,
            r_s: r_s,
        }
    }
}

impl PseudoRiemanian4Manifold for Schwarzschild4Manifold {
    fn get_subatlas_center(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("R_S", self.r_s as f64),
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }

    fn get_geometry_source(&self) -> String {
        include_str!("../geometry/schwarzschild.wgsl").to_string()
    }
}