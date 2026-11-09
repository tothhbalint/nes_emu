use std::sync::Arc;

use emulator_core::nes::Nes;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

pub const NES_PALETTE: [[u8; 3]; 64] = [
    // $00-$0F
    [0x57, 0x57, 0x57],
    [0x00, 0x0C, 0x8E],
    [0x08, 0x00, 0xA6],
    [0x34, 0x00, 0x96],
    [0x55, 0x00, 0x61],
    [0x63, 0x00, 0x15],
    [0x5A, 0x00, 0x00],
    [0x3C, 0x0E, 0x00],
    [0x11, 0x28, 0x00],
    [0x00, 0x3B, 0x00],
    [0x00, 0x42, 0x00],
    [0x00, 0x3A, 0x05],
    [0x00, 0x26, 0x52],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    // $10-$1F
    [0xA5, 0xA5, 0xA5],
    [0x00, 0x41, 0xD9],
    [0x2F, 0x1E, 0xFF],
    [0x67, 0x04, 0xF2],
    [0x94, 0x00, 0xB4],
    [0xAA, 0x00, 0x57],
    [0xA3, 0x18, 0x00],
    [0x80, 0x39, 0x00],
    [0x4B, 0x5B, 0x00],
    [0x13, 0x76, 0x00],
    [0x00, 0x81, 0x00],
    [0x00, 0x79, 0x23],
    [0x00, 0x62, 0x88],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    // $20-$2F
    [0xFF, 0xFF, 0xFF],
    [0x4A, 0x9F, 0xFF],
    [0x79, 0x7E, 0xFF],
    [0xAF, 0x63, 0xFF],
    [0xDD, 0x55, 0xFF],
    [0xF7, 0x57, 0xC2],
    [0xF7, 0x6A, 0x63],
    [0xDC, 0x88, 0x10],
    [0xAE, 0xA9, 0x00],
    [0x78, 0xC4, 0x00],
    [0x4A, 0xD2, 0x11],
    [0x2F, 0xCF, 0x64],
    [0x2F, 0xBD, 0xC4],
    [0x41, 0x41, 0x41],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    // $30-$3F
    [0xFF, 0xFF, 0xFF],
    [0xB9, 0xDD, 0xFF],
    [0xCA, 0xD1, 0xFF],
    [0xDE, 0xC6, 0xFF],
    [0xF0, 0xC0, 0xFF],
    [0xFC, 0xC0, 0xEE],
    [0xFD, 0xC6, 0xCA],
    [0xF5, 0xD0, 0xAA],
    [0xE4, 0xDD, 0x95],
    [0xD0, 0xE8, 0x92],
    [0xBD, 0xEE, 0xA2],
    [0xB2, 0xEE, 0xC0],
    [0xB0, 0xE8, 0xE3],
    [0xB3, 0xB3, 0xB3],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;
const SCALE: u32 = 3;

struct App {
    nes: Nes,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("NES")
            .with_inner_size(LogicalSize::new(WIDTH * SCALE, HEIGHT * SCALE))
            .with_min_inner_size(LogicalSize::new(WIDTH, HEIGHT));

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(WIDTH, HEIGHT, surface).expect("failed to create pixels");

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    let _ = pixels.resize_surface(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                self.nes.run_frame();

                let Some(pixels) = self.pixels.as_mut() else {
                    return;
                };
                write_rgba(&self.nes, pixels.frame_mut());

                if pixels.render().is_err() {
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// `FrameBuffer` is one byte per pixel; expand to RGBA8 in place.
fn write_rgba(nes: &Nes, out: &mut [u8]) {
    let fb = nes.frame();

    let mut i = 0;
    for row in fb.iter() {
        for &index in row.iter() {
            let [r, g, b] = debug_color(index);
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
            out[i + 3] = 0xFF;
            i += 4;
        }
    }
}

/// Placeholder until the real 64-entry 2C02 palette goes in.
fn debug_color(index: u8) -> [u8; 3] {
    NES_PALETTE[(index & 0x3F) as usize]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: emulator <rom.nes>")?;

    let mut nes = Nes::new();
    nes.load_cart(&path)?;

    let event_loop = EventLoop::new()?;
    let mut app = App {
        nes,
        window: None,
        pixels: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
