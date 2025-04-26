use crate::material::Bxdf;
use crate::math::{Color, Point3, Vec3, INF};

pub struct Ray {
    pub org: Point3,
    pub dir: Vec3,
}

pub struct HitRecord {
    pub pos: Point3,
    pub normal: Vec3,
    pub distance: f64,
    pub color: Color,
    pub bxdf: Bxdf,
    pub obj_id: i32,
}

impl HitRecord {
    pub fn new() -> Self {
        HitRecord {
            pos: Vec3::new(0.),
            normal: Vec3::new(0.),
            distance: INF,
            color: Vec3::new(0.),
            bxdf: Bxdf::Light,
            obj_id: -1,
        }
    }
}
