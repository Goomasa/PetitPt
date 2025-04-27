use bmp::{px, Image, Pixel};

use crate::{
    camera::Camera,
    math::{gamma_rev, is_valid, Color, PI},
    radiance::radiance,
    random::XorRand,
    ray::Ray,
    scene::Scene,
};

pub fn render(camera: &Camera, scene: &Scene) {
    let pixel_w = camera.pixel_w;
    let pixel_h = camera.pixel_h;
    let sensor_w_per_px = camera.sensor_w / pixel_w as f64;
    let sensor_h_per_px = camera.sensor_h / pixel_h as f64;
    let superpx_u = camera.sensor_u * sensor_w_per_px / camera.sspp as f64;
    let superpx_v = camera.sensor_v * sensor_h_per_px / camera.sspp as f64;

    let sensor_corner = camera.sensor_center
        - camera.sensor_u * camera.sensor_w / 2.
        - camera.sensor_v * camera.sensor_h / 2.;

    let pdf_inv =
        sensor_w_per_px * sensor_h_per_px * (PI * camera.lens_radius * camera.lens_radius)
            / ((camera.spp as f64) * (camera.sspp.pow(2) as f64));

    let mut img = Image::new(pixel_w, pixel_h);

    for v in 0..pixel_h {
        for u in 0..pixel_w {
            let mut rand = XorRand::new(u * v);
            let mut accumlated_color = Color::new(0.);

            for sv in 0..camera.sspp {
                for su in 0..camera.sspp {
                    let pixel_pos = sensor_corner
                        + superpx_u * ((u * camera.sspp + su) as f64 + 0.5)
                        + superpx_v * ((v * camera.sspp + sv) as f64 + 0.5);
                    let (coeff, org) = camera.sample_lens(pixel_pos, &mut rand);
                    let dir = camera.first_dir(pixel_pos, org);

                    for _ in 0..camera.spp {
                        let rad = radiance(scene, Ray { org, dir }, &mut rand) * coeff;
                        if !is_valid(&rad) {
                            continue;
                        }

                        accumlated_color = accumlated_color + rad;
                    }
                }
            }

            let rgb = gamma_rev(accumlated_color * pdf_inv * camera.iso);
            img.set_pixel(pixel_w - u - 1, pixel_h - v - 1, px!(rgb.0, rgb.1, rgb.2));
        }
        println!("{v}");
    }
    let _ = img.save("render.bmp");
}
