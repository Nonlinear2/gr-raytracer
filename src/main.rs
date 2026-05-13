mod graphics;

use std::collections::btree_set::Intersection;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window
};
use pixels::{Pixels, SurfaceTexture};

use crate::graphics::{ray::RayIntersection, surface::Surface, vector::{Point3, Vec3}};
use crate::graphics::ray::Ray;
use crate::graphics::color::Color;
use crate::graphics::surface::Sphere;


const ASPECT_RATIO: f32 = 16.0 / 9.0;

const HEIGHT: u32 = 400;
const WIDTH: u32 = ((HEIGHT as f32) * ASPECT_RATIO) as u32;

const focal_length: f32 = 1.0;
const viewport_height: f32 = 2.0;
const viewport_width: f32 = viewport_height * ASPECT_RATIO;

fn ray_color(ray: &Ray) -> Color {
    let sphere = Sphere {center: Vec3 { x: 0., y: 0., z: -1.}, radius: 0.5};
    return match sphere.hit(ray, 0., 20.) {
        Some(i) => Color {
            r: (255. * i.normal.dot(&ray.direction).abs()) as u8,
            g: 0,
            b: 0,
            a: 255
        },
        None => Color { r: ((ray.direction.normalize().y + 1.)*127.) as u8, g: 255, b: 255, a: 255 }
    }
}

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
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();

                    let camera_center = Point3{x: 0., y: 0., z: 0.};

                    let viewport_u_vect = Vec3{x: viewport_width, y: 0., z: 0.};
                    let viewport_v_vect = Vec3{x: 0., y: -viewport_height, z: 0.};

                    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
                    let pixel_delta_u = viewport_u_vect / (WIDTH as f32);
                    let pixel_delta_v = viewport_v_vect / (HEIGHT as f32);

                    // Calculate the location of the upper left pixel.
                    let viewport_upper_left = 
                        camera_center - Vec3{x: 0., y: 0., z: focal_length} - viewport_u_vect/2. - viewport_v_vect/2.;
                    
                    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

                    for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let i = idx % WIDTH as usize;
                        let j = idx / WIDTH as usize;

                        let pixel_center = pixel00_loc + (pixel_delta_u * (i as f32)) + (pixel_delta_v * (j as f32));
                        let ray_direction = pixel_center - camera_center;
                        let ray = Ray{origin: camera_center, direction: ray_direction};

                        let col = ray_color(&ray);

                        pixel[0] = col.r; // R
                        pixel[1] = col.g; // G
                        pixel[2] = col.b; // B
                        pixel[3] = col.a; // A
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
