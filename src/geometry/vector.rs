use glam::{Vec3, Vec4};
use rand::{rngs::StdRng, RngExt};

use crate::geometry::chart::{Cartesian, IsChart, IsSphericalChart};
use crate::geometry::point::{Point3, Point4};

/// vector of the tangent space at a point of the chart C, expressed in the coordinate
/// basis of C (see proposition 3.2 J.Lee smooth manifolds).
#[derive(Clone, Copy, PartialEq)]
pub struct ThreeVector<C> {
    pub inner: Vec3,
    pub chart: C
}

impl<C: IsChart> ThreeVector<C> {
    pub fn zero(chart: C) -> Self {
        Self {
            inner: Vec3::new(0., 0., 0.),
            chart: chart,
        }
    }

    pub fn new(x0: f32, x1: f32, x2: f32, chart: C) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            chart: chart,
        }
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }

    pub fn as_point3(self) -> Point3<C> {
        Point3::new(self.inner[0], self.inner[1], self.inner[2], self.chart)
    }
}

impl ThreeVector<Cartesian> {
    /// uniformly distributed unit vector, expressed in an orthonormal tangent space basis
    pub fn random_unit(rng: &mut StdRng, chart: Cartesian) -> Self {
        let z: f32 = rng.random_range(-1.0..=1.0);
        let phi: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let r_xy = (1.0 - z * z).max(0.0).sqrt();
        Self::new(r_xy * phi.cos(), r_xy * phi.sin(), z, chart)
    }

    pub fn length(&self) -> f32 {
        self.inner.length()
    }

    pub fn normalize(&self) -> Self {
        debug_assert!(self.length() != 0.);
        Self { inner: self.inner.normalize(), chart: self.chart }
    }

    pub fn dot(&self, other: Self) -> f32 {
        debug_assert!(self.chart == other.chart);
        self.inner.dot(other.inner)
    }

    pub fn x(&self) -> f32 {
        debug_assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn y(&self) -> f32 {
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn z(&self) -> f32 {
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }
}

// here the arguments are the components along the basis e_r, e_theta, e_phi of the tangent space.
// So r, theta, phi can be anything (negative, outside of -pi, pi, ...)

impl<C: IsSphericalChart> ThreeVector<C> {
    pub fn new_spherical(r: f32, theta: f32, phi: f32, chart: C) -> Self {
        Self {
            inner: Vec3::new(r, theta, phi),
            chart: chart,
        }
    }

    pub fn r(&self) -> f32 {
        debug_assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32 {
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32 {
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }
}

impl<C> std::ops::Index<usize> for ThreeVector<C> {
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

impl<C> std::ops::IndexMut<usize> for ThreeVector<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            _ => panic!("ThreeVector index out of bounds: {}", index),
        }
    }
}

impl<C: IsChart> std::ops::Neg for ThreeVector<C> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner.x, -self.inner.y, -self.inner.z, self.chart)
    }
}

impl<C: IsChart> std::ops::Add for ThreeVector<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.chart == rhs.chart);
        Self::new(
            self.inner.x + rhs.inner.x,
            self.inner.y + rhs.inner.y,
            self.inner.z + rhs.inner.z,
            self.chart,
        )
    }
}

impl<C: IsChart> std::ops::Sub for ThreeVector<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(self.chart == rhs.chart);
        Self::new(
            self.inner.x - rhs.inner.x,
            self.inner.y - rhs.inner.y,
            self.inner.z - rhs.inner.z,
            self.chart,
        )
    }
}

impl<C: IsChart> std::ops::Mul<f32> for ThreeVector<C> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.inner.x * rhs,
            self.inner.y * rhs,
            self.inner.z * rhs,
            self.chart,
        )
    }
}

impl<C: IsChart> std::ops::Mul<ThreeVector<C>> for f32 {
    type Output = ThreeVector<C>;

    fn mul(self, rhs: ThreeVector<C>) -> Self::Output {
        rhs * self
    }
}

/// vector of the tangent space at a point of the spacetime chart obtained by taking
/// the product of the time coordinate with the chart C.
#[derive(Clone, Copy, PartialEq)]
pub struct FourVector<C> {
    pub inner: Vec4,
    pub chart: C,
}

impl<C: IsChart> FourVector<C> {
    pub fn zero(chart: C) -> Self {
        Self {
            inner: Vec4::new(0., 0., 0., 0.),
            chart: chart,
        }
    }

    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32, chart: C) -> Self {
        Self {
            inner: Vec4::new(x0, x1, x2, x3),
            chart: chart,
        }
    }

    pub fn from_space_time(time: f32, space: ThreeVector<C>) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            chart: space.chart,
        }
    }

    pub fn t(&self) -> f32 {
        debug_assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn space(&self) -> ThreeVector<C> {
        ThreeVector::new(self.inner[1], self.inner[2], self.inner[3], self.chart)
    }

    pub fn as_point4(self) -> Point4<C> {
        Point4::new(self.inner[0], self.inner[1], self.inner[2], self.inner[3], self.chart)
    }

    pub fn as_vec4(self) -> Vec4 { self.inner }
}

impl FourVector<Cartesian> {
    pub fn x(&self) -> f32 {
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn y(&self) -> f32 {
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn z(&self) -> f32 {
        debug_assert!(self.inner[3].is_finite());
        self.inner[3]
    }
}

// here the arguments are the components along the basis e_t, e_r, e_theta, e_phi of the tangent space.
// So r, theta, phi can be anything (negative, outside of -pi, pi, ...)

impl<C: IsSphericalChart> FourVector<C> {
    pub fn new_spherical(t: f32, r: f32, theta: f32, phi: f32, chart: C) -> Self {
        Self {
            inner: Vec4::new(t, r, theta, phi),
            chart: chart,
        }
    }

    pub fn r(&self) -> f32 {
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        debug_assert!(self.inner[3].is_finite());
        self.inner[3]
    }
}

impl<C> std::ops::Index<usize> for FourVector<C> {
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

impl<C> std::ops::IndexMut<usize> for FourVector<C> {
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

impl<C: IsChart> std::ops::Add for FourVector<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.chart == rhs.chart);
        Self {
            inner: self.inner + rhs.inner,
            chart: self.chart,
        }
    }
}

impl<C: IsChart> std::ops::Mul<f32> for FourVector<C> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            chart: self.chart,
        }
    }
}

impl<C: IsChart> std::ops::Mul<FourVector<C>> for f32 {
    type Output = FourVector<C>;

    fn mul(self, rhs: FourVector<C>) -> Self::Output {
        rhs * self
    }
}
