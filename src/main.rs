mod graphics;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window
};
use pixels::{Pixels, SurfaceTexture};

use crate::graphics::vector::{Point3, Vec3};


const ASPECT_RATIO: f32 = 16.0 / 9.0;

const HEIGHT: u32 = 400;
const WIDTH: u32 = ((HEIGHT as f32) * ASPECT_RATIO) as u32;

const focal_length: f32 = 1.0;
const viewport_height: f32 = 2.0;
const viewport_width: f32 = viewport_height * ASPECT_RATIO;

#[derive(Default)]
struct App {
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let window = event_loop
            .create_window(Window::default_attributes().with_title("simulation"))
            .unwrap();

        let size = window.inner_size();
        let window_ref: &'static Window = Box::leak(Box::new(window));
        let surface_texture = SurfaceTexture::new(size.width, size.height, window_ref);

        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

        self.window = Some(window_ref);
        self.pixels = Some(pixels);

        let scale_factor = window_ref.scale_factor();
        println!("Scale factor : {}", scale_factor);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();

                    for spot in frame.chunks_exact_mut(4) {
                        spot[0] = 0x20; // R
                        spot[1] = 0x40; // G
                        spot[2] = 0xFF; // B
                        spot[3] = 0xFF; // A
                    }

                    pixels.render().unwrap();
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.window.expect("Bug - Window should exist").request_redraw();
    }
}

fn main() {
    let mut camera_center = Point3{x: 0., y: 0., z: 0.};

    let mut viewport_u_vect = Vec3{x: viewport_width, y: 0., z: 0.};
    let mut viewport_v_vect = Vec3{x: 0., y: -viewport_height, z: 0.};

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u = viewport_u_vect / (WIDTH as f32);
    let pixel_delta_v = viewport_v_vect / (HEIGHT as f32);

    // Calculate the location of the upper left pixel.
    let viewport_upper_left = 
        camera_center - Vec3{x: 0., y: 0., z: focal_length} - viewport_u_vect/2. - viewport_v_vect/2.;
    
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
