use crate::graphics::ray::{Photon, WorldPhoton};
use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{TangentSpace, ThreeVector, FourVector};
use crate::integration::euler;

use crate::integration::solvers::positive_root;
use glam::{Vec4, Mat4};

const SPH_EPS: f32 = 1e-3;
const EPS: f32 = 1e-5;

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

// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted).
pub trait HasAtlas3 {
    #[allow(dead_code)]
    fn has_chart(&self, chart: Chart) -> bool; // should always have CartesianWorld
    fn preferred_chart_for_point(&self, point: Point3) -> Chart;
    fn transition_point(&self, p: Point3, to: Chart) -> Point3;
    fn transition_vector(&self, p: Point3, v: ThreeVector, to: Chart) -> ThreeVector;
}

pub trait PseudoRiemanian4Manifold {

    fn is_singular(&self, x: Point4) -> bool;

    fn world_to_photon(&self, world_photon: WorldPhoton) -> Photon;

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton;

    fn g(&self, x: Point4) -> Mat4;

    fn g_inv(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon) -> Photon;
}