use crate::{
    math::{cross, Point3, Vec3, PI},
    random::XorRand,
};

struct Camera {
    pixel_w: u32,
    pixel_h: u32,
    sensor_dir: Vec3,
    sensor_center: Point3,
    sensor_w: f64,
    sensor_h: f64,
    sensor_u: Vec3,
    sensor_v: Vec3,
    sensor_to_lens: f64,
    lens_radius: f64,
    lens_center: Point3,
    lens_to_plane: f64,
    iso: f64,
    spp: u32,  //samples per pixel
    sspp: u32, //super samples per pixel
}

impl Camera {
    pub fn new(
        aspect_ratio: f64, //height to width ratio
        px_w: u32,
        sensor_dir: Vec3,
        sensor_c: Point3,
        sensor_w: f64,
        lens_r: f64,
        focal_len: f64,
        lens_to_plane: f64,
        iso_scale: f64,
        spp: u32,
        sspp: u32,
    ) -> Self {
        let px_h = (px_w as f64 * aspect_ratio) as u32;
        let sensor_h = sensor_w * aspect_ratio;
        let w_per_px = sensor_w / px_w as f64;
        let h_per_px = sensor_h / px_h as f64;
        let iso = iso_scale / (w_per_px * h_per_px);

        let sensor_u = cross(sensor_dir, Vec3(0., 1., 0.)).normalize();
        let sensor_v = cross(sensor_dir, sensor_u).normalize();
        let lens_center = sensor_c + sensor_dir * focal_len;

        Camera {
            pixel_w: px_w,
            pixel_h: px_h,
            sensor_dir: sensor_dir,
            sensor_center: sensor_c,
            sensor_w: sensor_w,
            sensor_h: sensor_h,
            sensor_u: sensor_u,
            sensor_v: sensor_v,
            sensor_to_lens: focal_len,
            lens_radius: lens_r,
            lens_center: lens_center,
            lens_to_plane: lens_to_plane,
            iso: iso,
            spp: spp,
            sspp: sspp,
        }
    }

    pub fn sample_lens(&self, rand: &mut XorRand) -> Point3 {
        //return sample_pos
        let theta = 2.0 * PI * rand.next01();
        let r = rand.next01().sqrt() * self.lens_radius;

        self.lens_center + self.sensor_u * r * theta.cos() + self.sensor_v * r * theta.sin()
    }

    pub fn first_dir(&self, pixel_pos: Point3, lens_pos: Point3) -> Vec3 {
        let plane_pos = (lens_pos - pixel_pos) * (self.sensor_to_lens + self.lens_to_plane)
            / self.sensor_to_lens
            + pixel_pos;
        (plane_pos - lens_pos).normalize()
    }
}
