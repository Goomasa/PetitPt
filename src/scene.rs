use crate::{
    math::Color,
    object::Object,
    ray::{HitRecord, Ray},
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
}
