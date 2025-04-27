use crate::{math::Color, object::Object};

pub struct Scene<'a> {
    objects: Vec<&'a Object>,
    background: Color,
    lights: Vec<&'a Object>,
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
}
