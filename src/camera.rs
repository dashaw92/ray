use crate::{ray::Ray, vec3::{Point3, Vec3}};

pub struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horiz: Vec3,
    pub vert: Vec3,
    pub cu: Vec3,
    pub cv: Vec3,
    pub lens_radius: f64,
}

impl Camera {
    pub fn new(
        lfrom: Point3, 
        lat: Point3, 
        vup: Vec3, 
        vfov: f64, 
        aspect_ratio: f64, 
        aperture: f64, 
        focus_dist: f64
    ) -> Camera {
        let theta = std::f64::consts::PI / 180.0 * vfov;
        let viewport_h = 2.0 * (theta / 2.0).tan();
        let viewport_w = aspect_ratio * viewport_h;

        let cw = (lfrom - lat).normalized();
        let cu = vup.cross(cw).normalized();
        let cv = cw.cross(cu);

        let h = focus_dist * viewport_w * cu;
        let v = focus_dist * viewport_h * cv;

        let lower_left_corner = lfrom - h / 2.0 - v / 2.0 - focus_dist * cw;

        Camera {
            origin: lfrom,
            horiz: h,
            vert: v,
            lower_left_corner,
            cu,
            cv,
            lens_radius: aperture / 2.0
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.cu * rd.x() + self.cv * rd.y();

        Ray::new(self.origin + offset, 
            self.lower_left_corner + s * self.horiz + t * self.vert - self.origin - offset)
    }
}