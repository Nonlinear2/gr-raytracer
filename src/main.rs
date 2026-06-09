mod graphics;
mod geometry;
mod integration;
mod config;

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

#[derive(Default)]
struct App {
    image_width: u32,
    image_height: u32,
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    frame: Vec<u8>,
    cursor_position: Option<(f64, f64)>,
}

impl App {
    fn new(frame: Vec<u8>, image_width: u32, image_height: u32) -> Self {
        Self {
            image_width,
            image_height,
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
                    .with_inner_size(PhysicalSize::new(self.image_width, self.image_height))
                    .with_resizable(false)
            )
            .unwrap();

        let size = window.inner_size();
        let window_ref: &'static Window = Box::leak(Box::new(window));
        let surface_texture = SurfaceTexture::new(size.width, size.height, window_ref);

        let mut pixels = Pixels::new(self.image_width, self.image_height, surface_texture).unwrap();
        
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
                    if let (Some((cursor_x, cursor_y)), ..) = (self.cursor_position, self.window) {
                        let pixel_x = cursor_x.floor() as u32;
                        let pixel_y = cursor_y.floor() as u32;

                        if pixel_x < self.image_width && pixel_y < self.image_height {
                            let pixel_number = pixel_y * self.image_width + pixel_x;

                            println!("Hovered pixel: {} (x={}, y={})", pixel_number, pixel_x, pixel_y);
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

    let world = World {
        scene_size: 3.0,
        manifold: Box::new(Schwarzschild4Manifold::new(
            Point3::new(0., 0., -1., CartesianWorld),
            0.25
        )), // Box::new(EuclideanMetric {}),
        textures: {
            let accretion = Texture::from_file("assets/accretion.jpg").expect("failed to load texture");
            let background = Texture::from_file("assets/space_sky.webp").expect("failed to load texture");
            Textures::new(accretion, background)
        },
        objects: vec![
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

    let mut buffer = vec![0u8; (config::IMAGE_WIDTH * config::IMAGE_HEIGHT * 4) as usize];

    let mut camera: Camera = Camera::new(config::IMAGE_WIDTH, config::IMAGE_HEIGHT);
    camera.samples_per_pixel = config::SAMPLES_PER_PIXEL;

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
