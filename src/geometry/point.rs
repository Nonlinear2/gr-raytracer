use glam::{Vec3, Vec4};

use crate::geometry::chart::{Cartesian, IsChart, IsSphericalChart};
use crate::geometry::vector::{FourVector, ThreeVector};

// Point3 is a point on the manifold R^3
#[derive(Clone, Copy, PartialEq)]
pub struct Point3<C> {
    pub inner: Vec3, // this is not a vector, here we are really differentiating between points
    // which live on the manifold R^3, and vectors which live in a tangent vector space to a point.
    pub chart: C
}

impl<C: IsChart> Point3<C> {
    pub fn new(x0: f32, x1: f32, x2: f32, chart: C) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            chart: chart,
        }
    }

    pub fn zero(chart: C) -> Self {
        Self {
            inner: Vec3::new(0., 0., 0.),
            chart: chart,
        }
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }

    pub fn as_threevector(self) -> ThreeVector<C> {
        ThreeVector::new(self.inner[0], self.inner[1], self.inner[2], self.chart)
    }
}

impl Point3<Cartesian> {
    pub fn distance_to_zero(&self) -> f32 {
        self.inner.length()
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

impl<C: IsSphericalChart> Point3<C> {
    pub fn new_spherical(r: f32, theta: f32, phi: f32, chart: C) -> Self {
        debug_assert!(r >= 0.0);
        debug_assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            chart: chart,
        }
    }

    pub fn r(&self) -> f32 {
        debug_assert!(self.inner[0] >= 0.);
        debug_assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32 {
        debug_assert!((0.0..=std::f32::consts::PI).contains(&self.inner[1]));
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32 {
        debug_assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[2]));
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }
}

impl<C> std::ops::Index<usize> for Point3<C> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            _ => panic!("Point3 index out of bounds: {}", index),
        }
    }
}

impl<C> std::ops::IndexMut<usize> for Point3<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            _ => panic!("Point3 index out of bounds: {}", index),
        }
    }
}

// coordinates only form an affine space in a cartesian chart

impl std::ops::Neg for Point3<Cartesian> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner.x, -self.inner.y, -self.inner.z, self.chart)
    }
}

impl std::ops::Add for Point3<Cartesian> {
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

impl std::ops::Sub for Point3<Cartesian> {
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

impl std::ops::Mul<f32> for Point3<Cartesian> {
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

impl std::ops::Mul<Point3<Cartesian>> for f32 {
    type Output = Point3<Cartesian>;

    fn mul(self, rhs: Point3<Cartesian>) -> Self::Output {
        rhs * self
    }
}

// Point4 is a point on the (non necessarly riemannian) smooth manifold R^4.
// its charts are products of the time coordinate with a (3 dimensional) chart.
#[derive(Clone, Copy, PartialEq)]
pub struct Point4<C> {
    pub inner: Vec4,
    pub chart: C
}

impl<C: IsChart> Point4<C> {
    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32, chart: C) -> Self {
        Self {
            inner: Vec4::new(x0, x1, x2, x3),
            chart: chart,
        }
    }

    pub fn zero(chart: C) -> Self {
        Self {
            inner: Vec4::new(0., 0., 0., 0.),
            chart: chart,
        }
    }

    pub fn from_space_time(time: f32, space: Point3<C>) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            chart: space.chart,
        }
    }

    pub fn t(&self) -> f32 {
        debug_assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn space(&self) -> Point3<C> {
        Point3::new(self.inner[1], self.inner[2], self.inner[3], self.chart)
    }

    pub fn as_fourvector(self) -> FourVector<C> {
        FourVector::new(self.inner[0], self.inner[1], self.inner[2], self.inner[3], self.chart)
    }

    pub fn as_vec4(self) -> Vec4 { self.inner }
}

impl Point4<Cartesian> {
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

impl<C: IsSphericalChart> Point4<C> {
    pub fn new_spherical(t: f32, r: f32, theta: f32, phi: f32, chart: C) -> Self {
        debug_assert!(r >= 0.0);
        debug_assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec4::new(t, r, theta, phi),
            chart: chart,
        }
    }

    pub fn r(&self) -> f32 {
        debug_assert!(self.inner[1] >= 0.);
        debug_assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        debug_assert!((0.0..=std::f32::consts::PI).contains(&self.inner[2]));
        debug_assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        debug_assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[3]));
        debug_assert!(self.inner[3].is_finite());
        self.inner[3]
    }
}

impl<C> std::ops::Index<usize> for Point4<C> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            3 => &self.inner.w,
            _ => panic!("Point4 index out of bounds: {}", index),
        }
    }
}

impl<C> std::ops::IndexMut<usize> for Point4<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            3 => &mut self.inner.w,
            _ => panic!("Point4 index out of bounds: {}", index),
        }
    }
}

impl std::ops::Add for Point4<Cartesian> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.chart == rhs.chart);
        Self {
            inner: self.inner + rhs.inner,
            chart: self.chart,
        }
    }
}

impl std::ops::Mul<f32> for Point4<Cartesian> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            chart: self.chart,
        }
    }
}

impl std::ops::Mul<Point4<Cartesian>> for f32 {
    type Output = Point4<Cartesian>;

    fn mul(self, rhs: Point4<Cartesian>) -> Self::Output {
        rhs * self
    }
}
