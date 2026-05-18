use crate::graphics::{ray::{StopReason, WorldPhoton}, vector::{Point3, ThreeVector}, world::World};
use crate::graphics::color::Color;
use crate::integration::integrate::integrate;

use rand::RngExt;

const MAX_LIGHT_BOUNCES: u32 = 2;
const SAMPLES_PER_PIXEL: u32 = 1;

pub struct Camera {
    pub center: Point3,
    pub focal_length: f32,
    pub viewport_height: f32,
    pub viewport_width: f32,
    pub max_distance: f32,
    pub ray_step_size: f32,

    pub samples_per_pixel: u32,

    pub img_width: u32,
    pub img_height: u32,

    pub viewport_u_vect: ThreeVector,
    pub viewport_v_vect: ThreeVector,
    pub pixel_delta_u: ThreeVector,
    pub pixel_delta_v: ThreeVector,
    pub viewport_upper_left: Point3,
    pub first_pixel_loc: Point3,
}

impl Camera {
    pub fn new(img_width: u32, img_height: u32) -> Self {
        let a_ratio = (img_width as f32) / (img_height as f32);

        let center = Point3::new_cartesian(0.,0.,0.);
        let focal_length = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;
    
        let viewport_u_vect = ThreeVector::new_cartesian(viewport_width, 0., 0.);
        let viewport_v_vect = ThreeVector::new_cartesian(0., -viewport_height, 0.);

        let pixel_delta_u = viewport_u_vect * (1.0 / img_width as f32);
        let pixel_delta_v = viewport_v_vect * (1.0 / img_height as f32);

        let viewport_upper_left = center
            - ThreeVector::new_cartesian(0., 0., focal_length)
            - viewport_u_vect * 0.5
            - viewport_v_vect * 0.5;
        let first_pixel_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            center: center,
            focal_length: focal_length,
            viewport_height: viewport_height,
            viewport_width: viewport_width,
            max_distance: 5.,
            ray_step_size: 0.02,
            samples_per_pixel: SAMPLES_PER_PIXEL,
            img_width: img_width,
            img_height: img_height,

            viewport_u_vect: viewport_u_vect,
            viewport_v_vect: viewport_v_vect,
            pixel_delta_u: pixel_delta_u,
            pixel_delta_v: pixel_delta_v,
            viewport_upper_left: viewport_upper_left,
            first_pixel_loc: first_pixel_loc,
        }
    }

    #[allow(dead_code)]
    pub fn debug_ray_trajectory(&self, world: &World) {
        let i = self.img_width as usize / 4;
        let j = 0;
        let ray_direction = self.get_pixel_position(i, j, false) - self.center;
        let mut photon: crate::graphics::ray::Photon = world.manifold.create_photon(self.center, ray_direction);

        println!("x,y,z");

        for _step in 0..500 {
            let world_photon = world.manifold.photon_to_world(photon);
            let p = world_photon.pos;
            println!("{:.6}, {:.6}, {:.6}", p.x(), p.y(), p.z());

            if world.manifold.is_singular(photon.pos) {
                eprintln!("hit singularity");
                break;
            }

            photon = world.manifold.step_along_null_geodesic(photon, self.ray_step_size);
        }
    }

    pub fn ray_color(&self, ray: WorldPhoton, depth: u32, world: &World) -> Color {
        if depth <= 0 {
            return Color::BLACK;
        }

        let (hit, stop_reason) = integrate(ray, world);

        match stop_reason {
            StopReason::HorizonHit => return Color::BLACK,
            StopReason::BackgroundReached => {
                let tx = (hit.world_photon.pos.x() * 2.).floor() as i32;
                let ty = (hit.world_photon.pos.y() * 2.).floor() as i32;

                if (tx + ty) % 2 == 0 {
                    return Color::new(35.0, 35.0, 35.0);
                } else {
                    return Color::new(235.0, 235.0, 235.0);
                }
            },
            StopReason::MaxStepsReached => {
                Color { r: 255., g: 0., b: 0. }
            },
            StopReason::ObjectHit => {
                let material = hit.material.unwrap();
                if let Some((attenuation, new_direction)) = material.scatter(&hit) {
                    let ray = WorldPhoton::new(hit.world_photon.pos, new_direction);

                    let bounced = self.ray_color(ray, depth - 1, world);

                    return material.emission()
                        + Color {
                            r: attenuation.r * bounced.r / 255.0,
                            g: attenuation.g * bounced.g / 255.0,
                            b: attenuation.b * bounced.b / 255.0,
                        };
                }
                return material.emission();
            }
        }
    }

    pub fn get_pixel_position(&self, i: usize, j: usize, offset: bool) -> Point3 {
        let mut pos = self.first_pixel_loc + (self.pixel_delta_u * (i as f32)) + (self.pixel_delta_v * (j as f32));
        if offset {
            let mut rng = rand::rng();
            pos = pos
                + rng.random_range(-0.5..0.5) * self.pixel_delta_u
                + rng.random_range(-0.5..0.5) * self.pixel_delta_v;
        }
        pos
    }

    pub fn render(&self, frame: &mut [u8], world: &World) {
        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if idx % 100 == 0 {
                println!("pixels computed: {}", idx);
            }

            let i = idx % self.img_width as usize;
            let j = idx / self.img_width as usize;

            let mut color = Color::BLACK;
            for _ in 0..self.samples_per_pixel {
                let ray_direction = self.get_pixel_position(i, j, true) - self.center;

                let ray = WorldPhoton::new(self.center, ray_direction);

                color += self.ray_color(ray, MAX_LIGHT_BOUNCES, &world);
            }

            color /= self.samples_per_pixel as f32;

            pixel[0] = color.r as u8; // R
            pixel[1] = color.g as u8; // G
            pixel[2] = color.b as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}