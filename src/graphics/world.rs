use crate::geometry::photon::{Photon3, Photon4, WorldPhoton3State, StopReason};
use crate::geometry::surface::Surface;
use crate::geometry::manifold::{Chart, PseudoRiemanian4Manifold};
use crate::graphics::gpu::GpuGeodesicIntegrator;
use crate::SCENE_SIZE;

const MAX_STEPS: u32 = 1000;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
}

impl World {
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

            if ray.pos.space().distance_to_zero() > SCENE_SIZE {
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

    pub fn evolve_until_stop_gpu(&self, initial_rays: Vec<Photon3>, _debug: bool) -> Vec<(WorldPhoton3State<'_>, StopReason)> {
        assert!(self.objects.is_empty(), "gpu currently does not support object hits");

        let stepper = GpuGeodesicIntegrator::new(&*self.manifold).expect("failed to initialize GPU compute pipeline");
        let input_rays: Vec<Photon4> = initial_rays
            .into_iter()
            .map(|ray| self.manifold.world_photon3_to_photon4(ray))
            .collect();

        let results = stepper.evolve_batch(&input_rays).expect("GPU evolution failed");

        results
            .into_iter()
            .map(|(photon, stop_reason)| {
                (
                    WorldPhoton3State {
                        photon3: self.manifold.to_world_photon3(photon),
                        normal: None,
                        material: None,
                    },
                    stop_reason,
                )
            })
            .collect()
    }
}