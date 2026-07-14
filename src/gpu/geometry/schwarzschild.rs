use crate::{geometry::schwarzschild::Schwarzschild4Manifold, gpu::geometry::manifold::GpuManifold};

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