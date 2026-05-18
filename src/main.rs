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

use crate::graphics::camera::Camera;
use crate::graphics::point::Point3;
use crate::graphics::world::World;
use crate::relativity::metric::Schwarzschild;

use std::time::Instant;

const HEIGHT: u32 = 80;
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
        manifold: Box::new(Schwarzschild::new(Point3::new_cartesian(0., 0., -1.), 0.25)), // Box::new(EuclideanMetric {}),
        objects: vec![
            // Box::new(Sphere {
            //     center: Point3::new_cartesian(0., 0., -1.),
            //     radius: 0.27,
            //     // material: Box::new(Diffuse {
            //     //     albedo: Color { x: 128., y: 0., z: 0. },
            //     //     emission: Vec3 { x: 0., y: 0., z: 0. },
            //     // }),
            //     material: Box::new( {}),
            // }),
            // Box::new(Sphere {
            //     center: Point3::new_cartesian(0.9, 0., -0.6),
            //     radius: 0.4,
            //     material: Box::new(Metal {
            //         albedo: Color::new(128., 128., 128.),
            //         emission: Color::new(0., 0., 0.),
            //         fuzz: 0.15,
            //     }),
            // })
        ],
    };

    let mut buffer = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    
    let camera: Camera = Camera::new(WIDTH, HEIGHT);

    // camera.debug_ray_trajectory(&world);
    // return;

    let start = Instant::now();

    camera.render(buffer.as_mut_slice(), &world);

    let elapsed = start.elapsed();
    println!("Elapsed time: {:?}", elapsed);

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(buffer);

    event_loop.run_app(&mut app).unwrap();
}
