use crate::graphics::{ray::{StopReason, WorldPhoton}, point::Point3, vector::ThreeVector, world::World};
use crate::graphics::color::Color;
use crate::integration::integrate::integrate;

use rand::{rngs::StdRng, RngExt};

const MAX_LIGHT_BOUNCES: u32 = 2;
const SAMPLES_PER_PIXEL: u32 = 1;

pub struct Camera {
    pub center: Point3,
    pub samples_per_pixel: u32,

    #[allow(dead_code)]
    pub img_width: u32,
    #[allow(dead_code)]
    pub img_height: u32,

    pub first_pixel_loc: Point3,
    pub pixel_delta_u: ThreeVector,
    pub pixel_delta_v: ThreeVector,
}

impl Camera {
    pub fn new(img_width: u32, img_height: u32) -> Self {
        let a_ratio = (img_width as f32) / (img_height as f32);

        let center: Point3 = Point3::new_cartesian(0.,0.,0.);
        const FOCAL_LENGTH: f32 = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;
    
        let viewport_u_vect = ThreeVector::new_cartesian(viewport_width, 0., 0.);
        let viewport_v_vect = ThreeVector::new_cartesian(0., -viewport_height, 0.);

        let pixel_delta_u = viewport_u_vect * (1.0 / img_width as f32);
        let pixel_delta_v = viewport_v_vect * (1.0 / img_height as f32);

        let viewport_upper_left = center
            - Point3::new_cartesian(0., 0., FOCAL_LENGTH)
            - viewport_u_vect.as_point3() * 0.5
            - viewport_v_vect.as_point3() * 0.5;
        let first_pixel_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v).as_point3();

        Self {
            center: center,
            samples_per_pixel: SAMPLES_PER_PIXEL,
            img_width: img_width,
            img_height: img_height,

            first_pixel_loc: first_pixel_loc,
            pixel_delta_u: pixel_delta_u,
            pixel_delta_v: pixel_delta_v,
        }
    }

    pub fn ray_color(&self, ray: WorldPhoton, depth: u32, world: &World, debug: bool, rng: &mut StdRng) -> Color {
        if depth <= 0 {
            return Color::BLACK;
        }

        let (hit, stop_reason) = integrate(ray, world, debug);

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
                if let Some((attenuation, new_direction)) = material.scatter(&hit, rng) {
                    let ray = WorldPhoton::new(hit.world_photon.pos, new_direction);

                    let bounced = self.ray_color(ray, depth - 1, world, debug, rng);

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

    pub fn get_pixel_position(&self, i: usize, j: usize, offset: bool, rng: &mut StdRng) -> Point3 {
        let mut pos = self.first_pixel_loc + (self.pixel_delta_u * (i as f32) + self.pixel_delta_v * (j as f32)).as_point3();
        if offset {
            pos = pos
                + rng.random_range(-0.5..0.5) * self.pixel_delta_u.as_point3()
                + rng.random_range(-0.5..0.5) * self.pixel_delta_v.as_point3();
        }
        pos
    }

    pub fn render(&self, frame: &mut [u8], world: &World, rng: &mut StdRng) {
        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if idx % 1 == 0 {
                println!("pixels computed: {}", idx);
            }

            let i = idx % self.img_width as usize;
            let j = idx / self.img_width as usize;

            let mut color = Color::BLACK;
            for _ in 0..self.samples_per_pixel {
                let ray_direction = (self.get_pixel_position(i, j, true, rng) - self.center).as_threevector();

                let ray = WorldPhoton::new(self.center, ray_direction);

                color += self.ray_color(ray, MAX_LIGHT_BOUNCES, &world, idx == 4385, rng);
            }

            color /= self.samples_per_pixel as f32;

            pixel[0] = color.r as u8; // R
            pixel[1] = color.g as u8; // G
            pixel[2] = color.b as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}