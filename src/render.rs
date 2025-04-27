use bmp::{px, Image, Pixel};
//using crate "bmp", https://github.com/sondrele/rust-bmp/tree/master/src

use crate::{
    camera::Camara,
    math::{gamma_rev, is_valid, Color},
    radiance::radiance,
    random::XorRand,
    ray::Ray,
    scene::Scene,
};

pub fn render(camera: &impl Camara, scene: &Scene) {
    let (pixel_w, pixel_h) = camera.get_pixel();
    let (spp, sspp) = camera.get_sample();
    let coeff = camera.get_coeff();

    let mut img = Image::new(pixel_w, pixel_h);

    for v in 0..pixel_h {
        for u in 0..pixel_w {
            let mut rand = XorRand::new(u * v);
            let mut accumlated_color = Color::new(0.);

            for sv in 0..sspp {
                for su in 0..sspp {
                    let (g_term, org, dir) = camera.setup(u, v, su, sv, &mut rand);

                    for _ in 0..spp {
                        let rad = radiance(scene, Ray { org, dir }, &mut rand) * g_term;
                        if !is_valid(&rad) {
                            continue;
                        }

                        accumlated_color = accumlated_color + rad;
                    }
                }
            }

            let rgb = gamma_rev(accumlated_color * coeff);
            img.set_pixel(u, v, px!(rgb.0, rgb.1, rgb.2));
        }
        println!("{v}");
    }
    let _ = img.save("render.bmp");
}
