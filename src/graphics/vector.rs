use rand::RngExt;
use glam::{Vec3, Vec4};

#[derive(Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    Cartesian,
    Spherical,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ThreeVector {
    pub inner: Vec3,
    pub coordinate_system: CoordinateSystem
}

impl ThreeVector {
    pub const ZERO_CART: Self = Self {inner: Vec3::new(0., 0., 0.), coordinate_system: CoordinateSystem::Cartesian};
    pub const ZERO_SPH: Self = Self {inner: Vec3::new(0., 0., 0.), coordinate_system: CoordinateSystem::Spherical};

    pub fn new(x0: f32, x1: f32, x2: f32, coordinate_system: CoordinateSystem) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            coordinate_system: coordinate_system,
        }
    }

    pub fn new_cartesian(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: Vec3::new(x, y, z),
            coordinate_system: CoordinateSystem::Cartesian,
        }
    }

    pub fn new_spherical(r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            coordinate_system: CoordinateSystem::Spherical,
        }
    }

    pub fn length(&self) -> f32 {
        match self.coordinate_system {
            CoordinateSystem::Cartesian => self.inner.length(),
            CoordinateSystem::Spherical => self.r(),
        }
    }

    pub fn to_cartesian(self) -> Self {
        match self.coordinate_system {
            CoordinateSystem::Cartesian => self,
            CoordinateSystem::Spherical => {

                let x = self.r() * self.theta().sin() * self.phi().cos();
                let y = self.r() * self.theta().sin() * self.phi().sin();
                let z = self.r() * self.theta().cos();

                Self::new_cartesian(x, y, z)
            }
        }
    }

    pub fn to_spherical(self) -> Self {
        match self.coordinate_system {
            CoordinateSystem::Spherical => self,
            CoordinateSystem::Cartesian => {
                let r = self.length();
                if r == 0.0 {
                    return Self::new_spherical(r, 0.0, 0.0);
                }
                let theta = (self.z() / r).acos();

                let phi = self.y().atan2(self.x()).rem_euclid(2.0 * std::f32::consts::PI);

                Self::new_spherical(r, theta, phi)
            }
        }
    }

    pub fn x(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn y(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn z(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn r(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!(self.inner[0] >= 0.);
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn theta(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[1]));
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn phi(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn normalize(&self) -> ThreeVector {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.length() != 0.);
        ThreeVector { inner: self.inner.normalize(), coordinate_system: CoordinateSystem::Cartesian }
    }

    pub fn dot(&self, other: ThreeVector) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner.dot(other.inner)
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

impl std::ops::Index<usize> for ThreeVector {
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

impl std::ops::IndexMut<usize> for ThreeVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            _ => panic!("ThreeVector index out of bounds: {}", index),
        }
    }
}

impl std::ops::Neg for ThreeVector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        Self::new_cartesian(-self.inner.x, -self.inner.y, -self.inner.z)
    }
}

impl std::ops::Add for ThreeVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.coordinate_system == rhs.coordinate_system);
        Self::new(
            self.inner.x + rhs.inner.x,
            self.inner.y + rhs.inner.y,
            self.inner.z + rhs.inner.z,
            self.coordinate_system,
        )
    }
}

impl std::ops::Sub for ThreeVector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.coordinate_system == rhs.coordinate_system);
        Self::new(
            self.inner.x - rhs.inner.x,
            self.inner.y - rhs.inner.y,
            self.inner.z - rhs.inner.z,
            self.coordinate_system,
        )
    }
}

impl std::ops::Mul<f32> for ThreeVector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.inner.x * rhs,
            self.inner.y * rhs,
            self.inner.z * rhs,
            self.coordinate_system,
        )
    }
}

impl std::ops::Mul<ThreeVector> for f32 {
    type Output = ThreeVector;

    fn mul(self, rhs: ThreeVector) -> Self::Output {
        rhs * self
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct FourVector {
    pub inner: Vec4,
    pub coordinate_system: CoordinateSystem
}

impl FourVector {
    pub const ZERO_CART: Self = Self {inner: Vec4::new(0., 0., 0., 0.), coordinate_system: CoordinateSystem::Cartesian};
    pub const ZERO_SPH: Self = Self {inner: Vec4::new(0., 0., 0., 0.), coordinate_system: CoordinateSystem::Spherical};

    pub fn new_cartesian(t: f32, x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: Vec4::new(t, x, y, z),
            coordinate_system: CoordinateSystem::Cartesian,
        }
    }

    pub fn new_spherical(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec4::new(t, r, theta, phi),
            coordinate_system: CoordinateSystem::Spherical,
        }
    }

    pub fn from_space_time(time: f32, space: ThreeVector) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            coordinate_system: space.coordinate_system
        }
    }

    pub fn t(&self) -> f32 {
        assert!(self.inner[0].is_finite());
        self.inner[0]
    }

    pub fn x(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn y(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn z(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }
    
    pub fn r(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!(self.inner[1] >= 0.);
        assert!(self.inner[1].is_finite());
        self.inner[1]
    }

    pub fn theta(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!((0.0..=std::f32::consts::PI).contains(&self.inner[2]));
        assert!(self.inner[2].is_finite());
        self.inner[2]
    }

    pub fn phi(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        assert!((0.0..=std::f32::consts::TAU).contains(&self.inner[3]));
        assert!(self.inner[3].is_finite());
        self.inner[3]
    }

    pub fn space(&self) -> ThreeVector {
        ThreeVector::new(self.inner[1], self.inner[2], self.inner[3], self.coordinate_system)
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
        assert!(self.coordinate_system == rhs.coordinate_system);
        Self {
            inner: self.inner + rhs.inner,
            coordinate_system: self.coordinate_system,
        }
    }
}

impl std::ops::Mul<f32> for FourVector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            coordinate_system: self.coordinate_system,
        }
    }
}

impl std::ops::Mul<FourVector> for f32 {
    type Output = FourVector;

    fn mul(self, rhs: FourVector) -> Self::Output {
        rhs * self
    }
}

#[allow(dead_code)]
pub fn random_on_sphere(coordinate_system: CoordinateSystem) -> ThreeVector {
    let mut rng = rand::rng();

    let costheta: f32 = rng.random_range((-1.)..(1.));
    let theta = costheta.acos();
    let phi = rng.random_range(0.0..std::f32::consts::TAU);

    let v = ThreeVector::new_spherical(1.0, theta, phi);
    match coordinate_system {
        CoordinateSystem::Spherical => v,
        CoordinateSystem::Cartesian => v.to_cartesian(),
    }
}

// // returns a random vector in the hemisphere aligned with v
// pub fn random_on_hemisphere(v: Vec3) -> Vec3 {
//     let vec = random_on_sphere();
//     if vec.dot(v) > 0.0 { vec } else { -vec }
// }

pub type Point3 = ThreeVector;
pub type Point4 = FourVector;