use crate::geometry::photon::{Photon3, Photon4};
use crate::geometry::surface::PackedGpuObject;
use crate::geometry::surface::Surface;
use crate::geometry::manifold::{PseudoRiemanian4Manifold};
use crate::graphics::geodesic_integrator::GpuGeodesicIntegrator;
use crate::graphics::color::Color;

pub type Objects = Vec<Box<dyn Surface>>;

pub struct World {
    pub manifold: Box<dyn PseudoRiemanian4Manifold>,
    pub objects: Objects,
}

impl World {
    pub fn evolve_until_stop(&self, initial_rays: Vec<Photon3>) -> Vec<Color> {
        let packed_objects: Vec<PackedGpuObject> = self.objects.iter().filter_map(|obj| obj.as_packed_gpu_object()).collect();
        let integrator = GpuGeodesicIntegrator::new(&*self.manifold, &packed_objects).unwrap();

        let input_rays: Vec<Photon4> = initial_rays
            .into_iter()
            .map(|ray| self.manifold.world_photon3_to_photon4(ray))
            .collect();

        integrator.integrate(input_rays).unwrap()
    }
}