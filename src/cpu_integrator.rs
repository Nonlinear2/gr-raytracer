use crate::{geometry::photon::{PackedTraceResult, Photon3}, graphics::{camera::World, color::Color}};

pub trait GeodesicIntegrator {
    fn new(world: &World) -> Self;
    fn run(&self, rays: Vec<Photon3>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>);
}

pub struct CpuIntegrator {
    world: &World,
}

impl CpuIntegrator {
    pub fn evolve_until_stop(&self, initial_ray: Photon3, debug: bool) -> (WorldPhoton3State<'_>, StopReason) {
        let mut ray = self.manifold.world_photon3_to_photon4(initial_ray);

        for _ in 0..MAX_STEPS {
            ray = self.manifold.step_along_null_geodesic(ray);

            if debug {
                let world_ray = self.manifold.to_world_photon3(ray);
                println!("vel sph: {:.6}, {:.6}, {:.6}", ray.vel.r(), ray.vel.theta(), ray.vel.phi());
                println!("vel: {:.6}, {:.6}, {:.6}", world_ray.vel.x(), world_ray.vel.y(), world_ray.vel.z());
                println!("{:.6}, {:.6}, {:.6}", world_ray.pos.x(), world_ray.pos.y(), world_ray.pos.z());
            }

            if self.manifold.is_singular(ray.pos) {
                return (
                    WorldPhoton3State {
                        photon3: self.manifold.to_world_photon3(ray),
                        normal: None,
                        material: None,
                    },
                    StopReason::HorizonHit
                );
            }

            if ray.pos.space().distance_to_zero() > config:: {
                return (
                    WorldPhoton3State {
                        photon3: self.manifold.to_world_photon3(ray),
                        normal: None,
                        material: None,
                    },
                    StopReason::BackgroundReached,
                );
            }

            let world_pos = self.manifold.transition_point(ray.pos.space(), Chart::CartesianWorld);
            for obj in &self.objects {
                if obj.hit(world_pos) {
                    let world_vel = self.manifold.transition_vector(
                        ray.pos.space(),
                        ray.vel.space(),
                        Chart::CartesianWorld,
                    );
                    return (obj.get_hit_data(&Photon3::new(world_pos, world_vel)), StopReason::ObjectHit);
                }
            }

            // check if we need to switch charts
            if self.manifold.preferred_chart_for_point(ray.pos.space()) != ray.pos.chart {
                // change photon chart
                let world_photon = self.manifold.to_world_photon3(ray);
                ray = self.manifold.world_photon3_to_photon4(world_photon);
            }
        }

        return (
            WorldPhoton3State {
                photon3: self.manifold.to_world_photon3(ray),
                normal: None,
                material: None,
            },
            StopReason::MaxStepsReached
        );
    }

}

impl GeodesicIntegrator for CpuIntegrator {
    fn new(world: &World) -> Option<Self> {
        Ok(Self { world })
    }

    fn run(&self, rays: Vec<Photon3>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>) {
        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if idx % 100 == 0 {
                println!("pixels computed: {}", idx);
            }

            let i = idx % self.img_width as usize;
            let j = idx / self.img_width as usize;

            let mut color = Color::BLACK;
            for _ in 0..self.samples_per_pixel {
                let ray_direction = (self.get_pixel_position(i, j, true, rng) - self.center).as_threevector();

                let ray = Photon3::new(self.center, ray_direction);

                color += self.ray_color(ray, MAX_LIGHT_BOUNCES, &world, false, rng);
            }

            color /= self.samples_per_pixel as f32;

            pixel[0] = color.r as u8; // R
            pixel[1] = color.g as u8; // G
            pixel[2] = color.b as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}