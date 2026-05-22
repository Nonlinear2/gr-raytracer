use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{TangentSpace, ThreeVector};
use crate::geometry::photon::{Photon4, Photon3};
use glam::Mat4;

/// which global chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
/// Important points: 
/// These charts will designate the maps from coordinates to "manifold" and not the opposite. They are technically inverse charts
#[derive(Clone, Copy, PartialEq)]
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

// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted).
pub trait HasAtlas3 {
    #[allow(dead_code)]
    fn has_chart(&self, chart: Chart) -> bool; // should always have CartesianWorld
    fn preferred_chart_for_point(&self, point: Point3) -> Chart;
    fn transition_point(&self, p: Point3, to: Chart) -> Point3;
    fn transition_vector(&self, p: Point3, v: ThreeVector, to: Chart) -> ThreeVector;
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

    fn get_shader(&self) -> String;
}

