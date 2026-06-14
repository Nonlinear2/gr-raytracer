use winit::{
    dpi::PhysicalSize,
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    keyboard::{Key, NamedKey},
    window::Window
};
use pixels::{Pixels, SurfaceTexture};

#[derive(Default)]
pub struct App {
    image_width: u32,
    image_height: u32,
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    frame: Vec<u8>,
    cursor_position: Option<(f64, f64)>,
}

impl App {
    pub fn new(frame: Vec<u8>, image_width: u32, image_height: u32) -> Self {
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
