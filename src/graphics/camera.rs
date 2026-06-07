use crate::{MAX_INTEGRATION_STEPS, geometry::{manifold::PseudoRiemanian4Manifold, photon::Photon3, vector::TangentSpace}, graphics::texture::Textures, integration::geodesic_integrator::GeodesicIntegrator};
use crate::geometry::{point::Point3, vector::ThreeVector};
use crate::graphics::color::Color;
use crate::geometry::manifold::Chart::CartesianWorld;
use crate::graphics::surface::Object;

use rand::{rngs::StdRng, RngExt};

const SAMPLES_PER_PIXEL: u32 = if cfg!(debug_assertions) { 1 } else { 2 };

pub type Objects = Vec<Box<dyn Object>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
    pub textures: Textures,
}

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

        let center: Point3 = Point3::new(0.,0.,0., CartesianWorld);
        const FOCAL_LENGTH: f32 = 1.0;

        let viewport_height = 2.0;
        let viewport_width = viewport_height * a_ratio;
    
        let viewport_u_vect = ThreeVector::new(viewport_width, 0., 0., TangentSpace::CartesianWorld);
        let viewport_v_vect = ThreeVector::new(0., -viewport_height, 0., TangentSpace::CartesianWorld);

        let pixel_delta_u = viewport_u_vect * (1.0 / img_width as f32);
        let pixel_delta_v = viewport_v_vect * (1.0 / img_height as f32);

        let viewport_upper_left = center
            - Point3::new(0., 0., FOCAL_LENGTH, CartesianWorld)
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
        let integrator = GeodesicIntegrator::new(world, MAX_INTEGRATION_STEPS).unwrap();

        let mut rays = Vec::with_capacity((self.img_width * self.img_height * self.samples_per_pixel) as usize);

        for j in 0..self.img_height as usize {
            for i in 0..self.img_width as usize {
                for _ in 0..self.samples_per_pixel {
                    let ray_direction = (self.get_pixel_position(i, j, true, rng) - self.center).as_threevector();
                    rays.push(Photon3::new(self.center, ray_direction));
                }
            }
        }

        let manifold_rays: Vec<_> = rays.into_iter().map(|ray| world.manifold.world_photon3_to_photon4(ray)).collect();

        let (colors, trace) = integrator.run_kernel(manifold_rays);

        if let Some(trace_result) = trace.as_ref().and_then(|trace| trace.first()) {
            for position in trace_result.positions.iter().copied().filter(|point| point.fill_flag > 0.5).map(|point| point.pos) {
                println!("{:.6}, {:.6}, {:.6}", position[0], position[1], position[2]);
            }
        }

        for (pixel, samples) in frame
                .chunks_exact_mut(4)
                .zip(colors.chunks_exact(self.samples_per_pixel as usize)) {

            let mut color = Color::BLACK;
            for sample_color in samples {
                color += *sample_color;
            }

            color /= self.samples_per_pixel as f32;

            pixel[0] = color.r as u8; // R
            pixel[1] = color.g as u8; // G
            pixel[2] = color.b as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}