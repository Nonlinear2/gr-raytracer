use glam::{Vec3, Vec4};

use crate::graphics::vector::{FourVector, TangentSpace, ThreeVector};

// which global chart we use to describe points on the manifolds R^3 and R^4. Synonym for "coordinate system"
// in R^4 Chart::Spherical represents the chart: (t, x, y, z) -> (t, (spherical on R^3 for x, y, z)).
#[derive(Clone, Copy, PartialEq)]
pub enum Chart { 
    Cartesian,
    Spherical,
}

// Point3 is a point on the manifold R^3
#[derive(Clone, Copy, PartialEq)]
pub struct Point3 {
    pub inner: Vec3, // misleading name, here we are really differentiating between points
    // which live on the manifold R^3, and vectors which live in a tangent vector space to a point.
    pub chart: Chart
}

impl Point3 {
    pub const ZERO_CART: Self = Self {inner: Vec3::new(0., 0., 0.), chart: Chart::Cartesian};
    pub const ZERO_SPH: Self = Self {inner: Vec3::new(0., 0., 0.), chart: Chart::Spherical};

    pub fn new(x0: f32, x1: f32, x2: f32, chart: Chart) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            chart: chart,
        }
    }

    pub fn new_cartesian(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: Vec3::new(x, y, z),
            chart: Chart::Cartesian,
        }
    }

    pub fn new_spherical(r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            chart: Chart::Spherical,
        }
    }

    pub fn distance_to_zero(&self) -> f32 {
        match self.chart {
            Chart::Cartesian => self.inner.length(),
            Chart::Spherical => self.r(),
        }
    }

    pub fn to_cartesian(self) -> Self {
        match self.chart {
            Chart::Cartesian => self,
            Chart::Spherical => {

                let x = self.r() * self.theta().sin() * self.phi().cos();
                let y = self.r() * self.theta().sin() * self.phi().sin();
                let z = self.r() * self.theta().cos();

                Self::new_cartesian(x, y, z)
            }
        }
    }

    pub fn to_spherical(self) -> Self {
        assert!(self.distance_to_zero() != 0.0);
        match self.chart {
            Chart::Spherical => self,
            Chart::Cartesian => {
                let r = self.distance_to_zero();
                let theta = (self.z() / r).acos();
                let phi = self.y().atan2(self.x()).rem_euclid(2.0 * std::f32::consts::PI);

                Self::new_spherical(r, theta, phi)
            }
        }
    }

    pub fn to_spherical_on_z_axis(self, phi: f32) -> Self {
        assert!(self.distance_to_zero() != 0.0);
        match self.chart {
            Chart::Spherical => self,
            Chart::Cartesian => {
                let r = self.distance_to_zero();
                let theta = (self.z() / r).acos();
                Self::new_spherical(r, theta, phi)
            }
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
        assert!(self.chart == Chart::Spherical);
        assert!(self.inner[0] >= 0.);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32{
        assert!(self.chart == Chart::Spherical);
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[1]));
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32{
        assert!(self.chart == Chart::Spherical);
        assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    // pub fn normalize(&self) -> ThreeVector {
    //     assert!(self.coordinate_system == Chart::Cartesian);
    //     assert!(self.distance_to_zero() != 0.);
    //     ThreeVector { inner: self.inner.normalize(), coordinate_system: Chart::Cartesian }
    // }

    // pub fn dot(&self, other: ThreeVector) -> f32 {
    //     assert!(self.coordinate_system == Chart::Cartesian);
    //     self.inner.dot(other.inner)
    // }

    pub fn as_threevector(self) -> ThreeVector {
        match self.chart {
            Chart::Cartesian => ThreeVector::new(
                self.inner[0], self.inner[1], self.inner[2], TangentSpace::Cartesian
            ),
            Chart::Spherical => ThreeVector::new(
                self.inner[0], self.inner[1], self.inner[2], TangentSpace::Spherical
            )
        }
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

impl std::ops::Index<usize> for Point3 {
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

impl std::ops::IndexMut<usize> for Point3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            _ => panic!("ThreeVector index out of bounds: {}", index),
        }
    }
}

impl std::ops::Neg for Point3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        assert!(self.chart == Chart::Cartesian);
        Self::new_cartesian(-self.inner.x, -self.inner.y, -self.inner.z)
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
        assert!(rhs.chart == Chart::Cartesian);
        rhs * self
    }
}

// Point4 is a point on the (non necessarly riemannian) smooth manifold R^4.
#[derive(Clone, Copy, PartialEq)]
pub struct Point4 {
    pub inner: Vec4,
    pub chart: Chart
}

impl Point4 {
    pub const ZERO_CART: Self = Self {inner: Vec4::new(0., 0., 0., 0.), chart: Chart::Cartesian};
    pub const ZERO_SPH: Self = Self {inner: Vec4::new(0., 0., 0., 0.), chart: Chart::Spherical};

    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32, chart: Chart) -> Self {
        Self {
            inner: Vec4::new(x0, x1, x2, x3),
            chart: chart,
        }
    }

    pub fn new_cartesian(t: f32, x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: Vec4::new(t, x, y, z),
            chart: Chart::Cartesian,
        }
    }

    pub fn new_spherical(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec4::new(t, r, theta, phi),
            chart: Chart::Spherical,
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
        assert!(self.chart == Chart::Spherical);
        assert!(self.inner[1] >= 0.);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        assert!(self.chart == Chart::Spherical);
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        assert!(self.chart == Chart::Spherical);
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
            Chart::Spherical => FourVector::new(
                self.inner[0], self.inner[1], self.inner[2], self.inner[3], TangentSpace::Spherical
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
        assert!(rhs.chart == Chart::Cartesian);
        rhs * self
    }
}