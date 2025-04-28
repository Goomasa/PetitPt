use crate::random::XorRand;
use crate::ray::*;
use crate::{material::Bxdf, math::*};

pub enum Axis {
    X,
    Y,
    Z,
}

pub enum Object {
    Sphere {
        center: Point3,
        radius: f64,
        bxdf: Bxdf,
        color: Color,
        id: i32,
    },

    Plane {
        axis: Axis,
        pos: f64,
        bxdf: Bxdf,
        color: Color,
        id: i32,
    },
}

impl Object {
    pub fn hit(&self, ray: &Ray, record: &mut HitRecord) {
        match self {
            Object::Sphere {
                center,
                radius,
                bxdf,
                color,
                id,
            } => {
                if let Some((t, normal)) = hit_sphere(center, radius, ray, record.distance) {
                    record.distance = t;
                    record.pos = ray.org + ray.dir * t;
                    record.normal = normal;
                    record.bxdf = *bxdf;
                    record.color = *color;
                    record.obj_id = *id;
                }
            }
            Object::Plane {
                axis,
                pos,
                bxdf,
                color,
                id,
            } => {
                if let Some((t, normal)) = hit_plane(axis, pos, ray, record.distance) {
                    record.distance = t;
                    record.pos = ray.org + ray.dir * t;
                    record.normal = normal;
                    record.bxdf = *bxdf;
                    record.color = *color;
                    record.obj_id = *id;
                }
            }
        }
    }

    pub fn get_bxdf(&self) -> Bxdf {
        match self {
            Object::Sphere { bxdf, .. } | Object::Plane { bxdf, .. } => *bxdf,
        }
    }
}

fn hit_sphere(center: &Point3, radius: &f64, ray: &Ray, max_dist: f64) -> Option<(f64, Vec3)> {
    //if hit, return (distant,normal)
    let oc = *center - ray.org;
    let oc_dir = dot(oc, ray.dir);
    let disc = oc_dir * oc_dir - oc.length_sq() + radius * radius;

    if disc < 0. {
        return None;
    }

    let t1 = oc_dir - disc.sqrt();
    let t2 = oc_dir + disc.sqrt();
    let t;

    if t1 > 0. {
        t = t1;
    } else if t2 > 0. {
        t = t2;
    } else {
        return None;
    }

    if t > max_dist {
        return None;
    }

    Some((t, (ray.dir * t - oc).normalize()))
}

fn hit_plane(axis: &Axis, pos: &f64, ray: &Ray, max_dist: f64) -> Option<(f64, Vec3)> {
    //if hit, return (distance, normal)
    match axis {
        Axis::X => {
            if ray.dir.0 == 0. {
                return None;
            }

            let t = (pos - ray.org.0) / ray.dir.0;
            if t > max_dist || t < 0. {
                None
            } else {
                Some((t, Vec3(1., 0., 0.)))
            }
        }
        Axis::Y => {
            if ray.dir.1 == 0. {
                return None;
            }

            let t = (pos - ray.org.1) / ray.dir.1;
            if t > max_dist || t < 0. {
                None
            } else {
                Some((t, Vec3(0., 1., 0.)))
            }
        }
        Axis::Z => {
            if ray.dir.2 == 0. {
                return None;
            }

            let t = (pos - ray.org.2) / ray.dir.2;
            if t > max_dist || t < 0. {
                None
            } else {
                Some((t, Vec3(0., 0., 1.)))
            }
        }
    }
}

pub fn sample_sphere(org: Point3, center: &Point3, radius: f64, rand: &mut XorRand) -> (f64, Vec3) {
    let pc = *center - org;
    let cos_mu = (1. - (radius * radius / pc.length_sq())).sqrt();

    let w = pc.normalize();
    let u = if w.0 > EPS || w.0 < (-EPS) {
        cross(w, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(w, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();
    let cos_theta = 1. - rand.next01() * (1. - cos_mu);
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();

    let dir = u * sin_theta * phi.cos() + v * sin_theta * phi.sin() + w * cos_theta;
    let pdf = 1. / (2. * PI * (1. - cos_mu));
    (pdf, dir)
}

pub fn sample_sphere_pdf(org: Point3, center: &Point3, radius: f64) -> f64 {
    let cos_mu = (1. - (radius * radius / (*center - org).length_sq())).sqrt();
    1. / (2. * PI * (1. - cos_mu))
}
