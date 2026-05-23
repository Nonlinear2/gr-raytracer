use crate::geometry::photon::{Photon3, Photon4, WorldPhoton3State, StopReason};
use crate::geometry::surface::Surface;
use crate::geometry::manifold::{Chart, PseudoRiemanian4Manifold};
use crate::graphics::geodesic_integrator::GpuGeodesicIntegrator;
use crate::SCENE_SIZE;

const MAX_STEPS: u32 = 1000;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
}

impl World {
    pub fn evolve_until_stop(&self, initial_rays: Vec<Photon3>, _debug: bool) -> Vec<(WorldPhoton3State<'_>, StopReason)> {
        assert!(self.objects.is_empty(), "gpu currently does not support object hits");

        let integrator = GpuGeodesicIntegrator::new(&*self.manifold).unwrap();

        let input_rays: Vec<Photon4> = initial_rays
            .into_iter()
            .map(|ray| self.manifold.world_photon3_to_photon4(ray))
            .collect();

        let results = integrator.integrate(input_rays).unwrap();

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