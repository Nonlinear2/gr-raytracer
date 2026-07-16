use num_enum::TryFromPrimitive;
use glam::{Vec3, Vec4};
use rand::{rngs::StdRng, RngExt};

use crate::geometry::manifold::{ChartWorld, Chart, TangentWorld};
use crate::geometry::point::{Point3, Point4};

/// basis of the tangent space at a point (unspecified) of a given chart on R^3_t (that is, a coordinate system).
/// the tangent space identified with R^3_t thus the basis is composed of vectors.
/// (see proposition 3.2 J.Lee smooth manifolds).
/// the world tangent space is tracked at the type level instead (ThreeVector<TangentWorld>).
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
pub enum TangentSpace {
    Cartesian = 1,
    SphericalZ = 2,
    SphericalX = 3,
}

/// vector of a given tangent space.
/// this struct assumes that the tangent space basis vectors are orthonormal.
/// This is the case for cartesian and spherical tangent spaces.
#[derive(Clone, Copy, PartialEq)]
pub struct ThreeVector<S = TangentSpace> {
    pub inner: Vec3,
    pub vector_space: S
}

impl<S> ThreeVector<S> {
    pub fn zero(space: S) -> Self {
        Self {
            inner: Vec3::new(0., 0., 0.),
            vector_space: space,
        }
    }

    pub fn new(x0: f32, x1: f32, x2: f32, space: S) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            vector_space: space,
        }
    }

    /// uniformly distributed unit vector, expressed in an orthonormal tangent space basis
    pub fn random_unit(rng: &mut StdRng, space: S) -> Self {
        let z: f32 = rng.random_range(-1.0..=1.0);
        let phi: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let r_xy = (1.0 - z * z).max(0.0).sqrt();
        Self::new(r_xy * phi.cos(), r_xy * phi.sin(), z, space)
    }

    pub fn length(&self) -> f32 {
        self.inner.length()
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

impl<S: Copy> ThreeVector<S> {
    pub fn normalize(&self) -> ThreeVector<S> {
        assert!(self.length() != 0.);
        ThreeVector { inner: self.inner.normalize(), vector_space: self.vector_space }
    }
}

impl ThreeVector<TangentWorld> {
    pub fn x(&self) -> f32 {
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn y(&self) -> f32 {
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn z(&self) -> f32 {
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn dot(&self, other: ThreeVector<TangentWorld>) -> f32 {
        self.inner.dot(other.inner)
    }

    pub fn as_point3(self) -> Point3<ChartWorld> {
        Point3::new(self.inner[0], self.inner[1], self.inner[2], ChartWorld)
    }
}

impl ThreeVector {
    // here the arguments are the components along the basis e_r, e_theta, e_phi of the tangent space.
    // So r, theta, phi can be anything (negative, outside of -pi, pi, ...)

    pub fn new_spherical_z(r: f32, theta: f32, phi: f32) -> Self {
        Self {
            inner: Vec3::new(r, theta, phi),
            vector_space: TangentSpace::SphericalZ,
        }
    }

    pub fn new_spherical_x(r: f32, theta: f32, phi: f32) -> Self {
        Self {
            inner: Vec3::new(r, theta, phi),
            vector_space: TangentSpace::SphericalX,
        }
    }

    pub fn x(&self) -> f32{
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn y(&self) -> f32{
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn z(&self) -> f32{
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn r(&self) -> f32{
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32{
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32{
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn dot(&self, other: ThreeVector) -> f32 {
        assert!(self.vector_space == other.vector_space);
        assert!(self.vector_space == TangentSpace::Cartesian);
        self.inner.dot(other.inner)
    }

    pub fn as_point3(self) -> Point3 {
        match self.vector_space {
            TangentSpace::Cartesian => Point3::new(
                self.inner[0], self.inner[1], self.inner[2], Chart::Cartesian
            ),
            TangentSpace::SphericalZ => Point3::new(
                self.inner[0], self.inner[1], self.inner[2], Chart::SphericalZ
            ),
            TangentSpace::SphericalX => Point3::new(
                self.inner[0], self.inner[1], self.inner[2], Chart::SphericalX
            )
        }
    }
}

impl<S> std::ops::Index<usize> for ThreeVector<S> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            _ => panic!("ThreeVector index out of bounds: {}", index),
        }
    }
}

impl<S> std::ops::IndexMut<usize> for ThreeVector<S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            _ => panic!("ThreeVector index out of bounds: {}", index),
        }
    }
}

impl<S: Copy> std::ops::Neg for ThreeVector<S> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner.x, -self.inner.y, -self.inner.z, self.vector_space)
    }
}

impl<S: Copy + PartialEq> std::ops::Add for ThreeVector<S> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.vector_space == rhs.vector_space);
        Self::new(
            self.inner.x + rhs.inner.x,
            self.inner.y + rhs.inner.y,
            self.inner.z + rhs.inner.z,
            self.vector_space,
        )
    }
}

impl<S: Copy + PartialEq> std::ops::Sub for ThreeVector<S> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.vector_space == rhs.vector_space);
        Self::new(
            self.inner.x - rhs.inner.x,
            self.inner.y - rhs.inner.y,
            self.inner.z - rhs.inner.z,
            self.vector_space,
        )
    }
}

impl<S: Copy> std::ops::Mul<f32> for ThreeVector<S> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.inner.x * rhs,
            self.inner.y * rhs,
            self.inner.z * rhs,
            self.vector_space,
        )
    }
}

impl<S: Copy> std::ops::Mul<ThreeVector<S>> for f32 {
    type Output = ThreeVector<S>;

    fn mul(self, rhs: ThreeVector<S>) -> Self::Output {
        rhs * self
    }
}

/// vector of a given tangent space.
/// this struct assumes that the tangent space basis vectors are orthonormal.
/// This is the case for cartesian and spherical tangent spaces.
#[derive(Clone, Copy, PartialEq)]
pub struct FourVector {
    pub inner: Vec4,
    pub vector_space: TangentSpace,
}

impl FourVector {

    pub fn zero(space: TangentSpace) -> Self {
        Self {
            inner: Vec4::new(0., 0., 0., 0.),
            vector_space: space,
        }
    }

    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32, space: TangentSpace) -> Self {
        Self {
            inner: Vec4::new(x0, x1, x2, x3),
            vector_space: space,
        }
    }

    // here the arguments are the components along the basis e_t, e_r, e_theta, e_phi of the tangent space.
    // So r, theta, phi can be anything (negative, outside of -pi, pi, ...)

    pub fn new_spherical_z(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        Self {
            inner: Vec4::new(t, r, theta, phi),
            vector_space: TangentSpace::SphericalZ,
        }
    }

    pub fn new_spherical_x(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        Self {
            inner: Vec4::new(t, r, theta, phi),
            vector_space: TangentSpace::SphericalX,
        }
    }

    pub fn from_space_time(time: f32, space: ThreeVector) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            vector_space: space.vector_space
        }
    }

    pub fn t(&self) -> f32 {
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn x(&self) -> f32 {
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn y(&self) -> f32 {
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn z(&self) -> f32 {
        assert!(self.vector_space == TangentSpace::Cartesian);
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }

    pub fn r(&self) -> f32 {
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        assert!(matches!(self.vector_space, TangentSpace::SphericalZ | TangentSpace::SphericalX));
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }

    pub fn space(&self) -> ThreeVector {
        ThreeVector::new(self.inner[1], self.inner[2], self.inner[3], self.vector_space)
    }

    pub fn as_point4(self) -> Point4 {
        match self.vector_space {
            TangentSpace::Cartesian => Point4::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], Chart::Cartesian
            ),
            TangentSpace::SphericalZ => Point4::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], Chart::SphericalZ
            ),
            TangentSpace::SphericalX => Point4::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], Chart::SphericalX
            )
        }
    }

    pub fn as_vec4(self) -> Vec4 { self.inner }
}

impl std::ops::Index<usize> for FourVector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            3 => &self.inner.w,
            _ => panic!("FourVector index out of bounds: {}", index),
        }
    }
}

impl std::ops::IndexMut<usize> for FourVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            3 => &mut self.inner.w,
            _ => panic!("FourVector index out of bounds: {}", index),
        }
    }
}

impl std::ops::Add for FourVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.vector_space == rhs.vector_space);
        Self {
            inner: self.inner + rhs.inner,
            vector_space: self.vector_space,
        }
    }
}

impl std::ops::Mul<f32> for FourVector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            vector_space: self.vector_space,
        }
    }
}

impl std::ops::Mul<FourVector> for f32 {
    type Output = FourVector;

    fn mul(self, rhs: FourVector) -> Self::Output {
        rhs * self
    }
}
