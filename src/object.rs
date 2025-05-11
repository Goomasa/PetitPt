use crate::aabb::AABB;
use crate::random::{FreshId, XorRand};
use crate::ray::*;
use crate::{material::Bxdf, math::*};

#[derive(Clone, Copy)]
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
        bbox: AABB,
    },

    Rectangle {
        axis: Axis,
        min_p: Point3,
        max_p: Point3,
        bxdf: Bxdf,
        color: Color,
        id: i32,
        bbox: AABB,
    },

    Triangle {
        p: Point3,
        pq: Point3,
        pr: Point3,
        normal: Vec3,
        bxdf: Bxdf,
        color: Color,
        id: i32,
        bbox: AABB,
    },
}

impl Object {
    pub fn set_sphere(
        center: Point3,
        radius: f64,
        bxdf: Bxdf,
        color: Color,
        freshid: &mut FreshId,
    ) -> Object {
        Object::Sphere {
            center,
            radius,
            bxdf,
            color,
            id: freshid.gen_id(),
            bbox: AABB {
                min_p: center - Vec3::new(radius),
                max_p: center + Vec3::new(radius),
            },
        }
    }

    pub fn set_rect(
        axis: Axis,
        p: Point3,
        q: Point3,
        bxdf: Bxdf,
        color: Color,
        freshid: &mut FreshId,
    ) -> Object {
        let max_p = Vec3(fmax(p.0, q.0), fmax(p.1, q.1), fmax(p.2, q.2));
        let min_p = Vec3(fmin(p.0, q.0), fmin(p.1, q.1), fmin(p.2, q.2));

        Object::Rectangle {
            axis,
            min_p,
            max_p,
            bxdf,
            color,
            id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.rev_aabb(),
        }
    }

    pub fn set_tri(
        p: Point3,
        q: Point3,
        r: Point3,
        bxdf: Bxdf,
        color: Color,
        freshid: &mut FreshId,
    ) -> Object {
        let normal = cross(q - p, r - p).normalize();
        let min_p = Vec3(
            fmin(p.0, fmin(q.0, r.0)),
            fmin(p.1, fmin(q.1, r.1)),
            fmin(p.2, fmin(q.2, r.2)),
        );
        let max_p = Vec3(
            fmax(p.0, fmax(q.0, r.0)),
            fmax(p.1, fmax(q.1, r.1)),
            fmax(p.2, fmax(q.2, r.2)),
        );
        Object::Triangle {
            p,
            pq: q - p,
            pr: r - p,
            normal,
            bxdf,
            color,
            id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.rev_aabb(),
        }
    }

    pub fn hit(&self, ray: &Ray, record: &mut HitRecord) {
        match self {
            Object::Sphere {
                center,
                radius,
                bxdf,
                color,
                id,
                ..
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
            Object::Rectangle {
                axis,
                min_p,
                max_p,
                bxdf,
                color,
                id,
                ..
            } => {
                if let Some((t, normal)) = hit_rect(axis, max_p, min_p, ray, record.distance) {
                    record.distance = t;
                    record.pos = ray.org + ray.dir * t;
                    record.normal = normal;
                    record.bxdf = *bxdf;
                    record.color = *color;
                    record.obj_id = *id;
                }
            }
            Object::Triangle {
                p,
                pq,
                pr,
                normal,
                bxdf,
                color,
                id,
                ..
            } => {
                if let Some((t, pos)) = hit_triangle(p, pq, pr, normal, ray, record.distance) {
                    record.distance = t;
                    record.pos = pos;
                    record.normal = *normal;
                    record.bxdf = *bxdf;
                    record.color = *color;
                    record.obj_id = *id;
                }
            }
        }
    }

    pub fn get_bxdf(&self) -> Bxdf {
        match self {
            Object::Sphere { bxdf, .. }
            | Object::Rectangle { bxdf, .. }
            | Object::Triangle { bxdf, .. } => *bxdf,
        }
    }

    pub fn get_id(&self) -> i32 {
        match self {
            Object::Sphere { id, .. }
            | Object::Rectangle { id, .. }
            | Object::Triangle { id, .. } => *id,
        }
    }

    pub fn get_bbox(&self) -> AABB {
        match self {
            Object::Sphere { bbox, .. }
            | Object::Rectangle { bbox, .. }
            | Object::Triangle { bbox, .. } => *bbox,
        }
    }

    pub fn get_area(&self) -> f64 {
        match self {
            Object::Sphere { radius, .. } => 2. * PI * radius,
            Object::Rectangle {
                axis, min_p, max_p, ..
            } => match axis {
                Axis::X => (max_p.1 - min_p.1) * (max_p.2 - min_p.2),
                Axis::Y => (max_p.0 - min_p.0) * (max_p.2 - min_p.2),
                Axis::Z => (max_p.0 - min_p.0) * (max_p.1 - min_p.1),
            },
            Object::Triangle { pq, pr, .. } => cross(*pq, *pr).length() / 2.,
        }
    }

    pub fn get_center(&self) -> Point3 {
        let bbox = self.get_bbox();
        (bbox.min_p + bbox.max_p) / 2.
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

pub fn hit_rect(
    axis: &Axis,
    max_p: &Point3,
    min_p: &Point3,
    ray: &Ray,
    max_dist: f64,
) -> Option<(f64, Vec3)> {
    let pos = match axis {
        Axis::X => max_p.0,
        Axis::Y => max_p.1,
        Axis::Z => max_p.2,
    };

    if let Some((t, normal)) = hit_plane(&axis, &pos, ray, max_dist) {
        let hitpoint = ray.org + ray.dir * t;
        match axis {
            Axis::X => {
                if hitpoint.1 > max_p.1
                    || hitpoint.1 < min_p.1
                    || hitpoint.2 > max_p.2
                    || hitpoint.2 < min_p.2
                {
                    return None;
                } else {
                    return Some((t, normal));
                }
            }
            Axis::Y => {
                if hitpoint.0 > max_p.0
                    || hitpoint.0 < min_p.0
                    || hitpoint.2 > max_p.2
                    || hitpoint.2 < min_p.2
                {
                    return None;
                } else {
                    return Some((t, normal));
                }
            }
            Axis::Z => {
                if hitpoint.0 > max_p.0
                    || hitpoint.0 < min_p.0
                    || hitpoint.1 > max_p.1
                    || hitpoint.1 < min_p.1
                {
                    return None;
                } else {
                    return Some((t, normal));
                }
            }
        }
    }
    None
}

pub fn hit_triangle(
    p: &Point3,
    pq: &Point3,
    pr: &Point3,
    normal: &Vec3,
    ray: &Ray,
    max_dist: f64,
) -> Option<(f64, Point3)> {
    let n_d = dot(*normal, ray.dir);
    if n_d == 0. {
        return None;
    }

    let t = dot(*normal, *p - ray.org) / n_d;
    if t > max_dist || t <= 0. {
        return None;
    }
    let pos = ray.org + ray.dir * t;
    let p_pos = pos - *p;
    let u = dot(cross(*pr, p_pos), *normal) / dot(cross(*pr, *pq), *normal);
    let v = dot(cross(*pq, p_pos), *normal) / dot(cross(*pq, *pr), *normal);

    if u + v > 1. || u < 0. || v < 0. {
        return None;
    }

    Some((t, pos))
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

pub fn sample_rect(
    org: Point3,
    axis: &Axis,
    max_p: &Point3,
    min_p: &Point3,
    rand: &mut XorRand,
) -> (f64, Vec3) {
    let diagnal = *max_p - *min_p;
    let area;
    let normal;
    let oa;
    let ob;
    match axis {
        Axis::X => {
            area = diagnal.1 * diagnal.2;
            normal = Vec3(1., 0., 0.);
            oa = Vec3(0., diagnal.1, 0.);
            ob = Vec3(0., 0., diagnal.2);
        }
        Axis::Y => {
            area = diagnal.0 * diagnal.2;
            normal = Vec3(0., 1., 0.);
            oa = Vec3(diagnal.0, 0., 0.);
            ob = Vec3(0., 0., diagnal.2);
        }
        Axis::Z => {
            area = diagnal.0 * diagnal.1;
            normal = Vec3(0., 0., 1.);
            oa = Vec3(diagnal.0, 0., 0.);
            ob = Vec3(0., diagnal.1, 0.);
        }
    }

    let sample_pos = *min_p + oa * rand.next01() + ob * rand.next01();
    let mut dir = sample_pos - org;
    let l_sq = dir.length_sq();
    dir = dir.normalize();
    let cos_theta = dot(dir, normal).abs();
    (l_sq / (area * cos_theta), dir)
}

pub fn sample_triangle(
    org: Point3,
    p: &Point3,
    pq: &Point3,
    pr: &Point3,
    normal: &Vec3,
    area: f64,
    rand: &mut XorRand,
) -> (f64, Vec3) {
    let mut r1 = rand.next01();
    let mut r2 = rand.next01();
    if r1 + r2 > 1. {
        r1 = 1. - r1;
        r2 = 1. - r2;
    }
    let sample_pos = *p + *pq * r1 + *pr * r2;
    let mut dir = sample_pos - org;

    let l_sq = dir.length_sq();
    dir = dir.normalize();
    let cos_theta = dot(dir, *normal).abs();
    (l_sq / (cos_theta * area), dir)
}

pub fn sample_sphere_pdf(org: Point3, center: &Point3, radius: f64) -> f64 {
    let cos_mu = (1. - (radius * radius / (*center - org).length_sq())).sqrt();
    1. / (2. * PI * (1. - cos_mu))
}

pub fn sample_rect_pdf(org: Point3, pos: Point3, obj: &Object, normal: Vec3) -> f64 {
    let l_sq = (pos - org).length_sq();
    let cos_theta = dot((pos - org).normalize(), normal).abs();
    l_sq / (obj.get_area() * cos_theta)
}

pub fn sample_tri_pdf(org: Point3, pos: Point3, obj: &Object, normal: Vec3) -> f64 {
    let l_sq = (pos - org).length_sq();
    let cos_theta = dot((pos - org).normalize(), normal).abs();
    l_sq / (obj.get_area() * cos_theta)
}
