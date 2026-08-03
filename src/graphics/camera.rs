use crate::{
    config::{self, IMAGE_HEIGHT, IMAGE_WIDTH, SAMPLES_PER_PIXEL}, constants::Pipeline::CPU, geometry::{chart::IsChart, manifold::Manifold, metric::Metric, photon::Photon3}, integrator::{GeodesicIntegrator, cpu::CpuIntegrator, gpu::GpuIntegrator}, scene::World,
};

use crate::geometry::{point::{Point3, Point4}, vector::ThreeVector};
use crate::graphics::color::Color;

use glam::{Quat, Vec3};
use rand::{rngs::StdRng, RngExt};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

pub struct Camera<C: IsChart> {
    pub center: Point3<C>,

    pub first_pixel_dir: ThreeVector<C>,
    pub pixel_delta_x: ThreeVector<C>,
    pub pixel_delta_y: ThreeVector<C>,
}

impl<C: IsChart> Camera<C> {
    pub fn new<M: Manifold<C>>(manifold: &M, center: Point3<C>, orientation: Quat) -> Self {
        let a_ratio = (IMAGE_WIDTH as f32) / (IMAGE_HEIGHT as f32);

        const FOCAL_LENGTH: f32 = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;

        let (e_1, e_2, e_3) = manifold.orthonormal_frame(Point4::from_space_time(0.0, center));
        let rotated = |d: Vec3| {
            let d = orientation.mul_vec3(d);
            e_1 * d.x + e_2 * d.y + e_3 * d.z
        };

        let u = rotated(Vec3::X);
        let v = rotated(Vec3::Y) * viewport_width;
        let w = rotated(Vec3::Z) * (-viewport_height);

        let pixel_delta_x = v * (1.0 / IMAGE_WIDTH as f32);
        let pixel_delta_y = w * (1.0 / IMAGE_HEIGHT as f32);

        let viewport_upper_left = u * FOCAL_LENGTH - v * 0.5 - w * 0.5;
        let first_pixel_dir = viewport_upper_left + (pixel_delta_x + pixel_delta_y) * 0.5;

        Self {
            center: center,
            first_pixel_dir: first_pixel_dir,
            pixel_delta_x: pixel_delta_x,
            pixel_delta_y: pixel_delta_y,
        }
    }

    pub fn get_pixel_direction(&self, i: usize, j: usize, offset: bool, rng: &mut StdRng) -> ThreeVector<C> {
        let mut dir = self.first_pixel_dir
            + self.pixel_delta_x * (i as f32)
            + self.pixel_delta_y * (j as f32);
        if offset {
            dir = dir
                + self.pixel_delta_x * rng.random_range(-0.5..0.5)
                + self.pixel_delta_y * rng.random_range(-0.5..0.5);
        }
        dir
    }

    pub fn render(&self, frame: &mut [u8], world: &World, rng: &mut StdRng) {
        let integrator: Box<dyn GeodesicIntegrator + '_> = if config::PIPELINE == CPU {
            Box::new(CpuIntegrator::new(world))
        } else {
            Box::new(GpuIntegrator::new(world))
        };

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
                    let ray_direction = self.get_pixel_direction(i, j, true, rng);
                    let world_photon = Photon3::new(self.center, ray_direction);

                    rays.push(world_photon);
                }
            }

            let (colors, trace) = integrator.run(rays);

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

            pixel[0] = (color.r * 255.0) as u8;
            pixel[1] = (color.g * 255.0) as u8;
            pixel[2] = (color.b * 255.0) as u8;
            pixel[3] = 0xff;
        }
    }
}