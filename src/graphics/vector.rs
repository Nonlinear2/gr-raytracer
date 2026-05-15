use std::ops::{Add, Div, Mul, Neg, Sub};
use rand::{Rng, RngExt};
use glam::{Vec3};

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
pub type Color = Vec3;