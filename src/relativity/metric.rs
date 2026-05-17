use crate::graphics::{ray::Photon, vector::{FourVector, Point3, Point4, SphVec3, SphVec4}};
use glam::{Vec4, Mat4, Vec3};

pub trait Metric {
    fn center(&self) -> Point3;
    fn g(&self, x: Point4) -> Mat4;
    fn g_sph(&self, x: SphVec4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;
    fn del_g_sph(&self, x: SphVec4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;
    fn christoffel_sph(&self, pos: SphVec4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon;

    // fn dot(&self, x: Point4, v1: Vec4, v2: Vec4) -> f32 {
    //     v1.dot(self.g(x) * v2)
    // }

    // fn norm(&self, x: Point4, v1: Vec4) -> f32 {
    //     self.dot(x, v1, v1)
    // }
}

pub struct EuclideanMetric {

}


impl Metric for EuclideanMetric {
    fn center(&self) -> Point3 {
        Point3::ZERO
    }

    fn g(&self, x: Point4) -> Mat4 {
        Mat4::ZERO
    }

    fn g_sph(&self, x: SphVec4) -> Mat4 {
        Mat4::ZERO
    }

    fn del_g(&self, x: Point4, i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn del_g_sph(&self, x: SphVec4, i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        0.
    }
    fn christoffel_sph(&self, pos: SphVec4, mu: usize, nu: usize, lambda: usize) -> f32 {
        0.
    }

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon {
        Photon::new(s.pos + s.vel * h, s.vel)
    }
}

pub struct SchwartzschildMetric {
    pub center: Point3,
    pub r_s: f32,
}

impl SchwartzschildMetric {
    pub fn new(center: Point3, r_s: f32) -> Self {
        Self {
            center: center,
            r_s: r_s,
        }
    }

    pub fn mass(&self) -> f32 { // schwartzschild radius
        return self.r_s; // r_s = 2GM/c^2.
    }

    pub fn to_spherical_coordinates(&self, pos: Point3) -> SphVec3 {
        let relative_pos = pos - self.center;
        let mut length = relative_pos.length();

        // guard against NaN/Inf or non-physical small/negative radii produced
        // by numerical errors elsewhere. Clamp to just outside the horizon.
        if !length.is_finite() {
            length = self.r_s + 1e-6;
        }
        if length <= self.r_s {
            length = self.r_s + 1e-6;
        }

        let theta = if length > 0. {
            // protect the acos argument against tiny numeric overshoot
            let cos_theta = (relative_pos.z / length).clamp(-1.0, 1.0);
            cos_theta.acos()
        } else { 0. };

        let phi = if relative_pos.x.is_finite() && relative_pos.y.is_finite() {
            relative_pos.y.atan2(relative_pos.x)
        } else { 0. };

        // Guard against NaN theta/phi from numeric errors; default to safe fallback values
        // Also avoid exact poles (theta == 0 or PI) which make the spherical chart
        // singular (sin theta == 0) and produce infinities when inverting the metric.
        let theta = if theta.is_finite() && (0.0..=std::f32::consts::PI).contains(&theta) {
            // clamp slightly away from the poles
            theta.clamp(1e-6, std::f32::consts::PI - 1e-6)
        } else {
            std::f32::consts::PI / 2.0  // default to equator if invalid
        };

        let phi = if phi.is_finite() {
            phi
        } else {
            0.0
        };

        SphVec3::new(length, theta, phi)
    }
}

impl Metric for SchwartzschildMetric {
    fn center(&self) -> Point3 {
        self.center
    }

    fn g(&self, pos: Point4) -> Mat4 {
        let sph_pos = self.to_spherical_coordinates(pos.space());
        self.g_sph(SphVec4::new(pos.time(), sph_pos.r(), sph_pos.theta(), sph_pos.phi()))
    }

    fn g_sph(&self, pos: SphVec4) -> Mat4 {
        let r = pos.r();
        let theta = pos.theta();
        assert!(r > self.r_s);

        let g = Mat4 {
            x_axis: Vec4::new(1. - self.r_s / r, 0., 0., 0.),
            y_axis: Vec4::new(0., -1./(1. - self.r_s / r), 0., 0.),
            z_axis: Vec4::new(0., 0., -r*r, 0.),
            w_axis: Vec4::new(0., 0., 0., -r*r*theta.sin()*theta.sin()),
        };
        
        if !g.determinant().is_finite() {
            eprintln!("[g_sph] Degenerate metric: r={}, theta={}, det={}", r, theta, g.determinant());
        }
        
        g
    }

    fn del_g(&self, pos: Point4, i: u32) -> Mat4 {
        let sph_pos = self.to_spherical_coordinates(pos.space());
        self.del_g_sph(SphVec4::new(pos.time(), sph_pos.r(), sph_pos.theta(), sph_pos.phi()), i)
    }

    fn del_g_sph(&self, pos: SphVec4, i: u32) -> Mat4 {
        let r = pos.r();
        let theta = pos.theta();
        match i {
            0 => Mat4::ZERO,
            1 => Mat4 {
                x_axis: Vec4::new(self.r_s / (r*r), 0., 0., 0.),
                y_axis: Vec4::new(0., self.r_s / (r*r*(1. - self.r_s / r) * (1. - self.r_s / r)), 0., 0.),
                z_axis: Vec4::new(0., 0., -2.*r, 0.),
                w_axis: Vec4::new(0., 0., 0., -2.*r*theta.sin()*theta.sin()),
            },
            2 => Mat4 {
                x_axis: Vec4::ZERO,
                y_axis: Vec4::ZERO,
                z_axis: Vec4::ZERO,
                w_axis: Vec4::new(0., 0., 0., -2.*r*r*theta.cos()*theta.sin()),
            },
            3 => Mat4::ZERO,
            _ => unreachable!()
        }
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        let sph_pos = self.to_spherical_coordinates(pos.space());
        self.christoffel_sph(SphVec4::new(pos.time(), sph_pos.r(), sph_pos.theta(), sph_pos.phi()), mu, nu, lambda)
    }

    fn christoffel_sph(&self, pos: SphVec4, mu: usize, nu: usize, lambda: usize) -> f32 {
        let g_inv = self.g_sph(pos).inverse();
        let mut gamma = 0.;

        let d_mu_g = self.del_g_sph(pos, mu as u32);
        let d_nu_g = self.del_g_sph(pos, nu as u32);

        for alpha in 0..4 {
            let d_alpha_g = self.del_g_sph(pos, alpha as u32);

            gamma += 0.5 * g_inv.col(lambda)[alpha] * (
                d_mu_g.col(alpha)[nu]
              + d_nu_g.col(alpha)[mu]
              - d_alpha_g.col(mu)[nu]
            )
        }
        if !gamma.is_finite() {
            eprintln!("[christoffel] NaN detected: mu={}, nu={}, lambda={}, pos=({},{},{}), gamma={}", mu, nu, lambda, pos.r(), pos.theta(), pos.phi(), gamma);
            eprintln!("[christoffel] g_inv determinant={:?}", g_inv.determinant());
        }
        gamma
    }

    fn step_along_null_geodesic(&self, photon: Photon, h: f32) -> Photon {
        // Integrate in the spherical coordinate chart to keep components and
        // Christoffel symbols in the same basis. Convert back to Cartesian
        // for the renderer/hit-testing.

        // Cartesian spatial pos/vel
            // Quick sanity check: if photon contains non-finite components, abort stepping.
            let pos_cart = photon.pos.space();
            if !(pos_cart.x.is_finite() && pos_cart.y.is_finite() && pos_cart.z.is_finite()) {
                return photon;
            }

        let vel_cart = photon.vel.space();

        // spherical position
        let sph_pos = self.to_spherical_coordinates(pos_cart);
        let pos_sph4 = SphVec4::new(photon.pos.time(), sph_pos.r(), sph_pos.theta(), sph_pos.phi());
        // eprintln!("[step] sph_pos: r={}, theta={}, phi={}", sph_pos.r(), sph_pos.theta(), sph_pos.phi());

        // convert spatial velocity (cartesian basis) -> spherical-basis components
        // Use coordinates relative to the metric center (the spherical chart origin).
        let (v_sph, phi_hint) = {
            let pos_rel = pos_cart - self.center;
            let x = pos_rel.x; let y = pos_rel.y; let z = pos_rel.z;
            let r = pos_rel.length();
            let rho = (x*x + y*y).sqrt();

            if r == 0.0 {
                (Vec3::ZERO, 0.0)
            } else {
                let vx = vel_cart.x; let vy = vel_cart.y; let vz = vel_cart.z;

                // dr/dx, dr/dy, dr/dz
                let dr_dx = x / r; let dr_dy = y / r; let dr_dz = z / r;

                if rho <= 1e-8 {
                    let pole_sign = if z >= 0.0 { 1.0 } else { -1.0 };
                    let tangential = (vx * vx + vy * vy).sqrt();
                    (
                        Vec3::new(
                            pole_sign * vz,
                            tangential / r.max(1e-8),
                            0.0,
                        ),
                        vy.atan2(vx),
                    )
                } else {
                    // dtheta/dx, dtheta/dy, dtheta/dz
                    let (dth_dx, dth_dy, dth_dz) = (
                        x * z / (r * r * rho),
                        y * z / (r * r * rho),
                        -rho / (r * r),
                    );

                    // dphi/dx, dphi/dy, dphi/dz
                    let (dph_dx, dph_dy) = (-y / (rho * rho), x / (rho * rho));

                    (
                        Vec3::new(
                            dr_dx * vx + dr_dy * vy + dr_dz * vz,
                            dth_dx * vx + dth_dy * vy + dth_dz * vz,
                            dph_dx * vx + dph_dy * vy,
                        ),
                        y.atan2(x),
                    )
                }
            }
        };

        // 4-vector in spherical components: (t, v_r, v_theta, v_phi)
        let mut x_sph = pos_sph4.as_vec4();
        let k_sph = Vec4::from_space_time(photon.vel[0], v_sph);
        if phi_hint != 0.0 {
            x_sph[3] = phi_hint;
        }
        // eprintln!("[step] k_sph before update: {:?}", k_sph);
        // eprintln!("[step] v_sph: {}", v_sph);

        // update position in spherical components
        for mu in 0..4 {
            x_sph[mu] += h * k_sph[mu];
        }

        // update k in spherical components using spherical Christoffels
        let mut k_new_sph = k_sph;
        for mu in 0..4 {
            let mut acc = 0.0;
            for alpha in 0..4 {
                for beta in 0..4 {
                    let gamma = self.christoffel_sph(pos_sph4, alpha, beta, mu);
                    acc += gamma * k_sph[alpha] * k_sph[beta];
                }
            }
            k_new_sph[mu] -= h * acc;
        }
        // eprintln!("[step] k_new_sph after update: {:?}", k_new_sph);
        // eprintln!("[step] x_sph after position update: {:?}", x_sph);

        // convert spherical-position back to cartesian space coords
        // x_sph[1] may have become negative due to numerical error; clamp it
        let r = x_sph[1].max(1e-8);
        let theta = x_sph[2];
        let phi = x_sph[3];
        let sin_th = theta.sin(); let cos_th = theta.cos();
        let sin_ph = phi.sin(); let cos_ph = phi.cos();

        let pos_cart_new = Vec3::new(
            r * sin_th * cos_ph,
            r * sin_th * sin_ph,
            r * cos_th,
        ) + self.center;

        // convert spherical-basis velocity components back to cartesian
        let v_sph_new = Vec3::new(k_new_sph[1], k_new_sph[2], k_new_sph[3]);
        let vx_new = (sin_th * cos_ph) * v_sph_new.x + (r * cos_th * cos_ph) * v_sph_new.y + (-r * sin_th * sin_ph) * v_sph_new.z;
        let vy_new = (sin_th * sin_ph) * v_sph_new.x + (r * cos_th * sin_ph) * v_sph_new.y + (r * sin_th * cos_ph) * v_sph_new.z;
        let vz_new = (cos_th) * v_sph_new.x + (-r * sin_th) * v_sph_new.y + 0.0 * v_sph_new.z;

        let vel_cart_new = Vec3::new(vx_new, vy_new, vz_new);

        Photon {
            pos: Vec4::from_space_time(x_sph[0], pos_cart_new),
            vel: Vec4::from_space_time(k_new_sph[0], vel_cart_new),
        }
    }
}