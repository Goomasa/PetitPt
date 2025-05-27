use crate::{
    material::*,
    math::{dot, max_elm, multiply, Color, Vec3, EPS, PI},
    object::sphere_uv,
    random::XorRand,
    ray::{HitRecord, Ray},
    scene::Scene,
    texture::{sample_hdr_pdf, Texture},
};

const DEPTH: u32 = 6;
const MAX_DEPTH: u32 = 30;
const PI_INV: f64 = 1. / PI;

pub struct Pathtracing {
    record: HitRecord,
    now_ray: Ray,
    pdf: f64,
    orienting_normal: Vec3,
    throughput: Vec3,
    rad: Color,
    brdf_sample_pdf: f64,
    ior_stack: Vec<(i32, f64)>,
}

impl Pathtracing {
    pub fn new(ray: Ray) -> Self {
        Pathtracing {
            record: HitRecord::new(),
            now_ray: ray,
            pdf: 1.,
            orienting_normal: Vec3::new(0.),
            throughput: Vec3::new(1.),
            rad: Vec3::new(0.),
            brdf_sample_pdf: -1.,
            ior_stack: vec![(-1, 1.)],
        }
    }

    fn roulette(&mut self, time: u32) -> f64 {
        let mut prob = match self.record.bxdf {
            Bxdf::Light => 1.,
            _ => max_elm(&self.record.color),
        };

        if time > MAX_DEPTH {
            prob /= 2_i32.pow(time - MAX_DEPTH) as f64;
        } else if time <= DEPTH {
            prob = 1.;
        }

        prob
    }

    fn trace_light(&mut self, scene: &Scene) {
        if self.brdf_sample_pdf < 0. {
            self.rad = self.rad + multiply(self.throughput, self.record.color) / self.pdf;
        } else {
            let nee_pdf = scene.sample_obj_pdf(self.now_ray.org, &self.record);
            let mis_weight = self.brdf_sample_pdf / (self.brdf_sample_pdf + nee_pdf);
            self.rad =
                self.rad + multiply(self.throughput, self.record.color) * mis_weight / self.pdf;
        }
    }

    fn trace_lambertian(&mut self, scene: &Scene, rand: &mut XorRand) {
        let dir = sample_lambert(&self.orienting_normal, rand);
        let org = self.record.pos + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        self.throughput = multiply(self.throughput, self.record.color);
        let nee_result = scene.nee(org, rand);

        if nee_result.pdf != 0. {
            let nee_dir_cos = dot(self.orienting_normal, nee_result.dir).abs();
            let mis_weight = 1. / (nee_result.pdf + nee_dir_cos * PI_INV);
            self.rad = self.rad
                + multiply(self.throughput, nee_result.color * PI_INV) * nee_dir_cos * mis_weight
                    / self.pdf;
        }
        self.brdf_sample_pdf = sample_lambert_pdf(&dir, &self.orienting_normal);
    }

    fn trace_specular(&mut self, cior: &Color, k: &Color) {
        let out_dir = reflection_dir(self.orienting_normal, self.now_ray.dir);
        self.now_ray = Ray {
            org: self.record.pos + self.orienting_normal * 0.00001,
            dir: out_dir,
        };

        let fresnel = if cior.0 < 0. {
            fr_dielectric_col(&self.record.color, &out_dir, &self.orienting_normal)
        } else {
            fr_conductor(cior, k, &out_dir, &self.orienting_normal)
        };
        self.throughput = multiply(self.throughput, fresnel);
        self.brdf_sample_pdf = -1.;
    }

    fn trace_dielectric(&mut self, ior_mat: f64, rand: &mut XorRand) {
        let (id, ior_env) = self.ior_stack[self.ior_stack.len() - 1];
        let into = id == -1; // TODO: set bxdf-id?

        let ior_env = if into {
            ior_env
        } else {
            self.ior_stack[self.ior_stack.len() - 2].1
        };

        //let ior_env = 1.;
        //let into = dot(self.record.normal, self.now_ray.dir) < 0.;

        let (is_refract, out_dir, fresnel, refl_prob) = refraction_dir(
            into,
            ior_env,
            ior_mat,
            self.orienting_normal,
            self.now_ray.dir,
            rand,
        );

        let new_org;
        let nnt;
        if is_refract {
            new_org = self.record.pos - self.orienting_normal * 0.00001;
            if into {
                nnt = ior_env / ior_mat;
                self.ior_stack.push((self.record.obj_id, ior_mat));
            } else {
                nnt = ior_mat / ior_env;
                let _ = self.ior_stack.pop();
            }
        } else {
            new_org = self.record.pos + self.orienting_normal * 0.00001;
            nnt = 1.;
        }

        self.now_ray = Ray {
            org: new_org,
            dir: out_dir,
        };

        self.throughput = multiply(self.throughput, self.record.color) * fresnel * nnt * nnt;
        self.pdf *= refl_prob;
        self.brdf_sample_pdf = -1.;
    }

    fn trace_microbrdf(
        &mut self,
        scene: &Scene,
        rand: &mut XorRand,
        ax: f64,
        ay: f64,
        cior: &Color,
        k: &Color,
    ) {
        let wi = -self.now_ray.dir;
        let vn = sample_ggx_vndf(&self.orienting_normal, &wi, ax, ay, rand);
        let dir = reflection_dir(vn, -wi);
        let alpha_sq = ggx_alpha2(ax, ay, &vn, &self.orienting_normal);
        let g1_wo = shadow_mask_fn(alpha_sq, &dir, &self.orienting_normal);
        let fresnel = if cior.0 < 0. {
            fr_dielectric_col(&self.record.color, &dir, &vn)
        } else {
            fr_conductor(cior, k, &dir, &vn)
        };

        let org = self.record.pos + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        let nee_result = scene.nee(org, rand);

        let g1_wi = shadow_mask_fn(alpha_sq, &wi, &self.orienting_normal);
        let d_vn = ggx_normal_df(alpha_sq, ax, ay, &self.orienting_normal, &vn);
        let dot_wi_n = dot(wi, self.orienting_normal).abs();
        let vndf = g1_wi * d_vn / (4. * dot_wi_n);

        if nee_result.pdf != 0. {
            let nee_vn = (wi + nee_result.dir).normalize();
            let d_nee_vn = ggx_normal_df(alpha_sq, ax, ay, &self.orienting_normal, &nee_vn);
            let nee_vndf = g1_wi * d_nee_vn / (4. * dot_wi_n);

            let g1_nee_wo = shadow_mask_fn(alpha_sq, &nee_result.dir, &self.orienting_normal);
            let mis_weight = 1. / (nee_result.pdf + nee_vndf);
            let nee_fresnel = if cior.0 < 0. {
                fr_dielectric_col(&self.record.color, &nee_result.dir, &nee_vn)
            } else {
                fr_conductor(cior, k, &nee_result.dir, &nee_vn)
            };
            let brdf = nee_fresnel * nee_vndf * g1_nee_wo;
            self.rad = self.rad
                + multiply(nee_result.color, multiply(self.throughput, brdf)) * mis_weight
                    / self.pdf;
        }

        self.throughput = multiply(self.throughput, fresnel * g1_wo);
        if ax == 0. || ay == 0. {
            self.brdf_sample_pdf = -1.;
        } else {
            self.brdf_sample_pdf = vndf;
        }
    }

    fn trace_microbtdf(&mut self, scene: &Scene, rand: &mut XorRand, a: f64, ior_mat: f64) {
        let wi = -self.now_ray.dir;
        let vn = sample_ggx_vndf(&self.orienting_normal, &wi, a, a, rand);
        let alpha_sq = a * a;

        let (id, ior_env) = self.ior_stack[self.ior_stack.len() - 1];
        let into = id == -1;

        let ior_env = if into {
            ior_env
        } else {
            self.ior_stack[self.ior_stack.len() - 2].1
        };

        let (is_refract, dir, fresnel, refl_prob) =
            refraction_dir(into, ior_env, ior_mat, vn, self.now_ray.dir, rand);

        let g1_wo = shadow_mask_fn(alpha_sq, &dir, &self.orienting_normal);
        let g1_wi = shadow_mask_fn(alpha_sq, &wi, &self.orienting_normal);
        let d_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &vn);
        let dot_wi_n = dot(wi, self.orienting_normal).abs();

        if is_refract {
            let org = self.record.pos - self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir };

            let ja;
            if into {
                ja = micro_btdf_j(ior_env, ior_mat, &wi, &dir, &vn);
                self.ior_stack.push((self.record.obj_id, ior_mat));
            } else {
                ja = micro_btdf_j(ior_mat, ior_env, &wi, &dir, &vn);
                let _ = self.ior_stack.pop();
            };

            let vndf = g1_wi * dot(wi, vn) * d_vn * ja / dot_wi_n;

            let nee_result = scene.nee(org, rand);
            if nee_result.pdf != 0. {
                let nee_wh;
                let ja;
                if into {
                    nee_wh = -(wi + nee_result.dir * ior_mat).normalize();
                    ja = micro_btdf_j(ior_env, ior_mat, &wi, &nee_result.dir, &nee_wh);
                } else {
                    nee_wh = -(wi * ior_mat + nee_result.dir).normalize();
                    ja = micro_btdf_j(ior_mat, ior_env, &wi, &nee_result.dir, &nee_wh);
                }

                if dot(self.orienting_normal, nee_wh) > EPS {
                    let g1_nee_wo =
                        shadow_mask_fn(alpha_sq, &nee_result.dir, &self.orienting_normal);
                    let d_nee_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &nee_wh);
                    let nee_vndf = g1_wi * dot(wi, nee_wh) * d_nee_vn * ja / dot_wi_n;
                    let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                    let nee_fresnel =
                        fr_dielectric_ior(into, ior_env, ior_mat, &nee_result.dir, &nee_wh);
                    let nee_btdf = (1. - nee_fresnel) * g1_nee_wo * nee_vndf * dot(wi, nee_wh);
                    self.rad = self.rad
                        + multiply(
                            nee_result.color,
                            multiply(self.throughput, self.record.color),
                        ) * nee_btdf
                            * mis_weight
                            / self.pdf;
                }
            }

            self.throughput = multiply(self.throughput, self.record.color) * fresnel * g1_wo;
            self.pdf *= refl_prob;
            self.brdf_sample_pdf = if a == 0. { -1. } else { vndf };
        } else {
            let org = self.record.pos + self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir };

            let vndf = g1_wi * d_vn / (4. * dot_wi_n);

            let nee_result = scene.nee(org, rand);
            if nee_result.pdf != 0. {
                let nee_vn = (wi + nee_result.dir).normalize();
                let d_nee_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &nee_vn);
                let nee_vndf = g1_wi * d_nee_vn / (4. * dot_wi_n);

                let g1_nee_wo = shadow_mask_fn(alpha_sq, &nee_result.dir, &self.orienting_normal);
                let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                let nee_fresnel = fr_dielectric_col(&self.record.color, &nee_result.dir, &nee_vn);
                let brdf = nee_fresnel * nee_vndf * g1_nee_wo;
                self.rad = self.rad
                    + multiply(nee_result.color, multiply(self.throughput, brdf)) * mis_weight
                        / self.pdf;
            }

            self.throughput = multiply(self.throughput, self.record.color) * fresnel * g1_wo;
            self.pdf *= refl_prob;
            self.brdf_sample_pdf = if a == 0. { -1. } else { vndf };
        }
    }

    pub fn integrate(&mut self, scene: &Scene, rand: &mut XorRand) -> Color {
        for time in 0.. {
            self.record = HitRecord::new();
            if !scene.intersect(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
                let (u, v) = sphere_uv(&self.now_ray.dir, &Vec3::new(0.));

                let mis_weight = if let Texture::ImageTex {
                    cdf,
                    cdf_row,
                    px_w,
                    px_h,
                    ..
                } = &scene.background
                {
                    let pdf = sample_hdr_pdf(cdf, cdf_row, u, v, *px_w, *px_h);
                    self.brdf_sample_pdf / (self.brdf_sample_pdf + pdf)
                } else {
                    1.
                };

                let background = scene.background.get_color(u, v);
                self.rad = self.rad + multiply(self.throughput, background) * mis_weight / self.pdf;
                break;
            }

            let roulette_prob = self.roulette(time);
            if rand.next01() > roulette_prob {
                break;
            }
            self.pdf *= roulette_prob;

            self.orienting_normal = if dot(self.record.normal, self.now_ray.dir) < 0. {
                self.record.normal
            } else {
                -self.record.normal
            };

            match self.record.bxdf {
                Bxdf::Light => {
                    self.trace_light(scene);
                    break;
                }
                Bxdf::Lambertian => {
                    self.trace_lambertian(scene, rand);
                }
                Bxdf::Specular { cior, k } => {
                    self.trace_specular(&cior, &k);
                }
                Bxdf::Dielectric { ior } => {
                    self.trace_dielectric(ior, rand);
                }
                Bxdf::MicroBrdf { ax, ay, cior, k } => {
                    self.trace_microbrdf(scene, rand, ax, ay, &cior, &k);
                }
                Bxdf::MicroBtdf { a, ior } => {
                    self.trace_microbtdf(scene, rand, a, ior);
                }
            }
        }
        self.rad
    }
}
