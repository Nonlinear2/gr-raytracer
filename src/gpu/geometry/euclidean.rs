use crate::{geometry::euclidean::Euclidean4Manifold, gpu::geometry::manifold::GpuManifold};

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