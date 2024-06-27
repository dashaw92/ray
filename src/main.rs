mod camera;
mod hit;
mod vec3;
mod ray;
mod sphere;
mod material;

use std::{fs::File, io::{stderr, Result as IOResult, Write}, sync::Arc};

use camera::Camera;
use hit::{Hit, World};
use material::{Dielectric, Lambertian, Metal};
use rand::Rng;
use ray::Ray;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sphere::Sphere;
use vec3::{Color, Point3, Vec3};

pub const ASPECT_RATIO: f64 = 3.0 / 2.0;
pub const IMG_WIDTH: u64 = 1200;
pub const IMG_HEIGHT: u64 = ((IMG_WIDTH as f64) / ASPECT_RATIO) as u64;
pub const SAMPLES_PER_PIXEL: u64 = 500;
pub const MAX_RAY_BOUNCES: u64 = 50;

pub const CAM_FROM: Point3 = Point3::new(13.0, 2.0, 3.0);
pub const CAM_LOOK_AT: Point3 = Point3::new(0.0, 0.0, 0.0);
pub const CAM_VUP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAM_VFOV_DEG: f64 = 20.0;
pub const CAM_DIST_TO_FOCUS: f64 = 10.0;
pub const CAM_APERTURE: f64 = 0.1;

fn main() {
    let ppm = create_ppm();
    let world = create_world();

    let mut f = File::create("render.ppm").unwrap();
    write_ppm(&mut f, &ppm, &world).unwrap();
}

fn create_ppm() -> Ppm {
    Ppm {
        img_w: IMG_WIDTH,
        img_h: IMG_HEIGHT,
        samples: SAMPLES_PER_PIXEL,
        cam: Camera::new(
            CAM_FROM,
            CAM_LOOK_AT,
            CAM_VUP,
            CAM_VFOV_DEG, 
            ASPECT_RATIO,
            CAM_APERTURE,
            CAM_DIST_TO_FOCUS,
        )
    }
}

fn create_world() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);
    
    world.push(Box::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new((a as f64) + rng.gen_range(0.0..0.9),
                                     0.2,
                                     (b as f64) + rng.gen_range(0.0..0.9));

            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));
    world
}

fn write_ppm(w: &mut impl Write, img: &Ppm, world: &World) -> IOResult<()> {
    writeln!(w, "P3")?;
    writeln!(w, "{} {}", img.img_w, img.img_h)?;
    writeln!(w, "255")?;

    for j in (0..img.img_h).rev() {
        eprint!("\r[%{:3.0}] Remaining scanlines: {:3}", ((img.img_h - j) as f64 / img.img_h as f64) * 100.0, j);
        stderr().flush()?;

        let scanline: Vec<Color> = (0..img.img_w).into_par_iter().map(|i| {
            let mut rng = rand::thread_rng();
            let mut color = Color::zero();
            for _ in 0..img.samples {
                let ru: f64 = rng.gen();
                let rv: f64 = rng.gen();

                let u = ((i as f64) + ru) / ((img.img_w - 1) as f64);
                let v = ((j as f64) + rv) / ((img.img_h - 1) as f64);

                let r = img.cam.get_ray(u, v);
                color += ray_color(&r, world);
            }

            color
        }).collect();

        for color in scanline {
            writeln!(w, "{}", color.format_color(img.samples))?;
        }
    }

    println!();
    Ok(())
}

struct Ppm {
    img_w: u64,
    img_h: u64,
    samples: u64,
    cam: Camera,
}

fn ray_color(r: &Ray, world: &World) -> Color {
    fn ray_color_aux(r: &Ray, world: &World, depth: u64) -> Color {
        if depth == 0 {
            return Color::zero() //black
        }

        if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
            if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
                attenuation * ray_color_aux(&scattered, world, depth - 1)
            } else {
                Color::zero()
            }
        } else {
            let unit = r.dir.normalized();
            let t = 0.5 * (unit.y() + 1.0);
            (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
        }
    }

    ray_color_aux(r, world, MAX_RAY_BOUNCES)
}