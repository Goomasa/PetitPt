use ply_rs as ply;
use ply_rs::ply::Property::{self, Float, ListInt};
//using crate "ply_rs", https://github.com/Fluci/ply-rs
use linked_hash_map::LinkedHashMap;
//using crate "linked_hash_map", https://github.com/contain-rs/linked-hash-map

use crate::material::Bxdf;
use crate::math::{Color, Point3, Vec3};
use crate::object::Object;
use crate::random::FreshId;
use crate::texture::Texture;

fn convert_to_coord(point: &LinkedHashMap<String, Property>) -> Point3 {
    let x = match point["x"] {
        Float(x) => x as f64,
        _ => 0.,
    };

    let y = match point["y"] {
        Float(y) => y as f64,
        _ => 0.,
    };

    let z = match point["z"] {
        Float(z) => z as f64,
        _ => 0.,
    };

    Vec3(x, y, z)
}

pub fn read_ply<'a>(
    file_path: &str,
    color: Color,
    bxdf: Bxdf,
    scale: f64,
    translation: Vec3,
    freshid: &'a mut FreshId,
) -> Vec<Object<'a>> {
    let mut file = std::fs::File::open(file_path).unwrap();
    let parser = ply::parser::Parser::<ply::ply::DefaultElement>::new();

    let ply = parser.read_ply(&mut file).unwrap();
    let points = &ply.payload["vertex"];
    let indices = &ply.payload["face"];

    let mut objects = Vec::new();

    for index in indices {
        let idx = match &index["vertex_indices"] {
            ListInt(v) => v,
            _ => &Vec::new(),
        };

        let p = convert_to_coord(&points[(idx[0]) as usize]) * scale + translation;
        let q = convert_to_coord(&points[(idx[1]) as usize]) * scale + translation;
        let r = convert_to_coord(&points[(idx[2]) as usize]) * scale + translation;

        let triangle = Object::set_tri(p, q, r, bxdf, Texture::SolidTex { color: color }, freshid);
        objects.push(triangle);
    }

    objects
}
