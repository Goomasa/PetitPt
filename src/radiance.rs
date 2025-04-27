use crate::{
    material::{reflection_dir, refraction_dir, sample_lambert, Bxdf},
    math::{dot, max_elm, multiply, Color, Vec3},
    random::XorRand,
    ray::{HitRecord, Ray},
    scene::Scene,
};

const DEPTH: u32 = 6;
const MAX_DEPTH: u32 = 30;

pub fn radiance(scene: &Scene, ray: Ray, rand: &mut XorRand) -> Color {
    let mut record = HitRecord::new();
    let mut now_ray = ray;
    let mut roulette_prob = 1.0;
    let mut pdf = 1.0;
    let mut orienting_normal;
    let mut throughput = Vec3::new(1.);
    let mut rad = Vec3::new(0.);

    for time in 0.. {
        record = HitRecord::new();
        if !scene.intersect(&now_ray, &mut record) {
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
                rad = rad + multiply(throughput, record.color) / pdf;
                break;
            }
            Bxdf::Lambertian => {
                let out_dir = sample_lambert(&orienting_normal, rand);
                now_ray = Ray {
                    org: record.pos + orienting_normal * 0.00001,
                    dir: out_dir,
                };
                throughput = multiply(throughput, record.color);
            }
            Bxdf::Specular => {
                let out_dir = reflection_dir(orienting_normal, now_ray.dir);
                now_ray = Ray {
                    org: record.pos + orienting_normal * 0.00001,
                    dir: out_dir,
                };
                throughput = multiply(throughput, record.color);
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
            }
        }
    }

    rad
}
