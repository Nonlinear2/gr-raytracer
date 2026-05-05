use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use pixels::{Pixels, SurfaceTexture};

#[derive(Default)]
struct App<'a> {
    window: Option<Window>,
    pixels: Option<Pixels<'a>>,
}


impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let size = window.inner_size();

        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);

        let pixels = Pixels::new(
            size.width,
            size.height,
            surface_texture,
        ).unwrap();

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                let pixels = self.pixels.as_mut().unwrap();

                let frame = pixels.frame_mut();

                // Fill screen (RGBA)
                for chunk in frame.chunks_exact_mut(4) {
                    chunk[0] = 0x20; // R
                    chunk[1] = 0x40; // G
                    chunk[2] = 0x80; // B
                    chunk[3] = 0xFF; // A
                }

                // Example: draw a white pixel at (100, 100)
                let width = pixels.texture().width();
                let x = 100;
                let y = 100;
                let i = ((y * width + x) * 4) as usize;

                frame[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);

                pixels.render().unwrap();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
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
    event_loop.run_app(&mut app);
}