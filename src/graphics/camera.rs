use crate::{config::{IMAGE_HEIGHT, IMAGE_WIDTH, SAMPLES_PER_PIXEL}, geometry::{manifold::PseudoRiemanian4Manifold, photon::Photon3, vector::TangentSpace}, graphics::texture::Textures, integration::geodesic_integrator::GeodesicIntegrator};
use crate::geometry::{point::Point3, vector::ThreeVector};
use crate::graphics::color::Color;
use crate::geometry::manifold::Chart::CartesianWorld;
use crate::graphics::surface::Object;

use rand::{rngs::StdRng, RngExt};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

pub type Objects = Vec<Box<dyn Object>>;

pub struct World {
    pub scene_size: f32,
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
    pub textures: Textures,
}

pub struct Camera {
    pub center: Point3,

    pub first_pixel_loc: Point3,
    pub pixel_delta_u: ThreeVector,
    pub pixel_delta_v: ThreeVector,
}

impl Camera {
    pub fn new() -> Self {
        let a_ratio = (IMAGE_WIDTH as f32) / (IMAGE_HEIGHT as f32);

        let center: Point3 = Point3::new(0.,0.,0., CartesianWorld);
        const FOCAL_LENGTH: f32 = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;
    
        let viewport_u_vect = ThreeVector::new(viewport_width, 0., 0., TangentSpace::CartesianWorld);
        let viewport_v_vect = ThreeVector::new(0., -viewport_height, 0., TangentSpace::CartesianWorld);

        let pixel_delta_u = viewport_u_vect * (1.0 / IMAGE_WIDTH as f32);
        let pixel_delta_v = viewport_v_vect * (1.0 / IMAGE_HEIGHT as f32);

        let viewport_upper_left = center
            - Point3::new(0., 0., FOCAL_LENGTH, CartesianWorld)
            - viewport_u_vect.as_point3() * 0.5
            - viewport_v_vect.as_point3() * 0.5;
        let first_pixel_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v).as_point3();

        Self {
            center: center,
            first_pixel_loc: first_pixel_loc,
            pixel_delta_u: pixel_delta_u,
            pixel_delta_v: pixel_delta_v,
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
        let integrator = GeodesicIntegrator::new(world).unwrap();

        let img_size = (IMAGE_WIDTH * IMAGE_HEIGHT) as usize;

        let mut image = vec![Color::BLACK; img_size];

        let progress_bar = ProgressBar::new(SAMPLES_PER_PIXEL as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{bar:40.cyan/blue} {pos}/{len} ETA: {eta}"
            )
            .unwrap()
        );
        progress_bar.tick();
    
        for sample_idx in 0..SAMPLES_PER_PIXEL {
            let mut rays = Vec::with_capacity(img_size);

            for j in 0..IMAGE_HEIGHT as usize {
                for i in 0..IMAGE_WIDTH as usize {
                    let ray_direction =
                        (self.get_pixel_position(i, j, true, rng) - self.center).as_threevector();
                    let world_photon = Photon3::new(self.center, ray_direction);

                    rays.push(world.manifold.world_photon3_to_photon4(world_photon));
                }
            }

            let (colors, trace) = integrator.run_kernel(rays);

            if sample_idx == 0 {
                if let Some(trace_result) = trace.as_ref().and_then(|trace| trace.first()) {
                    for position in trace_result
                        .positions
                        .iter()
                        .copied()
                        .filter(|point| point.fill_flag > 0.5)
                        .map(|point| point.pos)
                    {
                        println!("{:.6}, {:.6}, {:.6}", position[0], position[1], position[2]);
                    }
                }
            }

            for (acc, sample_color) in image.iter_mut().zip(colors.iter()) {
                *acc += *sample_color;
            }

            progress_bar.inc(1);
        }
        progress_bar.finish();

        for (pixel, color) in frame.chunks_exact_mut(4).zip(image.iter()) {
            let color = *color / SAMPLES_PER_PIXEL as f32;

            pixel[0] = color.r as u8;
            pixel[1] = color.g as u8;
            pixel[2] = color.b as u8;
            pixel[3] = 0xff;
        }
    }
}