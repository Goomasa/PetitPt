use std::fs::File;

use crate::{
    math::{Color, Vec3, PI},
    random::XorRand,
};

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
        cdf: &'a Vec<Vec<f64>>,
        cdf_row: Box<Vec<f64>>,
        px_w: usize,
        px_h: usize,
    },
}

impl<'a> Texture<'a> {
    pub fn set_solid(col: Color) -> Self {
        Texture::SolidTex { color: col }
    }

    pub fn set_checker(div: u32, col1: Color, col2: Color) -> Self {
        Texture::CheckerTex { div, col1, col2 }
    }

    pub fn set_image(
        data: &'a Vec<Color>,
        cdf: &'a Vec<Vec<f64>>,
        px_w: usize,
        px_h: usize,
    ) -> Self {
        let mut max_row = vec![0.; px_h];
        for h in 0..px_h {
            let mut max = 0.;
            let mut sum = 0.;
            for w in 0..px_w {
                let id = h * px_w + w;
                if data[id].length() > max {
                    max = data[id].length();
                }
                sum += data[id].length();
            }
            max_row[h] = sum / px_w as f64;
        }

        let cdf_row = Box::new(make_cdf_1d(&max_row));

        Texture::ImageTex {
            data,
            cdf,
            cdf_row,
            px_w,
            px_h,
        }
    }

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
            Texture::ImageTex {
                data, px_w, px_h, ..
            } => {
                let id_u = (px_w as f64 * u) as usize;
                let id_v = (px_h as f64 * v) as usize;
                let id = id_v * px_w + id_u;
                data[id]
            }
        }
    }

    pub fn sample_hdr(
        &self,
        cdf_hdr: &Vec<Vec<f64>>,
        cdf_hdr_row: &Box<Vec<f64>>,
        px_w: usize,
        px_h: usize,
        rand: &mut XorRand,
    ) -> (Color, Vec3, f64) {
        let r1 = rand.next01();
        let k1 = binary_search(&cdf_hdr_row, r1);
        let pdf_v = if k1 + 1 != px_h {
            cdf_hdr_row[k1 + 1] - cdf_hdr_row[k1]
        } else {
            1. - cdf_hdr_row[k1]
        };
        let v = (r1 - cdf_hdr_row[k1]) / (px_h as f64 * pdf_v) + k1 as f64 / px_h as f64;

        let r2 = rand.next01();
        let k2 = binary_search(&cdf_hdr[k1], r2);
        let pdf_u = if k2 + 1 != px_w {
            cdf_hdr[k1][k2 + 1] - cdf_hdr[k1][k2]
        } else {
            1. - cdf_hdr[k1][k2]
        };
        let u = (r2 - cdf_hdr[k1][k2]) / (px_w as f64 * pdf_u) + k2 as f64 / px_w as f64;

        let sin_theta = (v * PI).sin();
        let cos_theta = (v * PI).cos();
        let phi = 2. * PI * u;

        let dir = Vec3(sin_theta * phi.sin(), cos_theta, sin_theta * phi.cos());

        (
            self.get_color(u, v),
            dir,
            pdf_u * pdf_v / (2. * PI * sin_theta),
        )
    }
}

pub fn load_hdr(path: &str) -> (Vec<Color>, usize, usize) {
    let file = File::open(path).expect("failed to open hdr");
    let image = hdrldr::load(file).expect("failed to load hdr");

    let mut data = Vec::new();
    for rgb in image.data.iter() {
        data.push(Vec3(
            rgb.r.clamp(0., 10.) as f64,
            rgb.g.clamp(0., 10.) as f64,
            rgb.b.clamp(0., 10.) as f64,
        ));
    }

    (data, image.width, image.height)
}

pub fn make_cdf_hdr(hdr: &Vec<Color>, px_w: usize, px_h: usize) -> Vec<Vec<f64>> {
    let mut cdf = vec![vec![0.; px_w]; px_h];

    for h in 0..px_h {
        for w in 0..px_w {
            if w == 0 {
                cdf[h][w] = 0.;
            } else {
                cdf[h][w] = cdf[h][w - 1] + hdr[h * px_w + w].length();
            }
        }

        for w in 0..px_w {
            cdf[h][w] /= cdf[h][px_w - 1] + hdr[h * px_w + px_w - 1].length();
        }
    }

    cdf
}

fn make_cdf_1d(v: &Vec<f64>) -> Vec<f64> {
    let mut cdf = vec![0.; v.len()];
    let sum: f64 = v.iter().sum();

    for i in 0..v.len() {
        if i == 0 {
            cdf[i] = 0.;
        } else {
            cdf[i] = cdf[i - 1] + v[i] / sum;
        }
    }

    cdf
}

fn binary_search(cdf: &Vec<f64>, p: f64) -> usize {
    let mut left: usize = 0;
    let mut right: usize = cdf.len() - 1;
    let mut id: usize;

    while left < right {
        id = (left + right) / 2;
        if cdf[id] <= p && cdf[id + 1] > p {
            return id;
        } else if cdf[id] <= p && cdf[id + 1] <= p {
            left = id + 1;
        } else if cdf[id] > p {
            right = id;
        }
    }

    cdf.len() - 1
}

pub fn sample_hdr_pdf(
    cdf_hdr: &Vec<Vec<f64>>,
    cdf_hdr_row: &Box<Vec<f64>>,
    u: f64,
    v: f64,
    px_w: usize,
    px_h: usize,
) -> f64 {
    let id_u = (u * px_w as f64) as usize;
    let id_v = (v * px_h as f64) as usize;

    let pdf_u = if id_u + 1 != px_w {
        cdf_hdr[id_v][id_u + 1] - cdf_hdr[id_v][id_u]
    } else {
        1. - cdf_hdr[id_v][id_u]
    };

    let pdf_v = if id_v + 1 != px_h {
        cdf_hdr_row[id_v + 1] - cdf_hdr_row[id_v]
    } else {
        1. - cdf_hdr_row[id_v]
    };

    pdf_u * pdf_v / (2. * PI * (v * PI).cos())
}
