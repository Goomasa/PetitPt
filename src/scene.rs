use crate::{
    math::{Color, Point3},
    object::{sample_sphere, sample_sphere_pdf, Object},
    random::XorRand,
    ray::{HitRecord, NeeResult, Ray},
};

pub struct Scene<'a> {
    pub objects: Vec<&'a Object>,
    pub background: Color,
    pub lights: Vec<&'a Object>,
}

impl<'a> Scene<'a> {
    pub fn new(objs: Vec<&'a Object>, back: Color) -> Self {
        let lights = objs
            .clone()
            .into_iter()
            .filter(|obj| obj.get_bxdf().is_light())
            .collect();

        Scene {
            objects: objs,
            background: back,
            lights: lights,
        }
    }

    pub fn intersect(&self, ray: &Ray, record: &mut HitRecord) -> bool {
        for obj in self.objects.iter() {
            let _ = obj.hit(ray, record);
        }
        record.obj_id != -1
    }

    pub fn nee(&self, org: Point3, rand: &mut XorRand) -> NeeResult {
        let mut result = NeeResult::new();
        let mut record = HitRecord::new();
        for obj in self.lights.iter() {
            let (pdf, dir) = match obj {
                Object::Sphere { center, radius, .. } => sample_sphere(org, center, *radius, rand),
                Object::Plane { .. } => continue,
            };

            let _ = self.intersect(&Ray { org, dir }, &mut record);
            if !record.bxdf.is_light() {
                continue;
            }

            result.dir = dir;
            result.color = record.color;
            result.pdf = pdf;
        }
        result
    }

    pub fn sample_obj_pdf(&self, id: i32, org: Point3) -> f64 {
        let obj = self.objects[id as usize];
        match obj {
            Object::Sphere { center, radius, .. } => sample_sphere_pdf(org, center, *radius),
            Object::Plane { .. } => 0.,
        }
    }
}
