use minifb::{Window, WindowOptions, Key, Scale};
use std::error::Error;
use rand::Rng;
use std::time::{Instant, Duration};
use std::thread::sleep;
use rayon::prelude::*;

fn time<F: FnOnce()>(f: F) -> Duration {
    let timer = Instant::now();
    f();
    timer.elapsed()
}

fn clamped<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

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

    fn iter_mut(& mut self) -> impl Iterator<Item=(usize, usize, &mut u32)> {
        let width = self.width;
        let height = self.height;
        self.buffer
            .iter_mut()
            .enumerate()
            .map(move |(i, v)| {
                let y = i / width;
                let x = i - y * width;
                (x, height - y - 1, v)
            })
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

    fn random_in_unit_sphere() -> Vec3 {
        let mut random = rand::thread_rng();

        loop {
            let x = random.gen_range(-1.0, 1.0);
            let y = random.gen_range(-1.0, 1.0);
            let z = random.gen_range(-1.0, 1.0);
            let p = Vec3::new(x, y, z);

            if p.squared_length() < 1.0 {
                return p
            }
        }
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

    fn lerp(from: Vec3, to: Vec3, v: f32) -> Vec3 {
        let v = v.min(1.0).max(0.0);
        from * (1.0 - v) + to * v
    }
}

impl std::ops::Add<Self> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

impl std::ops::Sub<Self> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        self.subtract(rhs)
    }
}

impl std::ops::Mul<Self> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        self.multiply_scalar(rhs)
    }
}

impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs.multiply_scalar(self)
    }
}

impl std::ops::Div<Self> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        self.div_scalar(rhs)
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }

    fn origin(&self) -> Vec3 {
        self.origin
    }

    fn direction(&self) -> Vec3 {
        self.direction
    }

    fn point_at_parameter(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

struct MaterialScatter {
    attenuation: Vec3,
    scattered_ray: Ray
}

#[derive(Clone, Copy)]
enum Material {
    Diffuse { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 }
}

impl Material {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<MaterialScatter> {
        match self {
            Material::Diffuse { albedo } => {
                let target = hit.position + hit.normal + Vec3::random_in_unit_sphere();
                Some(MaterialScatter {
                    attenuation: *albedo,
                    scattered_ray: Ray::new(hit.position, target - hit.position)
                })
            }
            Material::Metal { albedo, fuzz } => {
                fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
                    v - 2.0 * v.dot(normal) * normal
                }

                let reflected = reflect(ray.direction.unit_vector(), hit.normal);
                let scattered_ray = Ray::new(hit.position, reflected + clamped(*fuzz, 0.0, 1.0) * Vec3::random_in_unit_sphere());
                if scattered_ray.direction.dot(hit.normal) > 0.0 {
                    Some(MaterialScatter{
                        attenuation: *albedo,
                        scattered_ray
                    })
                } else {
                    None
                }
            }
        }
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
    material: Material
}

impl Sphere {
    fn new(center: Vec3, radius: f32, material: Material) -> Sphere {
        Sphere { center, radius, material }
    }

    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let oc = ray.origin() - self.center;
        let a = ray.direction().dot(ray.direction());
        let b = oc.dot(ray.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;

        if discriminant > 0.0 {
            let temp = (-b - (b * b - a * c).sqrt()) / a;
            if temp < t_max && temp > t_min {
                let point = ray.point_at_parameter(temp);
                return Some(Hit::new(temp, point, (point - self.center) / self.radius));
            }
            let temp = (-b + (b * b - a * c).sqrt()) / a;
            if temp < t_max && temp > t_min {
                let point = ray.point_at_parameter(temp);
                return Some(Hit::new(temp, point, (point - self.center) / self.radius));
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

    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(Hit, Material)> {
        let mut closest_t = t_max;
        let mut result = None;

        self.spheres.iter().for_each(|s| {
            if let Some(hit) = s.hit_test(ray, t_min, closest_t) {
                closest_t = hit.t;
                result = Some((hit, s.material));
            }
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
    fn new(origin: Vec3, look_at: Vec3, up: Vec3, vertical_fov: f32, aspect_ratio: f32) -> Camera {
        let half_height = (vertical_fov.to_radians() / 2.0).tan();
        let half_width = aspect_ratio * half_height;

        // create orthonormal basis
        let w = (origin - look_at).unit_vector();
        let u = up.cross(w).unit_vector();
        let v = w.cross(u);

        Camera {
            lower_left_corner: origin - half_width * u - half_height * v - w,
            horizontal: 2.0 * half_width * u,
            vertical: 2.0 * half_height * v,
            origin,
        }
    }

    fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(self.origin, self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin)
    }
}

fn color(ray: &Ray, world: &World, bounces: usize) -> Vec3 {
    if let Some((hit, material)) = world.hit_test(ray, 0.001, 1000.0) {
        if bounces == 0 {
            return Vec3::zero();
        }

        return if let Some(scatter) = material.scatter(ray, &hit) {
            scatter.attenuation * color(&scatter.scattered_ray, world, bounces - 1)
        } else {
            Vec3::zero()
        }
    }

    Vec3::lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), (ray.direction.unit_vector().y + 1.0) * 0.5)
}

fn render(bitmap: &mut Bitmap) {
    fn apply_gamma_2_correction(c: Vec3) -> Vec3 {
        Vec3::new(c.x.sqrt(), c.y.sqrt(), c.z.sqrt())
    }

    let spheres = vec![
        Sphere::new(Vec3::new(0.0, -100.0, 0.0), 100.0, Material::Diffuse { albedo: Vec3::new(0.8, 0.8, 0.0) }),
        Sphere::new(Vec3::new(-1.0, 0.3, 0.0), 0.3, Material::Metal { albedo: Vec3::new(0.6, 0.6, 0.6), fuzz: 0.4 }),
        Sphere::new(Vec3::new(0.0, 0.5, 0.0), 0.5, Material::Diffuse { albedo: Vec3::new(0.9, 0.2, 0.2) }),
        Sphere::new(Vec3::new(1.0, 0.5, 0.0), 0.5, Material::Metal { albedo: Vec3::new(0.4, 0.4, 0.8), fuzz: 0.0 }),
    ];

    let world = World::new(spheres);
    let width = bitmap.width();
    let height = bitmap.height();
    let camera = Camera::new(
        Vec3::new(0.0, 2.0, 2.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        width as f32 / height as f32);
    let aa_samples = 100;

    bitmap
        .iter_mut()
        .par_bridge()
        .for_each(|(x, y, p)| {
            let mut random = rand::thread_rng();
            let mut c = Vec3::zero();

            for _ in 0..aa_samples {
                let x_scaled = ((x as f32) + random.gen_range(0.0, 1.0)) / (width as f32);
                let y_scaled = ((y as f32) + random.gen_range(0.0, 1.0)) / (height as f32);
                c = c + color(&camera.ray(x_scaled, y_scaled), &world, 50);
            }

            c = c / aa_samples as f32;
            c = apply_gamma_2_correction(c);
            let r = (c.x * std::u8::MAX as f32) as u32;
            let g = (c.y * std::u8::MAX as f32) as u32;
            let b = (c.z * std::u8::MAX as f32) as u32;
            *p = (*p & 0xff000000) | r << 16 | g << 8 | b;
        });
}

fn main() -> Result<(), Box<Error>> {
    let width = 400;
    let height = 300;

    let mut bitmap = Bitmap::new(width, height);
    eprintln!("Rendering...");
    let rendertime = time(|| { render(&mut bitmap) });
    eprintln!("Render completed ({} ms)", rendertime.as_millis());

    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window = Window::new("Raytracer", width, height, options)?;
    window.update_with_buffer(bitmap.buffer())?;

    let event_poll_frequency = 30.0;
    let millis_per_frame = (1.0 / event_poll_frequency * 1000.0) as i64;
    let mut event_poll_start = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        window.update_with_buffer(bitmap.buffer())?;

        let remaining_ms = millis_per_frame - event_poll_start.elapsed().as_millis() as i64;
        if remaining_ms > 0 {
            sleep(Duration::from_millis(remaining_ms as u64));
        }

        event_poll_start = Instant::now();
    }

    Ok(())
}
