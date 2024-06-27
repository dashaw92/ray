use std::sync::Arc;

use crate::{hit::{Hit, HitRecord}, material::Scatter, ray::Ray, vec3::{Point3, Vec3}};

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Scatter>
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Scatter>) -> Sphere {
        Self { center, radius, mat }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.orig - self.center;
        let a = r.dir.length().powi(2);
        let half_b = oc.dot(&r.dir);
        let c = oc.length().powi(2) - self.radius.powi(2);
        let discrim = half_b.powf(2.0) - a * c;

        if discrim < 0.0 {
            return None
        }

        let sqrtd = discrim.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None
            }
        }

        let p = r.at(root);
        let mut rec = HitRecord {
            t: root,
            p,
            mat: self.mat.clone(),
            normal: Vec3::zero(),
            front_face: false,
        };

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        Some(rec)
    }
}