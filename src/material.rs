use rand::Rng;

use crate::{hit::HitRecord, ray::Ray, vec3::{Color, Vec3}};

pub trait Scatter: Send + Sync {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(a: Color) -> Lambertian {
        Lambertian { albedo: a }
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, _r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_dir = rec.normal + Vec3::random_in_unit_sphere().normalized();
        if scatter_dir.near_zero() {
            scatter_dir = rec.normal;
        }
        let scattered = Ray::new(rec.p, scatter_dir);

        Some((self.albedo, scattered))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(a: Color, fuzz: f64) -> Metal {
        Metal { albedo: a, fuzz }
    }
}

impl Scatter for Metal {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r.dir.reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.p, reflected + self.fuzz * Vec3::random_in_unit_sphere());

        if scattered.dir.dot(&rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Self { ir: index_of_refraction }
    }

    pub fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(2)
    }
}

impl Scatter for Dielectric {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit = r.dir.normalized();
        let cos_theta = ((-1.0) * unit).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_refract = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_refract {
            unit.reflect(rec.normal)
        } else {
            unit.refract(rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);

        Some((Color::new(1.0, 1.0, 1.0), scattered))
    }
}