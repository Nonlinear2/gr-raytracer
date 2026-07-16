use glam::{Vec3, Vec4};

use crate::geometry::vector::{FourVector, TangentSpace, ThreeVector};
use crate::geometry::manifold::{ChartWorld, Chart, TangentWorld};

// Point3 is a point on the manifold R^3
#[derive(Clone, Copy, PartialEq)]
pub struct Point3<C = Chart> {
    pub inner: Vec3, // this is not a vector, here we are really differentiating between points
    // which live on the manifold R^3, and vectors which live in a tangent vector space to a point.
    pub chart: C
}

impl<C> Point3<C> {
    pub fn new(x0: f32, x1: f32, x2: f32, chart: C) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            chart: chart,
        }
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

impl Point3<ChartWorld> {
    pub const ZERO: Self = Self { inner: Vec3::new(0., 0., 0.), chart: ChartWorld };

    pub fn distance_to_zero(&self) -> f32 {
        self.inner.length()
    }

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

    pub fn as_threevector(self) -> ThreeVector<TangentWorld> {
        ThreeVector::new(self.inner[0], self.inner[1], self.inner[2], TangentWorld)
    }
}

impl Point3 {
    pub const ZERO_CART: Self = Self {inner: Vec3::new(0., 0., 0.), chart: Chart::Cartesian};
    pub const ZERO_SPHZ: Self = Self {inner: Vec3::new(0., 0., 0.), chart: Chart::SphericalZ};
    pub const ZERO_SPHX: Self = Self {inner: Vec3::new(0., 0., 0.), chart: Chart::SphericalX};

    pub fn new_spherical_z(r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            chart: Chart::SphericalZ,
        }
    }

    pub fn new_spherical_x(r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            chart: Chart::SphericalX,
        }
    }

    pub fn distance_to_zero(&self) -> f32 {
        match self.chart {
            Chart::Cartesian => self.inner.length(),
            Chart::SphericalZ | Chart::SphericalX => self.r(),
        }
    }

    pub fn x(&self) -> f32{
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn y(&self) -> f32{
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn z(&self) -> f32{
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn r(&self) -> f32{
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!(self.inner[0] >= 0.);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32{
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[1]));
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32{
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn as_threevector(self) -> ThreeVector {
        match self.chart {
            Chart::Cartesian => ThreeVector::new(
                self.inner[0], self.inner[1], self.inner[2], TangentSpace::Cartesian
            ),
            Chart::SphericalZ => ThreeVector::new(
                self.inner[0], self.inner[1], self.inner[2], TangentSpace::SphericalZ
            ),
            Chart::SphericalX => ThreeVector::new(
                self.inner[0], self.inner[1], self.inner[2], TangentSpace::SphericalX
            )
        }
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

impl std::ops::Neg for Point3<ChartWorld> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner.x, -self.inner.y, -self.inner.z, ChartWorld)
    }
}

impl std::ops::Add for Point3<ChartWorld> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.inner.x + rhs.inner.x,
            self.inner.y + rhs.inner.y,
            self.inner.z + rhs.inner.z,
            ChartWorld,
        )
    }
}

impl std::ops::Sub for Point3<ChartWorld> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.inner.x - rhs.inner.x,
            self.inner.y - rhs.inner.y,
            self.inner.z - rhs.inner.z,
            ChartWorld,
        )
    }
}

impl std::ops::Mul<f32> for Point3<ChartWorld> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.inner.x * rhs,
            self.inner.y * rhs,
            self.inner.z * rhs,
            ChartWorld,
        )
    }
}

impl std::ops::Mul<Point3<ChartWorld>> for f32 {
    type Output = Point3<ChartWorld>;

    fn mul(self, rhs: Point3<ChartWorld>) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Neg for Point3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        Self::new(-self.inner.x, -self.inner.y, -self.inner.z, self.chart)
    }
}

impl std::ops::Add for Point3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.chart == rhs.chart);
        Self::new(
            self.inner.x + rhs.inner.x,
            self.inner.y + rhs.inner.y,
            self.inner.z + rhs.inner.z,
            self.chart,
        )
    }
}

impl std::ops::Sub for Point3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.chart == rhs.chart);
        Self::new(
            self.inner.x - rhs.inner.x,
            self.inner.y - rhs.inner.y,
            self.inner.z - rhs.inner.z,
            self.chart,
        )
    }
}

impl std::ops::Mul<f32> for Point3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        Self::new(
            self.inner.x * rhs,
            self.inner.y * rhs,
            self.inner.z * rhs,
            self.chart,
        )
    }
}

impl std::ops::Mul<Point3> for f32 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        rhs * self
    }
}

// Point4 is a point on the (non necessarly riemannian) smooth manifold R^4.
// its charts are products of the time coordinate with a (3 dimensional) Chart,
// so the CartesianWorld chart is not available for it.
#[derive(Clone, Copy, PartialEq)]
pub struct Point4 {
    pub inner: Vec4,
    pub chart: Chart
}

impl Point4 {
    pub const ZERO_CART: Self = Self {inner: Vec4::new(0., 0., 0., 0.), chart: Chart::Cartesian};
    pub const ZERO_SPHZ: Self = Self {inner: Vec4::new(0., 0., 0., 0.), chart: Chart::SphericalZ};
    pub const ZERO_SPHX: Self = Self {inner: Vec4::new(0., 0., 0., 0.), chart: Chart::SphericalX};

    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32, chart: Chart) -> Self {
        Self {
            inner: Vec4::new(x0, x1, x2, x3),
            chart: chart,
        }
    }

    pub fn new_spherical_z(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec4::new(t, r, theta, phi),
            chart: Chart::SphericalZ,
        }
    }

    pub fn new_spherical_x(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec4::new(t, r, theta, phi),
            chart: Chart::SphericalX,
        }
    }

    pub fn from_space_time(time: f32, space: Point3) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            chart: space.chart
        }
    }

    pub fn t(&self) -> f32 {
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn x(&self) -> f32 {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn y(&self) -> f32 {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn z(&self) -> f32 {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }

    pub fn r(&self) -> f32 {
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!(self.inner[1] >= 0.);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        assert!(matches!(self.chart, Chart::SphericalZ | Chart::SphericalX));
        assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[3]));
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }

    pub fn space(&self) -> Point3 {
        Point3::new(self.inner[1], self.inner[2], self.inner[3], self.chart)
    }

    pub fn as_fourvector(self) -> FourVector {
        match self.chart {
            Chart::Cartesian => FourVector::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], TangentSpace::Cartesian
            ),
            Chart::SphericalZ => FourVector::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], TangentSpace::SphericalZ
            ),
            Chart::SphericalX => FourVector::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], TangentSpace::SphericalX
            )
        }
    }

    pub fn as_vec4(self) -> Vec4 { self.inner }
}

impl std::ops::Index<usize> for Point4 {
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

impl std::ops::IndexMut<usize> for Point4 {
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

impl std::ops::Add for Point4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        assert!(self.chart == rhs.chart);
        Self {
            inner: self.inner + rhs.inner,
            chart: self.chart,
        }
    }
}

impl std::ops::Mul<f32> for Point4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        Self {
            inner: self.inner * rhs,
            chart: self.chart,
        }
    }
}

impl std::ops::Mul<Point4> for f32 {
    type Output = Point4;

    fn mul(self, rhs: Point4) -> Self::Output {
        rhs * self
    }
}
