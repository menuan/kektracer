use minifb::{Window, WindowOptions, Key, CursorStyle, MouseMode, MouseButton, Scale};
use std::error::Error;
use rand::Rng;

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
            Some(&mut self.buffer[(self.height - y - 1) * self.width + (x)])
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

    fn zero() -> Vec3 {
        Vec3::new(0.00, 0.0, 0.0)
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
            y: self.z * other.x - self.x * other.z,
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

struct Hit {
    t: f32,
    position: Vec3,
    normal: Vec3,
}

impl Hit {
    fn new(t: f32, position: Vec3, normal: Vec3) -> Hit {
        Hit { t, position, normal }
    }
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    fn new(center: Vec3, radius: f32) -> Sphere {
        Sphere { center, radius }
    }

    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let oc = ray.origin().subtract(self.center);
        let a = ray.direction().dot(ray.direction());
        let b = oc.dot(ray.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;

        if discriminant > 0.0 {
            let temp = (-b - (b * b - a * c).sqrt()) / a;
            if temp < t_max && temp > t_min {
                let point = ray.point_at_parameter(temp);
                return Some(Hit::new(temp,
                                     point,
                                     point.subtract(self.center).div_scalar(self.radius)));;
            }
            let temp = (-b + (b * b - a * c).sqrt()) / a;
            if temp < t_max && temp > t_min {
                let point = ray.point_at_parameter(temp);
                return Some(Hit::new(temp,
                                     point,
                                     point.subtract(self.center).div_scalar(self.radius)));
            }
        }

        None
    }
}

struct World {
    spheres: Vec<Sphere>,
}

impl World {
    fn new(spheres: Vec<Sphere>) -> World {
        World { spheres }
    }

    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let mut closest_t = t_max;
        let mut result = None;

        self.spheres.iter().for_each(|s| {
            s.hit_test(ray, t_min, closest_t)
                .map(|h| {
                    closest_t = h.t;
                    result = Some(h);
                });
        });

        result
    }
}

struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    fn new() -> Camera {
        Camera {
            lower_left_corner :  Vec3::new(-2.0, -1.0, -1.0),
            horizontal :  Vec3::new(4.0, 0.0, 0.0),
            vertical :  Vec3::new(0.0, 2.0, 0.0),
            origin :  Vec3::new(0.0, 0.0, 0.0)
        }
    }

    fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(self.origin, self.lower_left_corner.add(self.horizontal.multiply_scalar(u).add(self.vertical.multiply_scalar(v))))
    }
}


fn color(ray: &Ray, world: &World) -> Vec3 {
    if let Some(hit) = world.hit_test(ray, 0.0, 100000.0) {
        let normal = hit.normal;
        return Vec3::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0).multiply_scalar(0.5);
    }

    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y + 1.0);
    Vec3::new(1.0, 1.0, 1.0)
        .multiply_scalar(1.0 - t)
        .add(Vec3::new(0.5, 0.7, 1.0).multiply_scalar(t))
}

fn render(bitmap: &mut Bitmap) {
    let world = World::new(vec![
        Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5),
        Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)
    ]);

    let camera = Camera::new();
    let width = bitmap.width();
    let height = bitmap.height();
    let aa_samples = 10;
    let mut random = rand::thread_rng();

    for y in 0..height {
        for x in 0..width {
            if let Some(p) = bitmap.get_mut(x, y) {
                let mut c = Vec3::zero();

                for _ in 0..aa_samples {
                    let x_scaled = ((x as f32) + random.gen_range(0.0, 1.0)) / (width as f32);
                    let y_scaled = ((y as f32) + random.gen_range(0.0, 1.0)) / (height as f32);
                    c = c.add(color(&camera.ray(x_scaled, y_scaled), &world));
                }

                c = c.div_scalar(aa_samples as f32);
                let r = (c.x * std::u8::MAX as f32) as u32;
                let g = (c.y * std::u8::MAX as f32) as u32;
                let b = (c.z * std::u8::MAX as f32) as u32;
                *p = (*p & 0xff0000ff) | r << 16 | g << 8 | b;
            }
        }
    }
}

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;
    let mut options = WindowOptions::default();
    options.scale = Scale::X4;
    let mut window = Window::new("Raytracer", width, height, options)?;
    window.set_cursor_style(CursorStyle::ClosedHand);

    let mut bitmap = Bitmap::new(width, height);
    render(&mut bitmap);

    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
                bitmap
                    .get_mut(x as usize, height - (y as usize) - 1)
                    .map(|p| *p = 0xffffffff);
            }
        }
        window.update_with_buffer(bitmap.buffer())?;
    }

    Ok(())
}
