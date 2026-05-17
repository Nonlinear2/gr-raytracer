use crate::graphics::{ray::{Photon, PhotonIntersection, StopReason}, world::World};
const MAX_STEPS: u32 = 1000;
const STEP_SIZE: f32 = 0.01;

pub fn integrate(ray: Photon, world: &World) -> (Option<PhotonIntersection>, StopReason) {
    let mut ray_ = ray;
    for _ in 0..MAX_STEPS {
        ray_ = world.manifold.step_along_null_geodesic(ray_, STEP_SIZE);

        if world.manifold.is_singular(ray_.pos) {
            return (None, StopReason::HorizonHit);
        }

        for obj in &world.objects {
            if let Some(hit) = obj.hit(&ray_) {
                return (Some(hit), StopReason::ObjectHit);
            }
        }
    }

    return (None, StopReason::MaxStepsReached);
}