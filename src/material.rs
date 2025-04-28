use crate::{
    math::{cross, dot, Vec3, EPS, PI},
    random::XorRand,
};

#[derive(Clone, Copy)]
pub enum Bxdf {
    Lambertian,
    Specular,
    Dielectric { ior: f64 },
    Light,
}

impl Bxdf {
    pub fn is_light(&self) -> bool {
        match self {
            Self::Light => true,
            _ => false,
        }
    }
}

pub fn sample_lambert(normal: &Vec3, rand: &mut XorRand) -> Vec3 {
    let w = *normal;
    let u = if w.0 > EPS || w.0 < (-EPS) {
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

pub fn reflection_dir(normal: Vec3, in_dir: Vec3) -> Vec3 {
    in_dir + normal * dot(in_dir, normal) * (-2.)
}

pub fn refraction_dir(
    into: bool,
    ior: f64,
    normal: Vec3,
    in_dir: Vec3,
    rand: &mut XorRand,
) -> (bool, Vec3, f64, f64) {
    //return (is_refract,new_dir, fresnel, prob)
    let reflection_dir = reflection_dir(normal, in_dir);
    let nnt = if into { 1. / ior } else { ior };
    let ddn = dot(in_dir, normal);
    let cos2t = 1. - nnt * nnt * (1. - ddn * ddn);

    if cos2t < 0. {
        return (false, reflection_dir, 1.0, 1.0);
    }

    let refraction_dir = (-normal * (cos2t.sqrt()) + (in_dir - normal * ddn) * nnt).normalize();
    let a = ior - 1.;
    let b = ior + 1.;
    let r0 = (a * a) / (b * b);
    let c = if into {
        1. + ddn
    } else {
        1. - dot(refraction_dir, -normal)
    };
    let fresnel_reflectance = r0 + (1. - r0) * c.powi(5);
    let fresnel_transmittance = (1.0 - fresnel_reflectance) * nnt * nnt;
    let reflection_prob = 0.25 + 0.5 * fresnel_reflectance;

    if rand.next01() < reflection_prob {
        (false, reflection_dir, fresnel_reflectance, reflection_prob)
    } else {
        (
            true,
            refraction_dir,
            fresnel_transmittance,
            1. - reflection_prob,
        )
    }
}

pub fn sample_lambert_pdf(dir: Vec3, normal: Vec3) -> f64 {
    dot(dir, normal).abs() / PI
}
