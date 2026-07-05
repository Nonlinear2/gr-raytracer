use crate::geometry::vector::TangentSpace;
use num_enum::{TryFromPrimitive};

/// which global chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
/// Important points: 
/// These charts will designate the maps from coordinates to "manifold" and not the opposite. They are technically inverse charts
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
pub enum Chart {
    CartesianWorld,
    Cartesian, // cartesian with center point
    SphericalZ, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z 
    SphericalX, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X 
}

pub fn tangent_space(chart: Chart) -> TangentSpace {
    match chart {
        Chart::Cartesian => TangentSpace::Cartesian,
        Chart::CartesianWorld => TangentSpace::CartesianWorld,
        Chart::SphericalZ => TangentSpace::SphericalZ,
        Chart::SphericalX => TangentSpace::SphericalX,
    }
}

pub trait PseudoRiemanian4Manifold {
    fn get_subatlas_center(&self) -> Vec<(&'static str, f64)>;
    fn get_geometry_source(&self) -> String;
}