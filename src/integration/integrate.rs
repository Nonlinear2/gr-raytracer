use crate::graphics::{ray::{WorldPhotonState, StopReason, WorldPhoton}, world::World};

const MAX_STEPS: u32 = 1000;

pub fn integrate(initial_ray: WorldPhoton, world: &World, debug: bool) -> (WorldPhotonState<'_>, StopReason) {
    let mut ray = world.manifold.create_photon(initial_ray.pos, initial_ray.vel);
    let mut world_ray = world.manifold.photon_to_world(ray);

    for _ in 0..MAX_STEPS {
        ray = world.manifold.step_along_null_geodesic(ray);
        world_ray = world.manifold.photon_to_world(ray);

        if debug {
            println!("vel sph: {:.6}, {:.6}, {:.6}", ray.vel.r(), ray.vel.theta(), ray.vel.phi());
            println!("vel: {:.6}, {:.6}, {:.6}", world_ray.vel.x(), world_ray.vel.y(), world_ray.vel.z());
            println!("{:.6}, {:.6}, {:.6}", world_ray.pos.x(), world_ray.pos.y(), world_ray.pos.z());
        }

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