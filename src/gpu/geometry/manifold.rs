pub trait GpuManifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)>;
    fn get_geometry_source(&self) -> String;
}