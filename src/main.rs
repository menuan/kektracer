use minifb::{Window, WindowOptions, Key, CursorStyle, MouseMode, MouseButton};
use std::error::Error;

struct Bitmap {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

impl Bitmap {
    fn new(width: usize, height: usize) -> Bitmap {
        Bitmap { width, height, buffer: vec![0; width * height] }
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut u32> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&mut self.buffer[y * self.width + x])
        }
    }

    fn buffer(&self) -> &[u32] {
        &self.buffer
    }
}

fn main() -> Result<(), Box<Error>> {
    let width = 800;
    let height = 600;
    let mut window = Window::new("Game of life",width, height, WindowOptions::default())?;
    window.set_cursor_style(CursorStyle::ClosedHand);

    let mut bitmap = Bitmap::new(width, height);

    for y in 0..height {
        let y_scaled = (y as f32) / (height as f32);
        for x in 0..width {
            let x_scaled = (x as f32) / (width as f32);
            bitmap.get_mut(x, y).map(|p| {
                let r = (x_scaled * std::u8::MAX as f32) as u32;
                let g = (y_scaled * std::u8::MAX as f32) as u32;
                let b = 24;
                *p = (*p & 0xff0000ff) | r << 16 | g << 8 | b;
            });
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                bitmap.get_mut(x as usize, y as usize).map(|p| *p = 0xffffffff);
            }
        }
        window.update_with_buffer(bitmap.buffer())?;
    }

    Ok(())
}

