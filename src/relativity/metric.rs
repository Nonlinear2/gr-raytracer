use crate::graphics::{ray::Photon, vector::{CoordinateSystem, FourVector, Point3, Point4, ThreeVector}, world::{Objects, World}};
use crate::graphics::ray::StopReason;
use crate::integration::solvers::positive_root;
use glam::{Vec4, Mat4, Vec3};


pub trait PseudoRiemanianManifold {
    // this trait only support manifolds with a single global chart, that we can access through
    // the world_to_chart function

    fn coordinate_system(&self) -> CoordinateSystem;

    fn is_singular(&self, x: Point4) -> bool;

    fn create_photon(&self, x: Point3, vel: ThreeVector) -> Photon;

    fn world_to_chart(&self, x: Point3) -> Point3;

    fn chart_to_world(&self, x: Point3) -> Point3;

    fn g(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon;

    // fn dot(&self, x: Point4, v1: Vec4, v2: Vec4) -> f32 {
    //     v1.dot(self.g(x) * v2)
    // }

    // fn norm(&self, x: Point4, v1: Vec4) -> f32 {
    //     self.dot(x, v1, v1)
    // }
}

pub struct Euclidean {
    pub center: Point3,
}


impl PseudoRiemanianManifold for Euclidean {

    fn coordinate_system(&self) -> CoordinateSystem {
        CoordinateSystem::Cartesian
    }

    fn is_singular(&self, x: Point4) -> bool {
        false
    }

    fn create_photon(&self, x: Point3, vel: ThreeVector) -> Photon {
        Photon::new(
            FourVector::from_space_time(0., x),
            FourVector::from_space_time(0., vel)
        )
    }

    fn world_to_chart(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        x - self.center
    }

    fn chart_to_world(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        x + self.center
    }

    fn g(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn del_g(&self, x: Point4, i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32 {
        0.
    }

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon {
        Photon::new(s.pos + s.vel * h, s.vel)
    }
}

pub struct Schwartzschild {
    pub center: Point3,
    pub r_s: f32,
}

impl Schwartzschild {
    pub fn new(center: Point3, r_s: f32) -> Self {
        assert!(center.coordinate_system == CoordinateSystem::Cartesian);
        Self {
            center: center,
            r_s: r_s,
        }
    }

    pub fn mass(&self) -> f32 { // schwartzschild radius
        return self.r_s; // r_s = 2GM/c^2.
    }
}

impl PseudoRiemanianManifold for Schwartzschild {
    fn coordinate_system(&self) -> CoordinateSystem {
        CoordinateSystem::Spherical
    }

    fn is_singular(&self, x: Point4) -> bool {
        (x.space() - self.center).length() <= self.r_s    
    }

    fn create_photon(&self, x: Point3, vel: ThreeVector) -> Photon {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        assert!(vel.coordinate_system == CoordinateSystem::Cartesian);

        let rel = x - self.center;
        let x = rel.x();
        let y = rel.y();
        let z = rel.z();
        let rho = (x * x + y * y).sqrt();
        let pos = rel.to_spherical();
        let r = pos.r();
        let theta = pos.theta();
        let phi= pos.phi();

        let (v_r, v_th, v_ph, phi_hint) = if r <= 1e-8 {
            (0.0, 0.0, 0.0, 0.0)
        } else if rho <= 1e-8 {
            let pole_sign = if z >= 0.0 { 1.0 } else { -1.0 };
            let tangential = (vel.x() * vel.x() + vel.y() * vel.y()).sqrt();
            (
                pole_sign * vel.z(),
                tangential / r.max(1e-8),
                0.0,
                vel.y().atan2(vel.x()).rem_euclid(std::f32::consts::TAU),
            )
        } else {
            let dr_dx = x / r;
            let dr_dy = y / r;
            let dr_dz = z / r;
            let dth_dx = x * z / (r * r * rho);
            let dth_dy = y * z / (r * r * rho);
            let dth_dz = -rho / (r * r);
            let dph_dx = -y / (rho * rho);
            let dph_dy = x / (rho * rho);
            (
                dr_dx * vel.x() + dr_dy * vel.y() + dr_dz * vel.z(),
                dth_dx * vel.x() + dth_dy * vel.y() + dth_dz * vel.z(),
                dph_dx * vel.x() + dph_dy * vel.y(),
                phi,
            )
        };

        let vel_sph = ThreeVector::new(v_r, v_th, v_ph, CoordinateSystem::Spherical);
        let pos = if rho <= 1e-8 {
            let phi_hint = vel.y().atan2(vel.x()).rem_euclid(std::f32::consts::TAU);
            ThreeVector::new_spherical(r, theta, phi_hint)
        } else {
            pos
        };

        let pos_sph4 = FourVector::from_space_time(0.0, pos);
        let g = self.g(pos_sph4);
        let b = 2.0 * (g.col(0)[1] * v_r + g.col(0)[2] * v_th + g.col(0)[3] * v_ph);
        let mut c = 0.0;
        c += g.col(1)[1] * v_r * v_r;
        c += 2.0 * g.col(1)[2] * v_r * v_th;
        c += 2.0 * g.col(1)[3] * v_r * v_ph;
        c += g.col(2)[2] * v_th * v_th;
        c += 2.0 * g.col(2)[3] * v_th * v_ph;
        c += g.col(3)[3] * v_ph * v_ph;

        let k_0 = positive_root(g.col(0)[0], b, c);

        Photon::new(FourVector::from_space_time(0.0, pos), FourVector::from_space_time(k_0, vel_sph))
    }

    fn world_to_chart(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        (x - self.center).to_spherical()
    }

    fn chart_to_world(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        x.to_cartesian() + self.center
    }

    fn g(&self, pos: Point4) -> Mat4 {
        assert!(pos.coordinate_system == self.coordinate_system());

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
        assert!(pos.coordinate_system == self.coordinate_system());

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
        assert!(pos.coordinate_system == self.coordinate_system());

        let g_inv = self.g(pos).inverse();
        let mut gamma = 0.;

        let d_mu_g = self.del_g(pos, mu as u32);
        let d_nu_g = self.del_g(pos, nu as u32);

        for alpha in 0..4 {
            let d_alpha_g = self.del_g(pos, alpha as u32);

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