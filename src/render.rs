use bmp::{px, Image, Pixel};
//using crate "bmp", https://github.com/sondrele/rust-bmp/tree/master/src
use rayon::prelude::*;
//using crate "rayon", https://github.com/rayon-rs/rayon

use crate::{
    camera::Camara,
    filter::{filter, guided_filter},
    math::{clamp_vec, gamma_rev, is_valid, Color, Vec3},
    pathtracing::Pathtracing,
    random::XorRand,
    ray::Ray,
    scene::Scene,
};

pub fn render(camera: &impl Camara, scene: &Scene) {
    let (pixel_w, pixel_h) = camera.get_pixel();
    let (spp, sspp) = camera.get_sample();
    let coeff = camera.get_coeff();

    let mut buffer = vec![Vec3::new(0.); (pixel_w * pixel_h) as usize];
    let mut img = Image::new(pixel_w, pixel_h);

    buffer
        .par_chunks_mut(pixel_w as usize)
        .enumerate()
        .for_each(|(v, row)| {
            for u in 0..pixel_w {
                let mut rand = XorRand::new(u * v as u32);
                let mut accumlated_color = Color::new(0.);

                for sv in 0..sspp {
                    for su in 0..sspp {
                        let (g_term, org, dir) = camera.setup(u, v as u32, su, sv, &mut rand);

                        for _ in 0..spp {
                            let mut tracer = Pathtracing::new(Ray { org, dir });
                            let rad = tracer.integrate(scene, &mut rand) * g_term;
                            //let rad = tracer.test_normal(scene) * g_term;
                            if !is_valid(&rad) {
                                continue;
                            }

                            accumlated_color = accumlated_color + rad;
                        }
                    }
                }
                row[u as usize] = clamp_vec(accumlated_color * coeff, 0., 1.);
            }
            println!("{v}");
        });

    let mut normals = vec![Vec3::new(0.); (pixel_w * pixel_h) as usize];

    normals
        .par_chunks_mut(pixel_w as usize)
        .enumerate()
        .for_each(|(v, row)| {
            for u in 0..pixel_w {
                let mut rand = XorRand::new(u * v as u32);
                let mut accumlated_color = Color::new(0.);

                for sv in 0..2 {
                    for su in 0..2 {
                        let (g_term, org, dir) = camera.setup(u, v as u32, su, sv, &mut rand);

                        for _ in 0..spp {
                            let mut tracer = Pathtracing::new(Ray { org, dir });
                            let rad = tracer.test_normal(scene) * g_term;
                            if !is_valid(&rad) {
                                continue;
                            }

                            accumlated_color = accumlated_color + rad;
                        }
                    }
                }
                row[u as usize] = accumlated_color * coeff;
            }
            println!("{v}");
        });

    //let filtered_buf = filter(&buffer, pixel_w as i32, pixel_h as i32, 5, 0.05);

    let filtered_buf = guided_filter(
        &buffer,
        &normals,
        pixel_w as i32,
        pixel_h as i32,
        5,
        0.01,
        0.1,
    );

    for i in 0..pixel_w * pixel_h {
        let y = i / pixel_w;
        let x = i - pixel_w * y;
        let rgb = gamma_rev(filtered_buf[i as usize]);
        img.set_pixel(x, y as u32, px!(rgb.0, rgb.1, rgb.2));
    }
    let _ = img.save("render.bmp");
}
