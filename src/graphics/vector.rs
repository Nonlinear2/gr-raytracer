use rand::RngExt;
use glam::{Vec3, Vec4};

pub trait FourVector {
    fn from_space_time(time: f32, space: Vec3) -> Vec4;
    fn time(&self) -> f32;
    fn space(&self) -> Vec3;
}

impl FourVector for Vec4 {
    fn from_space_time(time: f32, space: Vec3) -> Vec4 {
        Vec4::new(time, space.x, space.y, space.z)
    }

    fn time(&self) -> f32 {
        self.x
    }

    fn space(&self) -> Vec3 {
        Vec3::new(self.y, self.z, self.w)
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphVec3 {
    inner: Vec3,
}

impl SphVec3 {
    pub fn new(r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self { inner: Vec3::new(r, theta, phi) }
    }

    pub fn r(self) -> f32 { self.inner.x }
    pub fn theta(self) -> f32 { self.inner.y }
    pub fn phi(self) -> f32 { self.inner.z }

    pub fn with_r(mut self, r: f32) -> Self { self.inner.x = r; self }
    pub fn with_theta(mut self, theta: f32) -> Self { self.inner.y = theta; self }
    pub fn with_phi(mut self, phi: f32) -> Self { self.inner.z = phi; self }

    pub fn as_vec3(self) -> Vec3 { self.inner }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphVec4 {
    inner: Vec4,
}

impl SphVec4 {
    pub fn new(ct: f32, r: f32, theta: f32, phi: f32) -> Self {
        assert!(r >= 0.0);
        assert!((0.0..=std::f32::consts::PI).contains(&theta));
        Self { inner: Vec4::new(ct, r, theta, phi) }
    }

    /// Time / ct component
    pub fn ct(self) -> f32 { self.inner.x }
    pub fn time(self) -> f32 { self.inner.x }

    /// Spatial spherical components
    pub fn r(self) -> f32 { self.inner.y }
    pub fn theta(self) -> f32 { self.inner.z }
    pub fn phi(self) -> f32 { self.inner.w }

    pub fn with_ct(mut self, ct: f32) -> Self { self.inner.x = ct; self }
    pub fn with_time(mut self, t: f32) -> Self { self.inner.x = t; self }
    pub fn with_r(mut self, r: f32) -> Self { self.inner.y = r; self }
    pub fn with_theta(mut self, theta: f32) -> Self { self.inner.z = theta; self }
    pub fn with_phi(mut self, phi: f32) -> Self { self.inner.w = phi; self }

    pub fn as_vec4(self) -> Vec4 { self.inner }
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

pub type Point3 = Vec3;
pub type Point4 = Vec4;