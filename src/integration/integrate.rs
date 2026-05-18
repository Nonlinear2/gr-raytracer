use crate::graphics::{ray::{WorldPhotonState, StopReason, WorldPhoton}, world::World};
const MAX_STEPS: u32 = 1000;
const STEP_SIZE: f32 = 0.01;

pub fn integrate(initial_ray: WorldPhoton, world: &World) -> (WorldPhotonState<'_>, StopReason) {
    let mut ray = world.manifold.create_photon(initial_ray.pos, initial_ray.vel);
    let mut world_ray = world.manifold.photon_to_world(ray);

    for _ in 0..MAX_STEPS {
        ray = world.manifold.step_along_null_geodesic(ray, STEP_SIZE);
        world_ray = world.manifold.photon_to_world(ray);

        if world.manifold.is_singular(ray.pos) {
            return (
                WorldPhotonState {
                    world_photon: world_ray,
                    normal: None,
                    material: None,
                },
                StopReason::HorizonHit
            );
        }

        if world_ray.pos.z() < -2. {
            return (
                WorldPhotonState {
                    world_photon: world_ray,
                    normal: None,
                    material: None,
                },
                StopReason::BackgroundReached,
            );
        }

        for obj in &world.objects {
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