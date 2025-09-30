use num_complex::Complex;
// using crate "num-complex", https://github.com/rust-num/num-complex

use crate::{
    math::{cross, dot, fmax, Color, Vec3, EPS, PI},
    random::XorRand,
};

#[derive(Clone, Copy)]
pub enum Bxdf {
    Lambertian,
    Specular {
        cior: Color,
        k: Color,
    },
    Dielectric {
        ior: f64,
        trans_id: i32,
    },
    Light,
    MicroBrdf {
        ax: f64,
        ay: f64,
        cior: Color,
        k: Color,
    },
    MicroBtdf {
        a: f64,
        ior: f64,
        trans_id: i32,
    },
    //not bxdf
    Medium {
        sigma_a: f64,
        sigma_s: f64,
        sigma_e: f64,
        trans_id: i32,
    },
}

impl Bxdf {
    pub fn is_light(&self) -> bool {
        match self {
            Self::Light => true,
            _ => false,
        }
    }

    pub fn is_medium(&self) -> bool {
        match self {
            Self::Medium { .. } => true,
            _ => false,
        }
    }

    pub fn set_spec_di() -> Self {
        Self::Specular {
            cior: Vec3::new(-1.),
            k: Vec3::new(-1.),
        }
    }

    pub fn set_dielectric(ior: f64, trans_id: i32) -> Self {
        Self::Dielectric { ior, trans_id }
    }

    pub fn set_spec_co(cior: Color, k: Color) -> Self {
        Self::Specular { cior, k }
    }

    pub fn set_microbrdf_di(ax: f64, ay: f64) -> Self {
        Self::MicroBrdf {
            ax,
            ay,
            cior: Vec3::new(-1.),
            k: Vec3::new(-1.),
        }
    }

    pub fn set_microbrdf_co(ax: f64, ay: f64, cior: Color, k: Color) -> Self {
        Self::MicroBrdf { ax, ay, cior, k }
    }

    pub fn set_microbtdf(a: f64, ior: f64, trans_id: i32) -> Self {
        Self::MicroBtdf { a, ior, trans_id }
    }

    pub fn set_medium(s_ab: f64, s_sc: f64, trans_id: i32) -> Self {
        Self::Medium {
            sigma_a: s_ab,
            sigma_s: s_sc,
            sigma_e: s_ab + s_sc,
            trans_id,
        }
    }

    pub fn get_sigma_ex(&self) -> f64 {
        match self {
            Self::Medium { sigma_e: e, .. } => *e,
            _ => -1.,
        }
    }
}

pub fn sample_lambert(normal: &Vec3, rand: &mut XorRand) -> Vec3 {
    let w = *normal;
    let u = if w.0.abs() > EPS {
        cross(w, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(w, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();
    let sin_theta_sq = rand.next01();
    let sin_theta = sin_theta_sq.sqrt();

    (u * sin_theta * (phi.cos()) + v * sin_theta * (phi.sin()) + w * ((1. - sin_theta_sq).sqrt()))
        .normalize()
}

pub fn sample_lambert_pdf(dir: &Vec3, normal: &Vec3) -> f64 {
    dot(*dir, *normal).abs() / PI
}

pub fn reflection_dir(normal: Vec3, in_dir: Vec3) -> Vec3 {
    in_dir + normal * dot(in_dir, normal) * (-2.)
}

pub fn refraction_dir(
    into: bool,
    ior_i: f64,
    ior_t: f64,
    normal: Vec3,
    in_dir: Vec3,
    rand: &mut XorRand,
) -> (bool, Vec3, f64, f64) {
    //return (is_refract, new_dir, fresnel, prob)
    let reflection_dir = reflection_dir(normal, in_dir);
    let nnt = if into { ior_i / ior_t } else { ior_t / ior_i };
    let ddn = dot(in_dir, normal);
    let cos2t = 1. - nnt * nnt * (1. - ddn * ddn);

    if cos2t < 0. {
        return (false, reflection_dir, 1.0, 1.0);
    }

    let refraction_dir = (-normal * (cos2t.sqrt()) + (in_dir - normal * ddn) * nnt).normalize();
    let a = ior_t - ior_i;
    let b = ior_t + ior_i;
    let r0 = (a * a) / (b * b);
    let c = if into {
        1. + ddn
    } else {
        1. - dot(refraction_dir, -normal)
    };
    let fresnel_reflectance = r0 + (1. - r0) * c.powi(5);
    let reflection_prob = fresnel_reflectance;

    if rand.next01() < reflection_prob {
        (false, reflection_dir, fresnel_reflectance, reflection_prob)
    } else {
        (
            true,
            refraction_dir,
            1. - fresnel_reflectance,
            1. - reflection_prob,
        )
    }
}

pub fn sample_ggx_vndf(normal: &Vec3, wi: &Vec3, ax: f64, ay: f64, rand: &mut XorRand) -> Vec3 {
    let u = if normal.0.abs() > EPS {
        cross(*normal, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(*normal, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(*normal, u);

    let ve = Vec3(dot(u, *wi), dot(v, *wi), dot(*normal, *wi));

    let vh = Vec3(ax * ve.0, ay * ve.1, ve.2).normalize();
    let lensq = vh.0 * vh.0 + vh.1 * vh.1;
    let t1 = if lensq > 0. {
        Vec3(-vh.1, vh.0, 0.) * (1. / lensq.sqrt())
    } else {
        Vec3(1., 0., 0.)
    };
    let t2 = cross(vh, t1);

    let r = rand.next01().sqrt();
    let phi = 2. * PI * rand.next01();
    let p1 = r * phi.cos();
    let mut p2 = r * phi.sin();
    let s = 0.5 * (1. + vh.2);
    p2 = (1. - s) * (1. - p1 * p1).sqrt() + s * p2;

    let nh = t1 * p1 + t2 * p2 + vh * fmax(1. - p1 * p1 - p2 * p2, 0.).sqrt();
    let vn = Vec3(ax * nh.0, ay * nh.1, fmax(nh.2, 0.));
    (u * vn.0 + v * vn.1 + *normal * vn.2).normalize()
}

pub fn shadow_mask_fn(alpha_sq: f64, v: &Vec3, normal: &Vec3) -> f64 {
    let cos_theta = dot(*v, *normal);
    let tan_theta_sq = 1. / (cos_theta * cos_theta) - 1.;

    2. / (1. + (1. + alpha_sq * tan_theta_sq).sqrt())
}

pub fn ggx_alpha2(ax: f64, ay: f64, wi: &Vec3, normal: &Vec3) -> f64 {
    let wi_dash = *wi - *normal * dot(*wi, *normal);
    let u = cross(*normal, Vec3(0., 1., 0.)).normalize();
    let v = cross(*normal, u);

    let tan_phi = dot(wi_dash, v) / dot(wi_dash, u);
    let cos_phi_sq = 1. / (1. + tan_phi * tan_phi);
    ax * ax * cos_phi_sq + ay * ay * (1. - cos_phi_sq)
}

pub fn fr_dielectric_col(f0: &Color, wi: &Vec3, vn: &Vec3) -> Color {
    *f0 + (Vec3::new(1.) - *f0) * (1. - dot(*wi, *vn)).clamp(0., 1.).powf(5.)
}

pub fn fr_dielectric_ior(into: bool, ior_i: f64, ior_t: f64, wo: &Vec3, vn: &Vec3) -> f64 {
    let a = ior_i - ior_t;
    let b = ior_i + ior_t;
    let r0 = (a * a) / (b * b);
    let c = if into {
        1. + dot(*wo, -*vn)
    } else {
        1. - dot(*wo, -*vn)
    };

    r0 + (1. - r0) * c.powf(5.)
}

pub fn fr_conductor(cior: &Color, k: &Color, wi: &Vec3, vn: &Vec3) -> Color {
    let cos_theta = dot(*wi, *vn);
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();
    let cior = [cior.0, cior.1, cior.2];
    let k = [k.0, k.1, k.2];
    let mut refl = [0.; 3];

    for i in 0..3 {
        let n_complex = Complex::new(cior[i], k[i]);
        let sin_theta_i_sq = sin_theta * sin_theta;

        let n_complex_sq = n_complex * n_complex;
        let sqrt_term = (n_complex_sq - Complex::new(sin_theta_i_sq, 0.0)).sqrt();

        let rs =
            (Complex::new(cos_theta, 0.0) - sqrt_term) / (Complex::new(cos_theta, 0.0) + sqrt_term);
        let rs_reflectance = rs.norm_sqr();

        let rp_numer = n_complex_sq * Complex::new(cos_theta, 0.0) - sqrt_term;
        let rp_denom = n_complex_sq * Complex::new(cos_theta, 0.0) + sqrt_term;
        let rp = rp_numer / rp_denom;
        let rp_reflectance = rp.norm_sqr();

        refl[i] = (rs_reflectance + rp_reflectance) / 2.;
    }
    Vec3(refl[0], refl[1], refl[2])
}

pub fn ggx_normal_df(alpha_sq: f64, ax: f64, ay: f64, normal: &Vec3, vn: &Vec3) -> f64 {
    let cos_theta = dot(*vn, *normal);
    let tan_theta_sq = 1. / (cos_theta * cos_theta) - 1.;

    let vn_dash = *vn - *normal * cos_theta;
    let u = cross(*normal, Vec3(0., 1., 0.)).normalize();
    let v = cross(*normal, u);

    let tan_phi = dot(vn_dash, v) / dot(vn_dash, u);
    let cos_phi_sq = 1. / (1. + tan_phi * tan_phi);

    if ax == 0. || ay == 0. {
        0.
    } else {
        let s = 1. + (cos_phi_sq / (ax * ax) + (1. - cos_phi_sq) / (ay * ay)) * tan_theta_sq;
        1. / (PI * alpha_sq * cos_theta.powf(4.) * s * s)
    }
}

pub fn micro_btdf_j(ior_i: f64, ior_o: f64, wi: &Vec3, wo: &Vec3, wh: &Vec3) -> f64 {
    let dot_wo_wh = dot(*wo, *wh);
    ior_o * ior_o * dot_wo_wh.abs() / (ior_i * dot(*wi, *wh) + ior_o * dot_wo_wh).powf(2.)
}

pub fn sample_hg_phase(dir: &Vec3, g: f64, rand: &mut XorRand) -> Vec3 {
    let phi = 2. * PI * rand.next01();
    let cos_theta = if g < EPS {
        1. - 2. * rand.next01()
    } else {
        let tmp = ((1. - g * g) / (1. + g - 2. * g * rand.next01())).powi(2);
        -1. / (2. * g) * (1. + g * g - tmp)
    };
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();

    let w = *dir;
    let u = if w.0.abs() > EPS {
        cross(w, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(w, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(w, u);

    u * sin_theta * phi.cos() + v * sin_theta * phi.sin() + w * cos_theta
}

pub fn hg_phase_pdf(wo: &Vec3, wi: &Vec3, g: f64) -> f64 {
    let tmp = (1. + g * g + 2. * g * dot(*wo, *wi)).powf(1.5);
    1. / (4. * PI) * (1. - g * g) / tmp
}
