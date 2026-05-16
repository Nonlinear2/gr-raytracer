mod graphics;
mod relativity;
mod integration;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window
};
use pixels::{Pixels, SurfaceTexture};

use crate::graphics::{camera::Camera, surface::{Diffuse, Metal, Sphere}, vector::Color};
use crate::graphics::world::World;
use glam::Vec3;
use crate::relativity::metric::SchwartzschildMetric;

const HEIGHT: u32 = 200;
const WIDTH: u32 = ((HEIGHT as f32) * 16.0 / 9.0) as u32;

#[derive(Default)]
struct App {
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    frame: Vec<u8>,
}

impl App {
    fn new(frame: Vec<u8>) -> Self {
        Self {
            window: None,
            pixels: None,
            frame,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let window = event_loop
            .create_window(Window::default_attributes().with_title("simulation"))
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

    let world = World {
        metric: SchwartzschildMetric::new(),
        objects: vec![
            Box::new(Sphere {
                center: Vec3 { x: 0., y: 0., z: -1. },
                radius: 0.5,
                material: Box::new(Diffuse {
                    albedo: Color { x: 128., y: 0., z: 0. },
                    emission: Vec3 { x: 0., y: 0., z: 0. },
                }),
            }),
            Box::new(Sphere {
                center: Vec3 { x: 0.4, y: 0., z: -0.6 },
                radius: 0.1,
                material: Box::new(Metal {
                    albedo: Color { x: 128., y: 128., z: 128. },
                    emission: Vec3 { x: 0., y: 0., z: 0. },
                    fuzz: 0.15,
                }),
            })
        ],
    };

    let mut buffer = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    
    let camera: Camera = Camera::new(WIDTH, HEIGHT);

    camera.render(buffer.as_mut_slice(), &world);

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(buffer);

    event_loop.run_app(&mut app).unwrap();
}
