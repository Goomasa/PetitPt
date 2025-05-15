use crate::{
    material::*,
    math::{dot, max_elm, multiply, Color, Vec3, PI},
    random::XorRand,
    ray::{HitRecord, Ray},
    scene::Scene,
};

const DEPTH: u32 = 6;
const MAX_DEPTH: u32 = 30;
const PI_INV: f64 = 1. / PI;

pub fn radiance(scene: &Scene, ray: Ray, rand: &mut XorRand) -> Color {
    let mut record;
    let mut now_ray = ray;
    let mut roulette_prob;
    let mut pdf = 1.0;
    let mut orienting_normal;
    let mut throughput = Vec3::new(1.);
    let mut rad = Vec3::new(0.);

    let mut brdf_sample_pdf = -1.;

    for time in 0.. {
        record = HitRecord::new();
        if !scene.intersect(&now_ray, &mut record, &scene.bvh_tree[0]) {
            rad = rad + multiply(throughput, scene.background) / pdf;
            break;
        }

        roulette_prob = match record.bxdf {
            Bxdf::Light => 1.0,
            _ => max_elm(&record.color),
        };

        if time > MAX_DEPTH {
            roulette_prob /= 2_i32.pow(time - MAX_DEPTH) as f64;
        }

        if time > DEPTH {
            if rand.next01() >= roulette_prob {
                break;
            }
        } else {
            roulette_prob = 1.0;
        }

        pdf *= roulette_prob;
        orienting_normal = if dot(record.normal, now_ray.dir) < 0. {
            record.normal
        } else {
            -record.normal
        };

        match record.bxdf {
            Bxdf::Light => {
                if brdf_sample_pdf < 0. {
                    rad = rad + multiply(throughput, record.color) / pdf;
                } else {
                    let nee_pdf = scene.sample_obj_pdf(now_ray.org, &record);
                    let mis_weight = brdf_sample_pdf / (brdf_sample_pdf + nee_pdf);
                    rad = rad + multiply(throughput, record.color) * mis_weight / pdf;
                }
                break;
            }
            Bxdf::Lambertian => {
                let dir = sample_lambert(&orienting_normal, rand);
                let org = record.pos + orienting_normal * 0.00001;
                now_ray = Ray { org, dir };

                throughput = multiply(throughput, record.color);
                let nee_result = scene.nee(org, rand);

                if nee_result.pdf != 0. {
                    let nee_dir_cos = dot(orienting_normal, nee_result.dir).abs();
                    let mis_weight = 1. / (nee_result.pdf + nee_dir_cos * PI_INV);
                    rad = rad
                        + multiply(throughput, nee_result.color * PI_INV)
                            * nee_dir_cos
                            * mis_weight
                            / pdf;
                }
                brdf_sample_pdf = sample_lambert_pdf(&dir, &orienting_normal);
            }
            Bxdf::Specular => {
                let out_dir = reflection_dir(orienting_normal, now_ray.dir);
                now_ray = Ray {
                    org: record.pos + orienting_normal * 0.00001,
                    dir: out_dir,
                };
                throughput = multiply(throughput, record.color);
                brdf_sample_pdf = 1.;
            }
            Bxdf::Dielectric { ior } => {
                let into = dot(record.normal, now_ray.dir) < 0.;
                let (is_refract, out_dir, fresnel, refl_prob) =
                    refraction_dir(into, ior, orienting_normal, now_ray.dir, rand);

                let new_org = if is_refract {
                    record.pos - orienting_normal * 0.00001
                } else {
                    record.pos + orienting_normal * 0.00001
                };

                now_ray = Ray {
                    org: new_org,
                    dir: out_dir,
                };

                throughput = multiply(throughput, record.color) * fresnel;
                pdf *= refl_prob;
                brdf_sample_pdf = 1.;
            }
            Bxdf::MicroBrdf { ax, ay } => {
                let wi = -now_ray.dir;
                let vn = sample_ggx_vndf(&orienting_normal, &wi, ax, ay, rand);
                let dir = reflection_dir(vn, -wi);
                let alpha_sq = ggx_alpha2(ax, ay, &vn, &orienting_normal);
                let g1_wo = mask_shadow_fn(alpha_sq, &dir, &orienting_normal);
                let fresnel = fresnel_dielectric(&record.color, &wi, &vn);

                let org = record.pos + orienting_normal * 0.00001;
                now_ray = Ray { org, dir };

                let nee_result = scene.nee(org, rand);

                let g1_wi = mask_shadow_fn(alpha_sq, &wi, &orienting_normal);
                let d_vn = ggx_normal_df(alpha_sq, ax, ay, &orienting_normal, &vn);
                let dot_wi_n = dot(wi, orienting_normal).abs();
                let vndf = g1_wi * d_vn / (4. * dot_wi_n);

                if nee_result.pdf != 0. {
                    let nee_vn = (wi + nee_result.dir).normalize();
                    let d_nee_vn = ggx_normal_df(alpha_sq, ax, ay, &orienting_normal, &nee_vn);
                    let nee_vndf = g1_wi * d_nee_vn / (4. * dot_wi_n);

                    let g1_nee_wo = mask_shadow_fn(alpha_sq, &nee_result.dir, &orienting_normal);
                    let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                    let nee_fresnel = fresnel_dielectric(&record.color, &wi, &nee_vn);
                    let brdf = nee_fresnel * nee_vndf * g1_nee_wo;
                    rad = rad
                        + multiply(nee_result.color, multiply(throughput, brdf)) * mis_weight / pdf;
                }

                throughput = multiply(throughput, fresnel * g1_wo);
                brdf_sample_pdf = vndf;
            }
            Bxdf::MicroBtdf { a, ior } => {
                let wi = -now_ray.dir;
                let vn = sample_ggx_vndf(&orienting_normal, &wi, a, a, rand);
                let alpha_sq = a * a;

                let into = dot(record.normal, now_ray.dir) < 0.;
                let (is_refract, dir, fresnel, refl_prob) =
                    refraction_dir(into, ior, vn, now_ray.dir, rand);

                let g1_wo = mask_shadow_fn(alpha_sq, &dir, &orienting_normal);

                if is_refract {
                    let org = record.pos - orienting_normal * 0.00001;
                    now_ray = Ray { org, dir };

                    let ja;
                    let wh;
                    if into {
                        wh = -(wi + dir * ior).normalize();
                        ja = micro_btdf_j(1., ior, &wi, &dir, &wh);
                    } else {
                        wh = -(wi * ior + dir).normalize();
                        ja = micro_btdf_j(ior, 1., &wi, &dir, &wh);
                    };

                    throughput =
                        multiply(throughput, record.color) * fresnel * g1_wo * dot(dir, wh).abs()
                            / dot(wi, vn).abs();
                    pdf *= refl_prob;
                    brdf_sample_pdf = -1.;
                } else {
                    let org = record.pos + orienting_normal * 0.00001;
                    now_ray = Ray { org, dir };

                    throughput = multiply(throughput, record.color) * fresnel * g1_wo;
                    pdf *= refl_prob;
                    brdf_sample_pdf = -1.;
                }
            }
        }
    }
    rad
}
