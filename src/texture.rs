use std::fs::File;

use crate::math::{Color, Vec3};

pub enum Texture<'a> {
    SolidTex {
        color: Color,
    },
    CheckerTex {
        div: u32,
        col1: Color,
        col2: Color,
    },
    ImageTex {
        data: &'a Vec<Color>,
        px_w: usize,
        px_h: usize,
    },
}

pub fn load_hdr(path: &str) -> (Vec<Color>, usize, usize) {
    let file = File::open(path).expect("failed to open hdr");
    let image = hdrldr::load(file).expect("failed to load hdr");

    let mut data = Vec::new();
    for rgb in image.data.iter() {
        data.push(Vec3(rgb.r as f64, rgb.g as f64, rgb.b as f64));
    }

    (data, image.width, image.height)
}

impl<'a> Texture<'a> {
    pub fn get_color(&self, u: f64, v: f64) -> Color {
        match *self {
            Texture::SolidTex { color } => color,
            Texture::CheckerTex { div, col1, col2 } => {
                let id_u = u * (div as f64);
                let id_v = v * (div as f64);
                if (id_u as u32 + id_v as u32) % 2 == 0 {
                    col1
                } else {
                    col2
                }
            }
            Texture::ImageTex { data, px_w, px_h } => {
                let id_u = (px_w as f64 * u) as usize;
                let id_v = (px_h as f64 * v) as usize;
                let id = id_v * px_w + id_u;
                data[id]
            }
        }
    }
}
