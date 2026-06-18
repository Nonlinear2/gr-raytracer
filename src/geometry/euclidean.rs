use crate::geometry::manifold::PseudoRiemanian4Manifold;
use crate::geometry::point::Point3;
#[allow(dead_code)]
pub struct Euclidean4Manifold {
    pub subatlas_center: Point3 // center of the atlas for fixed-time submanifolds expressed in Chart::CartesianWorld 
}

impl PseudoRiemanian4Manifold for Euclidean4Manifold {
    fn geometry_parameters(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("SUBATLAS_CENTER_X", self.subatlas_center.x() as f64),
            ("SUBATLAS_CENTER_Y", self.subatlas_center.y() as f64),
            ("SUBATLAS_CENTER_Z", self.subatlas_center.z() as f64),
        ]
    }
}