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
    roulette_pdf: f64,
    orienting_normal: Vec3,
    throughput: Vec3,
    rad: Color,
    pt_sample_pdf: f64,
    medium_stack: Vec<(i32, f64, f64, f64)>, // (trans_id, ior, sigma_scatter, sigma_extinct)
}

impl Pathtracing {
    pub fn new(ray: Ray) -> Self {
        Pathtracing {
            record: HitRecord::new(),
            now_ray: ray,
            roulette_pdf: 1.,
            orienting_normal: Vec3::new(0.),
            throughput: Vec3::new(1.),
            rad: Vec3::new(0.),
            pt_sample_pdf: -1.,
            medium_stack: vec![(-1, 1., -1., 0.)],
        }
    }

    fn get_sigma_e(&mut self) -> f64 {
        self.medium_stack.last().unwrap().3
    }

    fn is_into(&mut self, trans_id: i32) -> bool {
        for (id, ..) in self.medium_stack.iter() {
            if *id == trans_id {
                return false;
            }
        }
        true
    }

    fn has_ior(&mut self) -> bool {
        for (_, ior, ..) in self.medium_stack.iter() {
            if *ior != 1. {
                return true;
            }
        }
        false
    }

    fn has_medium(&mut self) -> bool {
        if self.medium_stack.last().unwrap().2 < 0. {
            return false;
        }
        true
    }

    fn remove_medium(&mut self, trans_id: i32) {
        if let Some(idx) = self.medium_stack.iter().position(|(x, ..)| *x == trans_id) {
            self.medium_stack.remove(idx);
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

    fn ray_intersect(&mut self, scene: &Scene) -> bool {
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
                self.pt_sample_pdf / (self.pt_sample_pdf + pdf)
            } else {
                1.
            };

            let background = scene.background.get_color(u, v);
            self.rad =
                self.rad + multiply(self.throughput, background) * mis_weight / self.roulette_pdf;
            return false;
        }
        true
    }

    fn trace_light(&mut self, scene: &Scene) {
        if self.pt_sample_pdf < 0. {
            self.rad = self.rad + multiply(self.throughput, self.record.color) / self.roulette_pdf;
        } else {
            let nee_pdf = scene.sample_obj_pdf(self.now_ray.org, &self.record);
            let mis_weight = self.pt_sample_pdf / (self.pt_sample_pdf + nee_pdf);
            self.rad = self.rad
                + multiply(self.throughput, self.record.color) * mis_weight / self.roulette_pdf;
        }
    }

    fn trace_lambertian(&mut self, scene: &Scene, rand: &mut XorRand) {
        let dir = sample_lambert(&self.orienting_normal, rand);
        let org = self.record.pos + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        self.throughput = multiply(self.throughput, self.record.color);
        let (nee_result, transmittance) = scene.nee(org, rand, self.get_sigma_e());

        if nee_result.pdf != 0. {
            let nee_dir_cos = dot(self.orienting_normal, nee_result.dir).abs();
            let mis_weight = 1. / (nee_result.pdf + nee_dir_cos * PI_INV);
            self.rad = self.rad
                + multiply(self.throughput, nee_result.color * PI_INV)
                    * nee_dir_cos
                    * transmittance
                    * mis_weight
                    / self.roulette_pdf;
        }
        self.pt_sample_pdf = sample_lambert_pdf(&dir, &self.orienting_normal);
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
        self.pt_sample_pdf = -1.;
    }

    fn trace_dielectric(&mut self, ior_mat: f64, rand: &mut XorRand, trans_id: i32) {
        let into = self.is_into(trans_id);

        let (is_refract, out_dir, fresnel, refl_prob) = refraction_dir(
            into,
            1.,
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
                nnt = 1. / ior_mat;
                self.medium_stack.push((trans_id, ior_mat, -1., 0.));
            } else {
                nnt = ior_mat;
                self.remove_medium(trans_id);
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
        self.roulette_pdf *= refl_prob;
        self.pt_sample_pdf = -1.;
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

        let (nee_result, transmittance) = scene.nee(org, rand, self.get_sigma_e());

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
                + multiply(nee_result.color, multiply(self.throughput, brdf))
                    * transmittance
                    * mis_weight
                    / self.roulette_pdf;
        }

        self.throughput = multiply(self.throughput, fresnel * g1_wo);
        if ax == 0. || ay == 0. {
            self.pt_sample_pdf = -1.;
        } else {
            self.pt_sample_pdf = vndf;
        }
    }

    fn trace_microbtdf(
        &mut self,
        scene: &Scene,
        rand: &mut XorRand,
        a: f64,
        ior_mat: f64,
        trans_id: i32,
    ) {
        let wi = -self.now_ray.dir;
        let vn = sample_ggx_vndf(&self.orienting_normal, &wi, a, a, rand);
        let alpha_sq = a * a;

        let into = self.is_into(trans_id);

        let (is_refract, dir, fresnel, refl_prob) =
            refraction_dir(into, 1., ior_mat, vn, self.now_ray.dir, rand);

        let g1_wo = shadow_mask_fn(alpha_sq, &dir, &self.orienting_normal);
        let g1_wi = shadow_mask_fn(alpha_sq, &wi, &self.orienting_normal);
        let d_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &vn);
        let dot_wi_n = dot(wi, self.orienting_normal).abs();

        if is_refract {
            let org = self.record.pos - self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir };

            let ja;
            if into {
                ja = micro_btdf_j(1., ior_mat, &wi, &dir, &vn);
                self.medium_stack.push((trans_id, ior_mat, -1., 0.));
            } else {
                ja = micro_btdf_j(ior_mat, 1., &wi, &dir, &vn);
                self.remove_medium(trans_id);
            };

            let vndf = g1_wi * dot(wi, vn) * d_vn * ja / dot_wi_n;

            let (nee_result, transmittance) = scene.nee(org, rand, self.get_sigma_e());
            if nee_result.pdf != 0. {
                let nee_wh;
                let ja;
                if into {
                    nee_wh = -(wi + nee_result.dir * ior_mat).normalize();
                    ja = micro_btdf_j(1., ior_mat, &wi, &nee_result.dir, &nee_wh);
                } else {
                    nee_wh = -(wi * ior_mat + nee_result.dir).normalize();
                    ja = micro_btdf_j(ior_mat, 1., &wi, &nee_result.dir, &nee_wh);
                }

                if dot(self.orienting_normal, nee_wh) > EPS {
                    let g1_nee_wo =
                        shadow_mask_fn(alpha_sq, &nee_result.dir, &self.orienting_normal);
                    let d_nee_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &nee_wh);
                    let nee_vndf = g1_wi * dot(wi, nee_wh) * d_nee_vn * ja / dot_wi_n;
                    let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                    let nee_fresnel =
                        fr_dielectric_ior(into, 1., ior_mat, &nee_result.dir, &nee_wh);
                    let nee_btdf = (1. - nee_fresnel) * g1_nee_wo * nee_vndf * dot(wi, nee_wh);
                    self.rad = self.rad
                        + multiply(
                            nee_result.color,
                            multiply(self.throughput, self.record.color),
                        ) * nee_btdf
                            * transmittance
                            * mis_weight
                            / self.roulette_pdf;
                }
            }

            self.throughput = multiply(self.throughput, self.record.color) * fresnel * g1_wo;
            self.roulette_pdf *= refl_prob;
            self.pt_sample_pdf = if a == 0. { -1. } else { vndf };
        } else {
            let org = self.record.pos + self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir };

            let vndf = g1_wi * d_vn / (4. * dot_wi_n);

            let (nee_result, transmittance) = scene.nee(org, rand, self.get_sigma_e());
            if nee_result.pdf != 0. {
                let nee_vn = (wi + nee_result.dir).normalize();
                let d_nee_vn = ggx_normal_df(alpha_sq, a, a, &self.orienting_normal, &nee_vn);
                let nee_vndf = g1_wi * d_nee_vn / (4. * dot_wi_n);

                let g1_nee_wo = shadow_mask_fn(alpha_sq, &nee_result.dir, &self.orienting_normal);
                let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                let nee_fresnel = fr_dielectric_col(&self.record.color, &nee_result.dir, &nee_vn);
                let brdf = nee_fresnel * nee_vndf * g1_nee_wo;
                self.rad = self.rad
                    + multiply(nee_result.color, multiply(self.throughput, brdf))
                        * transmittance
                        * mis_weight
                        / self.roulette_pdf;
            }

            self.throughput = multiply(self.throughput, self.record.color) * fresnel * g1_wo;
            self.roulette_pdf *= refl_prob;
            self.pt_sample_pdf = if a == 0. { -1. } else { vndf };
        }
    }

    pub fn freepath_sample(&mut self, scene: &Scene, rand: &mut XorRand) -> bool {
        let (_, _, sigma_s, sigma_e) = self.medium_stack.last().unwrap();
        let dist = -1. * (rand.next01()).ln() / sigma_e;

        self.record = HitRecord::init_with_dist(dist);
        if !scene.intersect(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
            self.throughput = self.throughput * *sigma_s / *sigma_e;
            let org = self.now_ray.org + self.now_ray.dir * dist;
            let dir = sample_hg_phase(&self.now_ray.dir, 0.8, rand);
            let hg_pdf = hg_phase_pdf(&self.now_ray.dir, &dir, 0.8);

            let (nee_result, transmittance) = scene.nee(org, rand, *sigma_e);
            if nee_result.pdf != 0. {
                let nee_hg_pdf = hg_phase_pdf(&self.now_ray.dir, &nee_result.dir, 0.8);
                let mis_weight = 1. / (nee_result.pdf + nee_hg_pdf);
                self.rad = self.rad
                    + multiply(self.throughput, nee_result.color)
                        * nee_hg_pdf
                        * transmittance
                        * mis_weight
                        / self.roulette_pdf;
            }

            self.pt_sample_pdf = hg_pdf;
            self.now_ray = Ray { org, dir };
            return false;
        }
        true
    }

    pub fn integrate(&mut self, scene: &Scene, rand: &mut XorRand) -> Color {
        for time in 0.. {
            if self.has_medium() && !self.has_ior() {
                if !self.freepath_sample(scene, rand) {
                    let roulette_prob = self.roulette(time);
                    if rand.next01() > roulette_prob {
                        break;
                    }
                    self.roulette_pdf *= roulette_prob;
                    continue;
                }
            } else {
                if !self.ray_intersect(scene) {
                    break;
                }
            }

            let roulette_prob = self.roulette(time);
            if rand.next01() > roulette_prob {
                break;
            }
            self.roulette_pdf *= roulette_prob;

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
                Bxdf::Dielectric { ior, trans_id } => {
                    self.trace_dielectric(ior, rand, trans_id);
                }
                Bxdf::MicroBrdf { ax, ay, cior, k } => {
                    self.trace_microbrdf(scene, rand, ax, ay, &cior, &k);
                }
                Bxdf::MicroBtdf { a, ior, trans_id } => {
                    self.trace_microbtdf(scene, rand, a, ior, trans_id);
                }
                Bxdf::Medium {
                    sigma_a: _,
                    sigma_s,
                    sigma_e,
                    trans_id,
                } => {
                    if self.is_into(trans_id) {
                        self.medium_stack.push((trans_id, 1., sigma_s, sigma_e));
                        self.now_ray = Ray {
                            org: self.record.pos - self.orienting_normal * 0.00001,
                            dir: self.now_ray.dir,
                        };
                    } else {
                        self.now_ray = Ray {
                            org: self.record.pos - self.orienting_normal * 0.00001,
                            dir: self.now_ray.dir,
                        };
                        self.remove_medium(trans_id);
                    }
                }
            }
        }
        self.rad
    }

    pub fn test_normal(&mut self, scene: &Scene) -> Color {
        if scene.intersect_obj(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
            self.record.normal
        } else {
            Vec3::new(0.)
        }
    }
}
