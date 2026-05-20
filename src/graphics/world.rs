use crate::geometry::point::Point3;
use crate::graphics::{surface::Surface};
use crate::geometry::manifold::PseudoRiemanian4Manifold;
use crate::{SCENE_SIZE, graphics::{ray::{StopReason, WorldPhoton, WorldPhotonState}}};

const MAX_STEPS: u32 = 1000;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub scene_center: Point3,
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
}

impl World {
    pub fn evolve_until_stop(&self, initial_ray: WorldPhoton, debug: bool) -> (WorldPhotonState<'_>, StopReason) {
        let mut ray = self.manifold.world_to_photon(initial_ray);
        let mut world_ray = self.manifold.photon_to_world(ray);

        for _ in 0..MAX_STEPS {
            ray = self.manifold.step_along_null_geodesic(ray);
            world_ray = self.manifold.photon_to_world(ray);

            if debug {
                println!("vel sph: {:.6}, {:.6}, {:.6}", ray.vel.r(), ray.vel.theta(), ray.vel.phi());
                println!("vel: {:.6}, {:.6}, {:.6}", world_ray.vel.x(), world_ray.vel.y(), world_ray.vel.z());
                println!("{:.6}, {:.6}, {:.6}", world_ray.pos.x(), world_ray.pos.y(), world_ray.pos.z());
            }

            if self.manifold.is_singular(ray.pos) {
                return (
                    WorldPhotonState {
                        world_photon: world_ray,
                        normal: None,
                        material: None,
                    },
                    StopReason::HorizonHit
                );
            }

            if (world_ray.pos - self.scene_center).distance_to_zero() > SCENE_SIZE {
                return (
                    WorldPhotonState {
                        world_photon: world_ray,
                        normal: None,
                        material: None,
                    },
                    StopReason::BackgroundReached,
                );
            }

            for obj in &self.objects {
                if let Some(hit) = obj.hit(&world_ray) {
                    return (hit, StopReason::ObjectHit);
                }
            }
        }

        return (
            WorldPhotonState {
                world_photon: world_ray,
                normal: None,
                material: None,
            },
            StopReason::MaxStepsReached
        );
    }
}