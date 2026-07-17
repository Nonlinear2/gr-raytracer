use std::cell::RefCell;

use bytemuck::Zeroable;
use indicatif::{ProgressBar, ProgressStyle};
use rand::{rngs::StdRng, SeedableRng};

use crate::config;
use crate::geometry::manifold::ChartWorld;
use crate::geometry::photon::Photon3;
use crate::geometry::point::Point3;
use crate::graphics::color::Color;
use crate::integrator::{GeodesicIntegrator, PackedTracePoint, PackedTraceResult};
use crate::math::sphere_uv;
use crate::scene::texture::TextureId;
use crate::scene::World;

pub struct CpuIntegrator<'a> {
    world: &'a World,
    rng: RefCell<StdRng>,
}

impl<'a> CpuIntegrator<'a> {
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            rng: RefCell::new(StdRng::seed_from_u64(config::RNG_SEED)),
        }
    }

    fn sky_albedo(&self, world_pos: Point3<ChartWorld>) -> Color {
        let direction = (world_pos - self.world.manifold.subatlas_center()).as_threevector().normalize();
        let (u, v) = sphere_uv(direction);
        let rgba = self.world.textures.get(TextureId::SKY).sample(u, v);
        Color::new(rgba[0], rgba[1], rgba[2])
    }

    fn evolve_ray(
        &self,
        input_ray: Photon3,
        rng: &mut StdRng,
        mut trace: Option<&mut PackedTraceResult>,
    ) -> Color {
        let manifold = self.world.manifold.as_ref();

        let mut ray = manifold.world_photon3_to_photon4(input_ray);
        let mut bounce_count = 0u32;
        let mut radiance = Color::BLACK;
        let mut throughput = Color::WHITE;

        for step in 0..config::MAX_INTEGRATION_STEPS {
            let prev_ray = ray;
            ray = manifold.step_along_null_geodesic(ray);

            let prev_world_pos = manifold.point_to_world(prev_ray.pos.space());
            let world_pos = manifold.point_to_world(ray.pos.space());

            if let Some(trace) = trace.as_deref_mut() {
                trace.positions[step as usize] = PackedTracePoint {
                    pos: [world_pos.x(), world_pos.y(), world_pos.z()],
                    fill_flag: 1.0,
                };
            }

            if manifold.is_close_to_singular(ray.pos) {
                return radiance;
            }

            if (world_pos - manifold.subatlas_center()).as_threevector().length() > self.world.scene_size { // sky reached
                return radiance + throughput * self.sky_albedo(world_pos);
            }

            let preferred_chart = manifold.preferred_chart_for_point(world_pos);
            if preferred_chart != ray.pos.chart {
                ray = manifold.world_photon3_to_photon4(manifold.to_world_photon3(ray));
            }

            for object in &self.world.objects {
                let Some(hit_data) = object.hit(prev_world_pos, world_pos) else {
                    continue;
                };

                let texture_rgba = if object.texture() != TextureId::NONE {
                    let (u, v) = object.uv(hit_data.hit_point);
                    Some(self.world.textures.get(object.texture()).sample(u, v))
                } else {
                    None
                };

                let alpha = match texture_rgba {
                    Some(rgba) => rgba[3].clamp(0.0, 1.0),
                    None => 1.0,
                };

                if alpha <= 0.0 { // transparent material
                    continue;
                }

                if bounce_count >= config::MAX_BOUNCES {
                    return radiance;
                }

                let texture_rgb = texture_rgba.map(|rgba| Color::new(rgba[0], rgba[1], rgba[2]));
                let material = object.material();
                let scatter_direction = material.scatter(&hit_data, rng);

                if alpha >= 1.0 { // opaque material
                    radiance += throughput * material.emission(texture_rgb);
                    throughput *= material.albedo(texture_rgb);

                    let Some(direction) = scatter_direction else { // no bounce
                        return radiance;
                    };

                    ray = manifold.world_photon3_to_photon4(Photon3::new(hit_data.hit_point, direction));
                    bounce_count += 1;
                    break;

                } else { // semi transparent
                    // the ray continues with throughput scaled by (1 - alpha) representing the transmitted fraction.
                    radiance += throughput * alpha * material.emission(texture_rgb);
                    throughput *= 1.0 - alpha;
                    bounce_count += 1;
                }
            }
        }

        Color::new(1.0, 0.0, 0.0) // show rays that didnt hit anything in red for debugging
    }
}

impl GeodesicIntegrator for CpuIntegrator<'_> {
    fn run(&self, rays: Vec<Photon3>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>) {
        let mut rng_guard = self.rng.borrow_mut();
        let rng = &mut *rng_guard;

        let mut trace_results = if config::DEBUG {
            Some(vec![PackedTraceResult::zeroed()])
        } else {
            None
        };

        let progress_bar = ProgressBar::new(rays.len() as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{bar:40.green/blue} {pos}/{len} ETA: {eta}"
            )
            .unwrap()
        );
        progress_bar.tick();

        let colors = rays
            .iter()
            .enumerate()
            .map(|(ray_index, ray)| {
                let trace = trace_results
                    .as_mut()
                    .filter(|_| ray_index as u32 == config::DEBUG_RAY_INDEX)
                    .and_then(|traces| traces.first_mut());
                let color = self.evolve_ray(*ray, rng, trace);
                progress_bar.inc(1);
                color
            })
            .collect();

        progress_bar.finish_and_clear();

        (colors, trace_results)
    }
}
