use std::ops::{Add, Div, Mul, Neg, Sub};

pub const PI: f64 = 3.14159265358979323846;
pub const INF: f64 = 1e15;
pub const EPS: f64 = 1e-6;

#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub f64, pub f64, pub f64);

pub type Point3 = Vec3;
pub type Color = Vec3;

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self::Output {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl Vec3 {
    pub fn new(c: f64) -> Self {
        Vec3(c, c, c)
    }

    pub fn length_sq(&self) -> f64 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn length(&self) -> f64 {
        self.length_sq().sqrt()
    }

    pub fn normalize(&self) -> Self {
        *self / self.length()
    }
}

pub fn multiply(v: Vec3, w: Vec3) -> Vec3 {
    Vec3(v.0 * w.0, v.1 * w.1, v.2 * w.2)
}

pub fn dot(v: Vec3, w: Vec3) -> f64 {
    v.0 * w.0 + v.1 * w.1 + v.2 * w.2
}

pub fn cross(v: Vec3, w: Vec3) -> Vec3 {
    Vec3(
        v.1 * w.2 - v.2 * w.1,
        v.2 * w.0 - v.0 * w.2,
        v.0 * w.1 - v.1 * w.0,
    )
}

pub fn max_elm(v: &Vec3) -> f64 {
    let mut max = v.0;

    if max < v.1 {
        max = v.1;
    }
    if max < v.2 {
        max = v.2
    }
    max
}

pub fn fmax(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn fmin(a: f64, b: f64) -> f64 {
    if a > b {
        b
    } else {
        a
    }
}

pub fn is_valid(v: &Vec3) -> bool {
    if v.0.is_nan() || v.1.is_nan() || v.2.is_nan() {
        return false;
    } else if v.0 < 0. || v.1 < 0. || v.2 < 0. {
        return false;
    } else if v.0 > 100000. || v.1 > 100000. || v.2 > 100000. {
        return false;
    }
    true
}

pub fn gamma_rev(v: Color) -> (u32, u32, u32) {
    let r = (v.0.clamp(0., 1.).powf(1. / 2.2) * 255.) as u32;
    let g = (v.1.clamp(0., 1.).powf(1. / 2.2) * 255.) as u32;
    let b = (v.2.clamp(0., 1.).powf(1. / 2.2) * 255.) as u32;
    (r, g, b)
}

pub fn clamp_vec(v: Vec3, min: f64, max: f64) -> Vec3 {
    Vec3(
        v.0.clamp(min, max),
        v.1.clamp(min, max),
        v.2.clamp(min, max),
    )
}
