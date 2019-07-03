use minifb::{Window, WindowOptions, Key, CursorStyle, MouseMode, MouseButton, Scale};
use std::error::Error;

struct Bitmap {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

impl Bitmap {
    fn new(width: usize, height: usize) -> Bitmap {
        Bitmap {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
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

#[derive(Copy, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn x(self) -> f32 {
        self.x
    }

    fn y(self) -> f32 {
        self.y
    }

    fn z(self) -> f32 {
        self.z
    }

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn subtract(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn negate(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    fn multiply(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    fn multiply_scalar(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    fn div(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }

    fn div_scalar(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }

    fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: -(self.x * other.z - self.z * other.x),
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn squared_length(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn unit_vector(self) -> Vec3 {
        self.div_scalar(self.length())
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }

    fn origin(self) -> Vec3 {
        self.origin
    }

    fn direction(self) -> Vec3 {
        self.direction
    }

    fn point_at_parameter(self, t: f32) -> Vec3 {
        self.origin.add(self.direction.multiply_scalar(t))
    }
}

fn render(bitmap: &mut Bitmap) {
    let width = bitmap.width();
    let height = bitmap.height();
    for y in 0..height {
        let y_scaled = (y as f32) / (height as f32);
        for x in 0..width {
            let x_scaled = (x as f32) / (width as f32);
            bitmap
                .get_mut(x, y)
                .map(|p| {
                         let r = (x_scaled * std::u8::MAX as f32) as u32;
                         let g = (y_scaled * std::u8::MAX as f32) as u32;
                         let b = 24;
                         *p = (*p & 0xff0000ff) | r << 16 | g << 8 | b;
                     });
        }
    }
}

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;
    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window = Window::new("Raytracer", width, height, options)?;
    window.set_cursor_style(CursorStyle::ClosedHand);

    let mut bitmap = Bitmap::new(width, height);
    render(&mut bitmap);

    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                bitmap
                    .get_mut(x as usize, y as usize)
                    .map(|p| *p = 0xffffffff);
            }
        }
        window.update_with_buffer(bitmap.buffer())?;
    }

    Ok(())
}
