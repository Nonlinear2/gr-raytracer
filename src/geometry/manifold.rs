use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{TangentSpace, ThreeVector};
use crate::geometry::photon::{Photon4, Photon3};

use glam::Mat4;
use num_enum::{TryFromPrimitive};

/// which global chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
/// NOTE:
/// These charts will designate the maps from coordinates to "manifold" and not the opposite. They are technically inverse charts
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
pub enum Chart {
    Cartesian = 1, // cartesian with center point
    SphericalZ = 2, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z
    SphericalX = 3, // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X
}

/// the background cartesian chart of the fixed time submanifolds, in which the scene is described
/// it is tracked at the type level (Point3<ChartWorld>) rather than in the Chart enum
#[derive(Clone, Copy, PartialEq)]
pub struct ChartWorld;

/// basis of the tangent space corresponding to the ChartWorld chart,
/// tracked at the type level (ThreeVector<TangentWorld>) rather than in the TangentSpace enum
#[derive(Clone, Copy, PartialEq)]
pub struct TangentWorld;

pub fn tangent_space(chart: Chart) -> TangentSpace {
    match chart {
        Chart::Cartesian => TangentSpace::Cartesian,
        Chart::SphericalZ => TangentSpace::SphericalZ,
        Chart::SphericalX => TangentSpace::SphericalX,
    }
}

// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted)
// The CartesianWorld chart is always part of the atlas and is not listed in Chart
pub trait HasAtlas3 {
    fn has_chart(&self, chart: Chart) -> bool;
    fn subatlas_center(&self) -> Point3<ChartWorld>;
    fn preferred_chart_for_point(&self, point: Point3<ChartWorld>) -> Chart;
    fn point_to_world(&self, p: Point3) -> Point3<ChartWorld>;
    fn point_from_world(&self, p: Point3<ChartWorld>, to: Chart) -> Point3;
    fn vector_to_world(&self, p: Point3, v: ThreeVector) -> ThreeVector<TangentWorld>;
    fn vector_from_world(&self, p: Point3<ChartWorld>, v: ThreeVector<TangentWorld>, to: Chart) -> ThreeVector;
}

pub trait PseudoRiemanian4Manifold: HasAtlas3 {

    fn is_singular(&self, x: Point4) -> bool;

    fn world_photon3_to_photon4(&self, world_photon: Photon3) -> Photon4;

    fn to_world_photon3(&self, photon: Photon4) -> Photon3;

    fn g(&self, x: Point4) -> Mat4;

    fn g_inv(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon4) -> Photon4;
}

pub trait GpuManifold {
    fn get_constants(&self) -> Vec<(&'static str, f64)>;
    fn get_geometry_source(&self) -> String;
}

pub trait Manifold: PseudoRiemanian4Manifold + GpuManifold {}

impl<T: PseudoRiemanian4Manifold + GpuManifold> Manifold for T {}
