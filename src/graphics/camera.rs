use crate::graphics::{ray::Photon, vector::{self, FourVector, Point3}, world::World};
use crate::graphics::color::Color;
use glam::{Vec3, Vec4};
use rand::RngExt;
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

    pub viewport_u_vect: Vec3,
    pub viewport_v_vect: Vec3,
    pub pixel_delta_u: Vec3,
    pub pixel_delta_v: Vec3,
    pub viewport_upper_left: Point3,
    pub first_pixel_loc: Point3,
}

impl Camera {
    pub fn new(img_width: u32, img_height: u32) -> Self {
        let a_ratio = (img_width as f32) / (img_height as f32);

        let center = Point3{x: 0., y: 0., z: 0.};
        let focal_length = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;
    
        let viewport_u_vect = Vec3{x: viewport_width, y: 0., z: 0.};
        let viewport_v_vect = Vec3{x: 0., y: -viewport_height, z: 0.};

        let pixel_delta_u = viewport_u_vect / (img_width as f32);
        let pixel_delta_v = viewport_v_vect / (img_height as f32);

        let viewport_upper_left = center - Vec3{x: 0., y: 0., z: focal_length} - viewport_u_vect/2. - viewport_v_vect/2.;
        let first_pixel_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            center: center,
            focal_length: focal_length,
            viewport_height: viewport_height,
            viewport_width: viewport_width,
            max_distance: 5.,
            ray_step_size: 0.02,
            samples_per_pixel: 30,
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

    pub fn ray_color(&self, mut ray: Photon, depth: u32, world: &World) -> Color {
        if depth <= 0 {
            return Color {r: 0., g: 0., b: 0.};
        }

        while (ray.pos.space() - self.center).length() < self.max_distance {
            ray = world.metric.step_along_null_geodesic(ray, self.ray_step_size);
            for obj in &world.objects {
                if let Some(hit) = obj.hit(&ray, world.metric.g(ray.pos)) {
                    let mut scattered = Photon { pos: hit.point, vel: ray.vel };
                    let mut attenuation = Color { r: 0., g: 0., b: 0. };

                    if hit.material.scatter(&ray, &hit, &mut attenuation, &mut scattered) {
                        let bounced = self.ray_color(scattered, depth - 1, world);

                        return hit.material.emission()
                            + Color {
                                r: attenuation.r * bounced.r / 255.0,
                                g: attenuation.g * bounced.g / 255.0,
                                b: attenuation.b * bounced.b / 255.0,
                            };
                    }

                    return hit.material.emission();
                }
            }
        }

        let a = 0.5 * (ray.vel.normalize().y + 1.0);
        let mut col = (1.-a)*(Color {r: 255.0, g: 255.0, b: 255.0}) + a*(Color {r: 127.0, g: 190.0, b: 255.0});
        if ray.pos.x < 0. {
            col.b = 0.;
        }
        col
    }

    pub fn get_pixel_position(&self, i: usize, j: usize, offset: bool) -> Vec3 {
        let mut pos = self.first_pixel_loc + (self.pixel_delta_u * (i as f32)) + (self.pixel_delta_v * (j as f32));
        if offset {
            let mut rng = rand::rng();
            pos += rng.random_range(-0.5..0.5) * self.pixel_delta_u + rng.random_range(-0.5..0.5) * self.pixel_delta_v;
        }
        pos
    }

    pub fn render(&self, frame: &mut [u8], world: &World) {
        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let i = idx % self.img_width as usize;
            let j = idx / self.img_width as usize;

            let mut color = Color {r: 0., g: 0., b: 0.};
            for _ in 0..self.samples_per_pixel {
                let ray_direction = self.get_pixel_position(i, j, false) - self.center;

                let pos = Vec4::from_space_time(0., self.center);

                let ray = Photon::new(
                    world.metric.g(pos),
                    pos,
                    ray_direction
                );

                color += self.ray_color(ray, 3, &world);
            }

            color /= self.samples_per_pixel as f32;

            pixel[0] = color.r as u8; // R
            pixel[1] = color.g as u8; // G
            pixel[2] = color.b as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}