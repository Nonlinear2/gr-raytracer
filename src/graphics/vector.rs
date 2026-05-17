use rand::RngExt;
use glam::{Vec3, Vec4};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    Cartesian,
    Spherical,
}

pub struct ThreeVector {
    inner: Vec3,
    coordinate_system: CoordinateSystem
}

impl ThreeVector {
    pub fn new(x0: f32, x1: f32, x2: f32, coordinate_system: CoordinateSystem) -> Self {
        Self {
            inner: Vec3::new(x0, x1, x2),
            coordinate_system: coordinate_system,
        }
    }

    pub fn new_carthesian(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: Vec3::new(x, y, z),
            coordinate_system: CoordinateSystem::Cartesian,
        }
    }

    pub fn new_spherical(t: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self {
            inner: Vec3::new(r, theta, phi),
            coordinate_system: CoordinateSystem::Spherical,
        }
    }

    pub fn x(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[0]
    }

    pub fn y(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[1]
    }

    pub fn z(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[2]
    }

    pub fn r(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[0]
    }

    pub fn theta(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[1]
    }

    pub fn phi(&self) -> f32{
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[2]
    }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

impl Index<usize> for ThreeVector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            _ => panic!(),
        }
    }
}

pub struct FourVector {
    inner: Vec4,
    coordinate_system: CoordinateSystem
}

impl FourVector {
    pub fn new_carthesian(t: f32, x: f32, y: f32, z: f32) -> Self {
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

    fn from_space_time(time: f32, space: ThreeVector) -> Self {
        Self {
            inner: Vec4::new(time, space.inner.x, space.inner.y, space.inner.z),
            coordinate_system: space.coordinate_system
        }
    }

    fn t(&self) -> f32 {
        self.inner[0]
    }

    fn x(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[1]
    }

    fn y(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[2]
    }

    fn z(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Cartesian);
        self.inner[3]
    }
    
    fn r(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[1]
    }

    fn theta(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[2]
    }

    fn phi(&self) -> f32 {
        assert!(self.coordinate_system == CoordinateSystem::Spherical);
        self.inner[3]
    }

    fn space(&self) -> ThreeVector {
        ThreeVector::new(self.inner[1], self.inner[2], self.inner[3], self.coordinate_system)
    }

    pub fn as_vec4(self) -> Vec4 { self.inner }
}

impl Index<usize> for FourVector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.inner.x,
            1 => &self.inner.y,
            2 => &self.inner.z,
            3 => &self.inner.w,
            _ => panic!(),
        }
    }
}

impl IndexMut<usize> for FourVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.inner.x,
            1 => &mut self.inner.y,
            2 => &mut self.inner.z,
            3 => &mut self.inner.w,
            _ => panic!(),
        }
    }
}

pub fn random_on_sphere() -> Vec3 {
    let mut rng = rand::rng();

    let phi = rng.random_range(0.0..2.*std::f32::consts::PI);
    let costheta: f32 = rng.random_range((-1.)..(1.));

    let theta = costheta.acos();
    Vec3 {
        x: theta.sin() * phi.cos(),
        y: theta.sin() * phi.sin(),
        z: theta.cos(),
    }
}

// returns a random vector in the hemisphere aligned with v
pub fn random_on_hemisphere(v: Vec3) -> Vec3 {
    let vec = random_on_sphere();
    if vec.dot(v) > 0.0 { vec } else { -vec }
}

pub type Point3 = ThreeVector;
pub type Point4 = FourVector;