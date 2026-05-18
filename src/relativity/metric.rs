use crate::graphics::{ray::{WorldPhoton, Photon}, vector::{CoordinateSystem, FourVector, Point3, Point4, ThreeVector}};
use crate::integration::solvers::positive_root;
use glam::{Vec4, Mat4};


pub trait PseudoRiemanianManifold {
    // this trait only support manifolds with a single global chart, that we can access through
    // the world_to_chart function

    fn coordinate_system(&self) -> CoordinateSystem;

    #[allow(dead_code)]
    fn world_to_chart(&self, x: Point3) -> Point3;

    #[allow(dead_code)]
    fn chart_to_world(&self, x: Point3) -> Point3;

    fn is_singular(&self, x: Point4) -> bool;

    fn create_photon(&self, x: Point3, vel: ThreeVector) -> Photon;

    fn photon_to_world_pos(&self, photon: Photon) -> ThreeVector;

    fn photon_to_world_vel(&self, photon: Photon) -> ThreeVector;

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton;

    fn g(&self, x: Point4) -> Mat4;

    fn del_g(&self, x: Point4, i: u32) -> Mat4;

    fn christoffel(&self, pos: Point4, mu: usize, nu: usize, lambda: usize) -> f32;

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon;

}

#[allow(dead_code)]
pub struct Euclidean {
    pub center: Point3,
}


impl PseudoRiemanianManifold for Euclidean {

    fn coordinate_system(&self) -> CoordinateSystem {
        CoordinateSystem::Cartesian
    }

    fn world_to_chart(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        x - self.center
    }

    fn chart_to_world(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        x + self.center
    }

    fn is_singular(&self, _x: Point4) -> bool {
        false
    }

    fn create_photon(&self, x: Point3, vel: ThreeVector) -> Photon {
        Photon::new(
            FourVector::from_space_time(0., x),
            FourVector::from_space_time(0., vel)
        )
    }

    fn photon_to_world_pos(&self, photon: Photon) -> ThreeVector {
        assert!(photon.pos.coordinate_system == CoordinateSystem::Cartesian);
        assert!(photon.vel.coordinate_system == CoordinateSystem::Cartesian);

        photon.pos.space() + self.center
    }

    fn photon_to_world_vel(&self, photon: Photon) -> ThreeVector {
        assert!(photon.pos.coordinate_system == CoordinateSystem::Cartesian);
        assert!(photon.vel.coordinate_system == CoordinateSystem::Cartesian);

        photon.vel.space()
    }

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton {
        assert!(photon.pos.coordinate_system == CoordinateSystem::Cartesian);
        assert!(photon.vel.coordinate_system == CoordinateSystem::Cartesian);

        WorldPhoton::new(self.photon_to_world_pos(photon), self.photon_to_world_vel(photon))
    }

    fn g(&self, _x: Point4) -> Mat4 {
        Mat4::IDENTITY
    }

    fn del_g(&self, _x: Point4, _i: u32) -> Mat4 {
        Mat4::ZERO
    }

    fn christoffel(&self, _pos: Point4, _mu: usize, _nu: usize, _lambda: usize) -> f32 {
        0.
    }

    fn step_along_null_geodesic(&self, s: Photon, h: f32) -> Photon {
        Photon::new(s.pos + s.vel * h, s.vel)
    }
}

pub struct Schwarzschild {
    pub center: Point3,
    pub r_s: f32,
}

impl Schwarzschild {
    pub fn new(center: Point3, r_s: f32) -> Self {
        assert!(center.coordinate_system == CoordinateSystem::Cartesian);
        Self {
            center: center,
            r_s: r_s,
        }
    }

    #[allow(dead_code)]
    pub fn mass(&self) -> f32 { // schwartzschild radius
        return self.r_s; // r_s = 2GM/c^2.
    }
}

impl PseudoRiemanianManifold for Schwarzschild {
    fn coordinate_system(&self) -> CoordinateSystem {
        CoordinateSystem::Spherical
    }

    fn world_to_chart(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Cartesian);
        (x - self.center).to_spherical()
    }

    fn chart_to_world(&self, x: Point3) -> Point3 {
        assert!(x.coordinate_system == CoordinateSystem::Spherical);
        x.to_cartesian() + self.center
    }

    fn is_singular(&self, x: Point4) -> bool {
        x.r() <= self.r_s
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

        let (v_r, v_th, v_ph, _phi_hint) = if r <= 1e-8 {
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

    fn photon_to_world_pos(&self, photon: Photon) -> ThreeVector {
        assert!(photon.pos.coordinate_system == CoordinateSystem::Spherical);
        assert!(photon.vel.coordinate_system == CoordinateSystem::Spherical);

        photon.pos.space().to_cartesian() + self.center
    }

    fn photon_to_world_vel(&self, photon: Photon) -> ThreeVector {
        assert!(photon.pos.coordinate_system == CoordinateSystem::Spherical);
        assert!(photon.vel.coordinate_system == CoordinateSystem::Spherical);

        let pos = photon.pos.space();
        let vel = photon.vel.space();

        let r = pos.r();
        let theta = pos.theta();
        let phi = pos.phi();
        let v_r = vel.inner[0];
        let v_theta = vel.inner[1];
        let v_phi = vel.inner[2];

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        ThreeVector::new_cartesian(
            sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
            sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
            cos_theta * v_r - r * sin_theta * v_theta,
        )
    }

    fn photon_to_world(&self, photon: Photon) -> WorldPhoton {
        WorldPhoton::new(
            self.photon_to_world_pos(photon),
            self.photon_to_world_vel(photon),
        )
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
        // Avoid computing the inverse metric exactly at the coordinate singularities
        // (the poles) where g contains factors of sin(theta)->0 causing g^{-1}
        // to blow up. Perturb theta slightly when it is too close to 0 or PI.
        let mut pos_for_metric = pos;
        let theta = pos.theta();
        let eps: f32 = 1e-6;
        if theta <= eps || theta >= std::f32::consts::PI - eps {
            let theta_adj = if theta <= eps { eps } else { std::f32::consts::PI - eps };
            // eprintln!("[christoffel] perturbing theta from {} to {} to avoid pole", theta, theta_adj);
            pos_for_metric = FourVector::new_spherical(pos.t(), pos.r(), theta_adj, pos.phi());
        }

        let g_inv = self.g(pos_for_metric).inverse();
        let mut gamma = 0.;

        let d_mu_g = self.del_g(pos_for_metric, mu as u32);
        let d_nu_g = self.del_g(pos_for_metric, nu as u32);

        for alpha in 0..4 {
            let d_alpha_g = self.del_g(pos_for_metric, alpha as u32);

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
        let x = photon.pos.as_vec4();
        let mut k = photon.vel.as_vec4();

        // 1. compute velocity update using CURRENT position
        for mu in 0..4 {
            let mut acc = 0.0_f32;
            for alpha in 0..4 {
                for beta in 0..4 {
                    let gamma = self.christoffel(photon.pos, alpha, beta, mu);
                    acc += gamma * k[alpha] * k[beta];
                }
            }
            k[mu] -= h * acc;
        }

        // 2. update position using CURRENT (pre-step) velocity
        let mut new_x = x;
        for mu in 0..4 {
            new_x[mu] += h * photon.vel.as_vec4()[mu];
        }

        // 3. fix up spherical coordinates
        let mut r     = new_x[1];
        let mut theta = new_x[2];
        let mut phi   = new_x[3];

        if r < 0.0 {
            r = -r;
            theta = std::f32::consts::PI - theta;
            k[1] = -k[1];
            k[2] = -k[2];
            phi += std::f32::consts::PI;
        }

        while theta < 0.0 {
            theta = -theta;
            k[2] = -k[2];
            phi += std::f32::consts::PI;
        }
        while theta > std::f32::consts::PI {
            theta = 2.0 * std::f32::consts::PI - theta;
            k[2] = -k[2];
            phi += std::f32::consts::PI;
        }

        new_x[1] = r;
        new_x[2] = theta.clamp(0.0, std::f32::consts::PI);
        new_x[3] = phi.rem_euclid(std::f32::consts::TAU);

        Photon {
            pos: FourVector { inner: new_x, coordinate_system: photon.pos.coordinate_system },
            vel: FourVector { inner: k,     coordinate_system: photon.vel.coordinate_system },
        }
    }
}