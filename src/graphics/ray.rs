use crate::graphics::vector::{FourVector, Point3, Point4};
use crate::graphics::surface::Material;
use crate::integration::solvers::positive_root;

use glam::{Vec3, Vec4, Mat4};


pub struct PhotonIntersection<'a> {
    pub g: Mat4,
    pub metric_center: Point3,
    pub point: Point4,
    pub normal: Vec3,
    pub material: &'a dyn Material
}


pub struct Photon {
    pub pos: Point4,
    pub vel: Vec4,
}

impl Photon {
    pub fn from_space_vel(g: Mat4, metric_center: Point3, pos: Point4, vel: Vec3) -> Self {
        // Metric g is spherical-basis, vel is Cartesian-basis. Convert vel to spherical
        // to avoid mixing bases when solving the null condition g_ij k^i k^j = 0.

        let pos_cart = pos.space() - metric_center;
        let r = pos_cart.length();
        let x = pos_cart.x; let y = pos_cart.y; let z = pos_cart.z;
        let rho = (x*x + y*y).sqrt();

        // Convert Cartesian velocity to spherical-basis components.
        // At the polar axis, phi is undefined, so we pick a local azimuth from the
        // velocity itself and keep the transverse direction instead of dropping it.
        let (v_r, v_th, v_ph, phi_hint) = if r > 1e-8 {
            let dr_dx = x / r; let dr_dy = y / r; let dr_dz = z / r;
            let (dth_dx, dth_dy, dth_dz) = if rho > 1e-8 {
                ( x*z / (r*r*rho), y*z / (r*r*rho), -rho / (r*r) )
            } else { (0.0, 0.0, 0.0) };
            let (dph_dx, dph_dy) = if rho > 1e-8 {
                ( -y / (rho*rho), x / (rho*rho) )
            } else { (0.0, 0.0) };
            (
                dr_dx * vel.x + dr_dy * vel.y + dr_dz * vel.z,
                dth_dx * vel.x + dth_dy * vel.y + dth_dz * vel.z,
                dph_dx * vel.x + dph_dy * vel.y,
                y.atan2(x),
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        let (v_r, v_th, v_ph, phi_hint) = if rho <= 1e-8 {
            let pole_sign = if z >= 0.0 { 1.0 } else { -1.0 };
            let tangential = (vel.x * vel.x + vel.y * vel.y).sqrt();
            (
                pole_sign * vel.z,
                tangential / r.max(1e-8),
                0.0,
                vel.y.atan2(vel.x),
            )
        } else {
            (v_r, v_th, v_ph, phi_hint)
        };

        // Solve null condition in spherical basis
        let b = 2.0 * (g.col(0)[1] * v_r + g.col(0)[2] * v_th + g.col(0)[3] * v_ph);
        let mut c = 0.;
        c += g.col(1)[1] * v_r * v_r;
        c += 2.0 * g.col(1)[2] * v_r * v_th;
        c += 2.0 * g.col(1)[3] * v_r * v_ph;
        c += g.col(2)[2] * v_th * v_th;
        c += 2.0 * g.col(2)[3] * v_th * v_ph;
        c += g.col(3)[3] * v_ph * v_ph;

        let k_0 = positive_root(g.col(0)[0], b, c);

        // Convert spherical-basis velocity back to Cartesian
        let theta = if r > 1e-8 { (z / r).clamp(-1.0, 1.0).acos() } else { 0.0 };
    let phi = phi_hint;
        let sin_th = theta.sin(); let cos_th = theta.cos();
        let sin_ph = phi.sin(); let cos_ph = phi.cos();
        let e_r = Vec3::new(sin_th * cos_ph, sin_th * sin_ph, cos_th);
        let e_th = Vec3::new(r * cos_th * cos_ph, r * cos_th * sin_ph, -r * sin_th);
        let e_ph = Vec3::new(-r * sin_th * sin_ph, r * sin_th * cos_ph, 0.0);
        let vel_cart = e_r * v_r + e_th * v_th + e_ph * v_ph;

        Self::new(pos, Vec4::from_space_time(k_0, vel_cart))
    }

    pub fn new(pos: Point4, vel: Vec4) -> Self {
        Self {
            pos: pos,
            vel: vel,
        }

    }
}