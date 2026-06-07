mod graphics;
mod geometry;
mod integration;

use winit::{
    dpi::PhysicalSize,
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window
};
use pixels::{Pixels, SurfaceTexture};
use crate::graphics::texture::Textures;
use crate::graphics::texture::Texture;
use crate::graphics::texture::TextureId;
use crate::graphics::surface::{Diffuse, Disc, Metal, Sphere};
use crate::geometry::vector::ThreeVector;
use crate::graphics::camera::{Camera, World};
use crate::graphics::color::Color;
use crate::geometry::point::Point3;
use crate::geometry::schwarzschild::Schwarzschild4Manifold;
use crate::geometry::manifold::Chart::CartesianWorld;

use rand::{rngs::StdRng, SeedableRng};
use std::time::Instant;

const HEIGHT: u32 = 300;
const WIDTH: u32 = ((HEIGHT as f32) * 16.0 / 9.0) as u32;
const RNG_SEED: u64 = 0;

const MAX_INTEGRATION_STEPS: u32 = 1000;

#[derive(Default)]
struct App {
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    frame: Vec<u8>,
    cursor_position: Option<(f64, f64)>,
}

impl App {
    fn new(frame: Vec<u8>) -> Self {
        Self {
            window: None,
            pixels: None,
            frame,
            cursor_position: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("")
                    .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
                    .with_resizable(false)
            )
            .unwrap();

        let size = window.inner_size();
        let window_ref: &'static Window = Box::leak(Box::new(window));
        let surface_texture = SurfaceTexture::new(size.width, size.height, window_ref);

        let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();
        
        pixels.frame_mut().copy_from_slice(&self.frame);

        self.window = Some(window_ref);
        self.pixels = Some(pixels);
        window_ref.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some((position.x, position.y));
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() && matches!(event.logical_key, Key::Named(NamedKey::Space)) {
                    if let (Some((cursor_x, cursor_y)), Some(window)) = (self.cursor_position, self.window) {
                        let pixel_x = cursor_x.floor() as u32;
                        let pixel_y = cursor_y.floor() as u32;

                        if pixel_x < WIDTH && pixel_y < HEIGHT {
                            let pixel_number = pixel_y * WIDTH + pixel_x;

                            println!("Hovered pixel: {} (x={}, y={})", pixel_number, pixel_x, pixel_y);
                            window.set_title(&format!("simulation - pixel {}", pixel_number));
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.render().unwrap();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let scene_center = Point3::new(0., 0., -1., CartesianWorld);

    let world = World {
        manifold: Box::new(Schwarzschild4Manifold::new(scene_center, 0.25)), // Box::new(EuclideanMetric {}),
        textures: {
            let accretion = Texture::from_file("assets/accretion.jpg").expect("failed to load texture");
            let background = Texture::from_file("assets/space_sky.webp").expect("failed to load texture");
            Textures::new(accretion, background)
        },
        objects: vec![
            // Box::new(Sphere {
            //     center: Point3::new_cartesian(0., 0., -1.),
            //     radius: 0.27,
            //     // material: Box::new(Diffuse {
            //     //     color: Color { x: 128., y: 0., z: 0. },
            //     //     emission: Vec3 { x: 0., y: 0., z: 0. },
            //     // }),
            //     material: Box::new( {}),
            // }),
            Box::new(Sphere {
                center: Point3::new(0.6, 0., -0.5, CartesianWorld),
                radius: 0.08,
                material: Box::new(Metal {
                    color: Color::new(128., 128., 128.),
                    emission: Color::new(0., 0., 0.),
                    fuzz: 0.0,
                }),
                texture: TextureId::NONE,
            }),
            Box::new(Disc {
                center: Point3::new(0.0, 0.0, -1.0, CartesianWorld),
                normal: ThreeVector::new(0.2, 0.8, 0.3, geometry::vector::TangentSpace::CartesianWorld),
                // normal: ThreeVector::new(0.0, 0.0, 1., geometry::vector::TangentSpace::CartesianWorld),
                radius: 0.6,
                material: Box::new(Diffuse {
                    // color: Color::new(128., 128., 128.),
                    // emission: Color::new(237.0, 193.0, 154.0),
                    color: Color::new(255., 255., 255.),
                    emission: Color::new(255., 255., 255.),
                }),
                texture: TextureId::ACCRETION,
            })
        ],
    };

    let mut buffer = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

    let camera: Camera = Camera::new(WIDTH, HEIGHT);

    let start = Instant::now();
    let mut rng = StdRng::seed_from_u64(RNG_SEED);

    camera.render(buffer.as_mut_slice(), &world, &mut rng);

    let elapsed = start.elapsed();
    println!("Elapsed time: {:?}", elapsed);

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(buffer);

    event_loop.run_app(&mut app).unwrap();
}
