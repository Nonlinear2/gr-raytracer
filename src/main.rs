mod graphics;
mod geometry;
mod integration;
mod integrator;
mod math;
mod config;
mod constants;
mod scene;

use crate::graphics::window::App;
use crate::scene::texture::{Textures, Texture, TextureId};
#[allow(unused_imports)]
use crate::scene::surface::{Diffuse, Disc, Metal, Sphere};
use crate::scene::World;
use crate::graphics::camera::Camera;
use crate::graphics::color::Color;

use crate::geometry::vector::ThreeVector;
use crate::geometry::point::Point3;
#[allow(unused_imports)]
use crate::geometry::schwarzschild::Schwarzschild4Manifold;
#[allow(unused_imports)]
use crate::geometry::euclidean::Euclidean4Manifold;
use crate::geometry::manifold::{ChartWorld, TangentWorld};

use rand::{rngs::StdRng, SeedableRng};
use winit::event_loop::{ControlFlow, EventLoop};

use std::time::Instant;

fn main() {
    env_logger::init();

    let world = World {
        scene_size: 6.0,
        manifold: Box::new(Schwarzschild4Manifold::new(
            Point3::new(0., 0., -2., ChartWorld),
            0.25
        )),
        textures: {
            let accretion = Texture::from_file("assets/textures/accretion.png", 1.0).expect("failed to load texture");
            let sky = Texture::from_file("assets/textures/space_sky.jpg", 0.6).expect("failed to load texture");
            Textures::new(accretion, sky)
        },
        objects: vec![
            Box::new(Sphere {
                center: Point3::new(-0.6, 0.14, -0.7, ChartWorld),
                radius: 0.02,
                material: Box::new(Diffuse {
                    color: Color::new(61.0 / 255.0, 34.0 / 255.0, 17.0 / 255.0),
                    emission: Color::new(0., 0., 0.),
                }),
                texture: TextureId::NONE,
            }),
            Box::new(Disc {
                center: Point3::new(0.0, 0.0, -2.0, ChartWorld),
                normal: ThreeVector::new(0.09, 0.8, 0.1, TangentWorld),
                // normal: ThreeVector::new(0.0, 0.0, 1., geometry::vector::TangentSpace::CartesianWorld),
                radius: 1.5,
                inner_radius: 0.75,
                material: Box::new(Diffuse { 
                    // color: Color::new(128., 128., 128.),
                    // emission: Color::new(237.0, 193.0, 154.0),
                    color: Color::WHITE,
                    emission: Color::WHITE,
                }),
                texture: TextureId::ACCRETION,
            })
        ],
    };

    let mut buffer = vec![0u8; (config::IMAGE_WIDTH * config::IMAGE_HEIGHT * 4) as usize];

    let camera: Camera = Camera::new();

    let start = Instant::now();
    let mut rng = StdRng::seed_from_u64(config::RNG_SEED);

    camera.render(buffer.as_mut_slice(), &world, &mut rng);

    let elapsed = start.elapsed();
    println!("Elapsed time: {:?}", elapsed);

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(buffer, config::IMAGE_WIDTH, config::IMAGE_HEIGHT);

    event_loop.run_app(&mut app).unwrap();
}
