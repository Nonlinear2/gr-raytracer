use crate::geometry::point::{Point3, Point4};
use crate::geometry::vector::{FourVector, ThreeVector};

/// a chart of the atlas of the fixed time submanifolds M_t
pub trait IsChart: Copy + PartialEq {}

/// charts whose image is described by coordinates (r, theta, phi)
pub trait IsSphericalChart: IsChart {}

/// coordinates (x, y, z) with center
#[derive(Clone, Copy, PartialEq)]
pub struct Cartesian;
impl IsChart for Cartesian {}

/// spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z
#[derive(Clone, Copy, PartialEq)]
pub struct SphericalZ;
impl IsChart for SphericalZ {}
impl IsSphericalChart for SphericalZ {}

/// spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X
#[derive(Clone, Copy, PartialEq)]
pub struct SphericalX;
impl IsChart for SphericalX {}
impl IsSphericalChart for SphericalX {}


/// transition map from chart A to chart B, restricted to the intersection of their domains.
/// transition maps do not depend on the metric, so they are shared between manifolds.
pub trait Transition<A: IsChart, B: IsChart> {
    /// tells is p (given in A-coordinates) inside the domain of chart B
    fn intersects(&self, p: Point3<A>) -> bool;
    fn transition_point(&self, p: Point3<A>) -> Point3<B>;
    fn transition_vector(&self, p: Point3<A>, v: ThreeVector<A>) -> ThreeVector<B>;

    /// charts of M are extended to charts of M by taking their product with the time coordinate,
    /// which the transition leaves untouched.
    fn point4(&self, p: Point4<A>) -> Point4<B> {
        Point4::from_space_time(p.t(), self.transition_point(p.space()))
    }

    fn vector4(&self, p: Point4<A>, v: FourVector<A>) -> FourVector<B> {
        FourVector::from_space_time(v.t(), self.transition_vector(p.space(), v.space()))
    }
}
